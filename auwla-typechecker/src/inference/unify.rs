use auwla_ast::Type;
use std::collections::HashMap;

/// A disjoint-set (Union-Find) data structure to track type variables
/// and their unified constraints during typechecking.
#[derive(Debug, Clone)]
pub struct Unifier {
    parent: HashMap<usize, usize>,
    // Mapping from type variable ID to an optionally resolved type string/struct.
    types: HashMap<usize, Type>,
    next_id: usize,
}

impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Unifier {
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
            types: HashMap::new(),
            next_id: 0,
        }
    }

    /// Allocates a new unknown Type Variable and returns its unique ID.
    pub fn new_type_var(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.parent.insert(id, id);
        id
    }

    /// Finds the representative (root) ID for a given type variable.
    pub fn find(&mut self, id: usize) -> usize {
        let mut root = id;
        while let Some(&p) = self.parent.get(&root) {
            if p == root {
                break;
            }
            root = p;
        }

        // Path compression
        let mut curr = id;
        while let Some(&p) = self.parent.get(&curr) {
            if p == root {
                break;
            }
            self.parent.insert(curr, root);
            curr = p;
        }

        root
    }

    /// Unifies two type variables, assigning `id2` to point to `id1`'s constraint group.
    pub fn unify_vars(&mut self, id1: usize, id2: usize) -> Result<(), String> {
        let root1 = self.find(id1);
        let root2 = self.find(id2);

        if root1 != root2 {
            // Check if both have concrete types. If so, they must unify.
            let ty1_opt = self.types.get(&root1).cloned();
            let ty2_opt = self.types.get(&root2).cloned();

            if let (Some(ty1), Some(ty2)) = (ty1_opt.clone(), ty2_opt.clone()) {
                self.unify(&ty1, &ty2)?;
            }

            self.parent.insert(root2, root1);

            // Inherit the type if root2 had one and root1 didn't.
            if ty1_opt.is_none() && ty2_opt.is_some() {
                self.types.insert(root1, ty2_opt.unwrap());
            }
        }
        Ok(())
    }

    /// Binds a type variable to a concrete type.
    pub fn bind(&mut self, id: usize, ty: &Type) -> Result<(), String> {
        let root = self.find(id);
        if let Some(existing_ty) = self.types.get(&root).cloned() {
            self.unify(&existing_ty, ty)
        } else {
            // Occurs check logic could be injected here later
            self.types.insert(root, ty.clone());
            Ok(())
        }
    }

    /// The core unification algorithm to unify two potentially complex AST Types.
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), String> {
        match (t1, t2) {
            (Type::InferenceVar(id1), Type::InferenceVar(id2)) => self.unify_vars(*id1, *id2),
            (Type::InferenceVar(id), other) | (other, Type::InferenceVar(id)) => {
                self.bind(*id, other)
            }
            (Type::Basic(n1), Type::Basic(n2)) if n1 == n2 => Ok(()),
            (Type::Custom(n1), Type::Custom(n2)) if n1 == n2 => Ok(()),
            (
                Type::Result {
                    ok_type: o1,
                    err_type: e1,
                },
                Type::Result {
                    ok_type: o2,
                    err_type: e2,
                },
            ) => {
                self.unify(o1, o2)?;
                self.unify(e1, e2)
            }
            (Type::Array(i1), Type::Array(i2)) => self.unify(i1, i2),
            (Type::Optional(i1), Type::Optional(i2)) => self.unify(i1, i2),
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                if p1.len() != p2.len() {
                    return Err(format!(
                        "Cannot unify functions with different arity: {} vs {}",
                        p1.len(),
                        p2.len()
                    ));
                }
                for (arg1, arg2) in p1.iter().zip(p2.iter()) {
                    self.unify(arg1, arg2)?;
                }
                self.unify(r1, r2)
            }
            (Type::Generic(n1, args1), Type::Generic(n2, args2)) if n1 == n2 => {
                if args1.len() != args2.len() {
                    return Err(format!(
                        "Cannot unify generic {} with different type arguments arity",
                        n1
                    ));
                }
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify(a1, a2)?;
                }
                Ok(())
            }
            // Allow unifying anything with 'unknown' for permissive checking during transition
            (Type::Basic(n), _) if n == "unknown" => Ok(()),
            (_, Type::Basic(n)) if n == "unknown" => Ok(()),
            // Otherwise, type mismatch
            _ => Err(format!(
                "Type Mismatch: expected {}, found {}",
                t1.to_string(),
                t2.to_string()
            )),
        }
    }

    /// Fully resolves a type by walking the disjoint set and replacing all InferenceVars.
    pub fn resolve(&mut self, ty: &Type) -> Type {
        match ty {
            Type::InferenceVar(id) => {
                let root = self.find(*id);
                if let Some(concrete_ty) = self.types.get(&root).cloned() {
                    // recursively resolve
                    self.resolve(&concrete_ty)
                } else {
                    // Unresolved variable retains its identity
                    Type::InferenceVar(root)
                }
            }
            Type::Array(inner) => Type::Array(Box::new(self.resolve(inner))),
            Type::Optional(inner) => Type::Optional(Box::new(self.resolve(inner))),
            Type::Result { ok_type, err_type } => Type::Result {
                ok_type: Box::new(self.resolve(ok_type)),
                err_type: Box::new(self.resolve(err_type)),
            },
            Type::Function(params, ret) => {
                let resolved_params = params.iter().map(|p| self.resolve(p)).collect();
                Type::Function(resolved_params, Box::new(self.resolve(ret)))
            }
            Type::Generic(name, args) => {
                let resolved_args = args.iter().map(|a| self.resolve(a)).collect();
                Type::Generic(name.clone(), resolved_args)
            }
            _ => ty.clone(), // Basic, Custom, TypeVar remain unchanged
        }
    }
}
