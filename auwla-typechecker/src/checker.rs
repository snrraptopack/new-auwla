use crate::scope::{Mutability, Scope};
use auwla_ast::{Expr, Program, Stmt, Type};

pub struct Typechecker {
    scopes: Vec<Scope>,
    current_return_type: Option<Option<Type>>,
}

impl Default for Typechecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Typechecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            current_return_type: None,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop().expect("Cannot pop the global scope");
    }

    pub fn declare_variable(&mut self, name: String, ty: Type, mutability: Mutability) {
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.mutability.insert(name.clone(), mutability);
        current_scope.variables.insert(name, ty);
    }

    pub fn declare_function(&mut self, name: String, params: Vec<Type>, ret: Option<Type>) {
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.functions.insert(name, (params, ret));
    }

    pub fn is_mutable(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if let Some(m) = scope.mutability.get(name) {
                return *m == Mutability::Mutable;
            }
        }
        false // unknown vars treated as immutable
    }

    pub fn lookup_variable(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.variables.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub fn lookup_function(&self, name: &str) -> Option<(Vec<Type>, Option<Type>)> {
        for scope in self.scopes.iter().rev() {
            if let Some(sig) = scope.functions.get(name) {
                return Some(sig.clone());
            }
        }
        None
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let {
                name,
                ty,
                initializer,
            } => {
                let init_ty = self.check_expr(initializer)?;
                let final_ty = if let Some(declared_ty) = ty {
                    self.assert_type_eq(declared_ty, &init_ty)?;
                    declared_ty.clone()
                } else {
                    init_ty
                };
                self.declare_variable(name.clone(), final_ty, Mutability::Immutable);
                Ok(())
            }
            Stmt::Var {
                name,
                ty,
                initializer,
            } => {
                let init_ty = self.check_expr(initializer)?;
                let final_ty = if let Some(declared_ty) = ty {
                    self.assert_type_eq(declared_ty, &init_ty)?;
                    declared_ty.clone()
                } else {
                    init_ty
                };
                self.declare_variable(name.clone(), final_ty, Mutability::Mutable);
                Ok(())
            }
            Stmt::Assign { name, value } => {
                let var_ty = self.lookup_variable(name).ok_or_else(|| {
                    format!(
                        "Undefined variable '{}' — declare it with `var` first",
                        name
                    )
                })?;

                if !self.is_mutable(name) {
                    return Err(format!(
                        "Cannot reassign '{}' — it was declared with `let` (immutable). Use `var` to allow reassignment.",
                        name
                    ));
                }

                let val_ty = self.check_expr(value)?;
                self.assert_type_eq(&var_ty, &val_ty)?;
                Ok(())
            }
            Stmt::Fn {
                name,
                params,
                return_ty,
                body,
            } => {
                let param_types = params.iter().map(|(_, ty)| ty.clone()).collect();
                self.declare_function(name.clone(), param_types, return_ty.clone());

                let prev_return = self.current_return_type.take();
                self.current_return_type = Some(return_ty.clone());

                self.enter_scope();
                // Fn params are always mutable within their scope
                for (param_name, ty) in params {
                    self.declare_variable(param_name.clone(), ty.clone(), Mutability::Mutable);
                }
                for body_stmt in body {
                    self.check_stmt(body_stmt)?;
                }
                self.exit_scope();

                self.current_return_type = prev_return;
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let _cond_ty = self.check_expr(condition)?;

                // Condition must strictly evaluate to a boolean expression
                // Currently our language binary expressions evaluate to the LHS type.
                // We will enforce this loosely for now but catch invalid types later when we add Booleans explicitly.
                // self.assert_type_eq(&Type::Basic("bool".to_string()), &cond_ty)?;

                self.enter_scope();
                for stmt in then_branch {
                    self.check_stmt(stmt)?;
                }
                self.exit_scope();

                if let Some(els) = else_branch {
                    self.enter_scope();
                    for stmt in els {
                        self.check_stmt(stmt)?;
                    }
                    self.exit_scope();
                }
                Ok(())
            }
            Stmt::Return(expr_opt) => {
                let actual_ty = if let Some(expr) = expr_opt {
                    Some(self.check_expr(expr)?)
                } else {
                    None
                };

                if let Some(expected_ty_opt) = &self.current_return_type {
                    match (expected_ty_opt, actual_ty) {
                        (Some(expected), Some(actual)) => {
                            self.assert_type_eq(expected, &actual).map_err(|_| {
                                format!(
                                    "Strict Type error: Function expects to return '{:?}', but returned '{:?}'",
                                    expected, actual
                                )
                            })?;
                        }
                        (None, Some(actual)) => {
                            return Err(format!(
                                "Strict Type error: Function expected to return nothing, but returned '{:?}'",
                                actual
                            ));
                        }
                        (Some(expected), None) => {
                            return Err(format!(
                                "Strict Type error: Function expects to return '{:?}', but returned nothing",
                                expected
                            ));
                        }
                        (None, None) => {}
                    }
                } else {
                    return Err(
                        "Strict Type error: 'return' statement outside of function".to_string()
                    );
                }

                Ok(())
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
        }
    }

    pub fn check_expr(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::StringLit(_) => Ok(Type::Basic("string".to_string())),
            Expr::NumberLit(_) => Ok(Type::Basic("number".to_string())),
            Expr::Identifier(name) => self
                .lookup_variable(name)
                .ok_or_else(|| format!("Undefined variable: '{}'", name)),
            Expr::Binary { op, left, right } => {
                let left_ty = self.check_expr(left)?;
                let right_ty = self.check_expr(right)?;

                self.assert_type_eq(&left_ty, &right_ty)
                    .map_err(|_| format!("Strict Type inferred error: Operators must have matching types. Left: {:?}, Right: {:?}", left_ty, right_ty))?;

                // Return boolean for comparative ops, otherwise return the evaluated mathematical type (number/string)
                match op {
                    auwla_ast::BinaryOp::Eq
                    | auwla_ast::BinaryOp::Neq
                    | auwla_ast::BinaryOp::Lt
                    | auwla_ast::BinaryOp::Gt
                    | auwla_ast::BinaryOp::Lte
                    | auwla_ast::BinaryOp::Gte => Ok(Type::Basic("bool".to_string())),
                    _ => Ok(left_ty),
                }
            }
            // If it evaluates `some()`, it wraps the OK branch.
            Expr::Some(inner) => {
                let inner_ty = self.check_expr(inner)?;
                Ok(Type::Result {
                    ok_type: Box::new(inner_ty),
                    err_type: Box::new(Type::Basic("unknown".to_string())),
                })
            }
            // If it evaluates `none()`, it wraps the ERR branch.
            Expr::None(inner) => {
                let inner_ty = self.check_expr(inner)?;
                Ok(Type::Result {
                    ok_type: Box::new(Type::Basic("unknown".to_string())),
                    err_type: Box::new(inner_ty),
                })
            }
        }
    }

    fn assert_type_eq(&self, expected: &Type, actual: &Type) -> Result<(), String> {
        // Handle `type?error_type` resolution from `some()` and `none()` with unknowns.
        if let (
            Type::Result {
                ok_type: e_ok,
                err_type: e_err,
            },
            Type::Result {
                ok_type: a_ok,
                err_type: a_err,
            },
        ) = (expected, actual)
        {
            let ok_match = if let Type::Basic(name) = &**a_ok {
                name == "unknown" || e_ok == a_ok
            } else {
                e_ok == a_ok
            };

            let err_match = if let Type::Basic(name) = &**a_err {
                name == "unknown" || e_err == a_err
            } else {
                e_err == a_err
            };

            if ok_match && err_match {
                return Ok(());
            }
        }

        if expected == actual {
            Ok(())
        } else {
            Err(format!(
                "Strict Type mismatch: Expected '{:?}', found '{:?}'",
                expected, actual
            ))
        }
    }
}
