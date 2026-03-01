use crate::TypeError;
use crate::scope::{Mutability, Scope};
use auwla_ast::{Program, Span, Type};

use std::collections::HashMap;

pub struct Typechecker {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) current_return_type: Option<Option<Type>>,
    pub(crate) current_function_name: Option<String>,
    pub(crate) structs: HashMap<String, Vec<(String, Type)>>,
    pub(crate) enums: HashMap<String, Vec<(String, Vec<Type>)>>,
    pub(crate) type_aliases: HashMap<String, Type>,
    /// type_name -> [(type_params, method_name, is_static, params_with_types, return_ty)]
    pub extensions: HashMap<
        String,
        Vec<(
            Option<Vec<String>>,
            String,
            bool,
            Vec<(String, Type)>,
            Option<Type>,
        )>,
    >,
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
            current_function_name: None,
            structs: HashMap::new(),
            enums: HashMap::new(),
            type_aliases: HashMap::new(),
            extensions: HashMap::new(),
        }
    }

    /// Returns a reference to the extension method registry.
    /// Used by the code generator to identify extension call sites.
    pub fn get_extensions(
        &self,
    ) -> &HashMap<
        String,
        Vec<(
            Option<Vec<String>>,
            String,
            bool,
            Vec<(String, Type)>,
            Option<Type>,
        )>,
    > {
        &self.extensions
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scopes.pop().expect("Cannot pop the global scope");
    }

    pub(crate) fn error<T>(&self, span: Span, message: impl Into<String>) -> Result<T, TypeError> {
        Err(TypeError {
            span,
            message: message.into(),
        })
    }

    pub(crate) fn declare_variable(
        &mut self,
        _span: Span, // Will be used for error reporting if needed
        name: String,
        ty: Type,
        mutability: Mutability,
    ) -> Result<(), TypeError> {
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.variables.contains_key(&name) {
            return self.error(
                _span,
                format!("Variable '{}' is already defined in this scope.", name),
            );
        }
        current_scope.mutability.insert(name.clone(), mutability);
        current_scope.variables.insert(name, ty);
        Ok(())
    }

    pub(crate) fn declare_function(
        &mut self,
        name: String,
        type_params: Option<Vec<String>>,
        params: Vec<Type>,
        ret: Option<Type>,
    ) {
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope
            .functions
            .insert(name, (type_params, params, ret));
    }

    pub(crate) fn is_mutable(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if let Some(m) = scope.mutability.get(name) {
                return *m == Mutability::Mutable;
            }
        }
        false
    }

    pub(crate) fn lookup_variable(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.variables.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub(crate) fn lookup_function(
        &self,
        name: &str,
    ) -> Option<(Option<Vec<String>>, Vec<Type>, Option<Type>)> {
        for scope in self.scopes.iter().rev() {
            if let Some(sig) = scope.functions.get(name) {
                return Some(sig.clone());
            }
        }
        None
    }

    /// Typecheck a standalone program (no cross-file imports).
    pub fn check_program(&mut self, program: &Program) -> Result<(), TypeError> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    /// Typecheck a program that can import names from `imports`.
    /// `imports` maps the *path* of a dependency to its `ExportMap`.
    /// This must be called after all dependencies have been typechecked.
    pub fn check_program_with_imports(
        &mut self,
        program: &Program,
        imports: &std::collections::HashMap<String, crate::module::ExportMap>,
    ) -> Result<(), TypeError> {
        // Pre-populate the global scope with everything each `import` statement needs.
        for stmt in &program.statements {
            if let auwla_ast::StmtKind::Import { names, path } = &stmt.node {
                let export_map = imports.get(path.as_str()).ok_or_else(|| TypeError {
                    span: stmt.span.clone(),
                    message: format!("Import error: could not resolve module '{}'", path),
                })?;
                for name in names {
                    if let Some(sig) = export_map.functions.get(name) {
                        self.declare_function(
                            name.clone(),
                            sig.0.clone(),
                            sig.1.clone(),
                            sig.2.clone(),
                        );
                    } else if let Some(ty) = export_map.variables.get(name) {
                        self.declare_variable(
                            stmt.span.clone(),
                            name.clone(),
                            ty.clone(),
                            Mutability::Immutable,
                        )?;
                    } else if let Some(fields) = export_map.structs.get(name) {
                        self.structs.insert(name.clone(), fields.clone());
                    } else if let Some(variants) = export_map.enums.get(name) {
                        self.enums.insert(name.clone(), variants.clone());
                    } else {
                        return self.error(
                            stmt.span.clone(),
                            format!("Import error: '{}' not found in module '{}'", name, path),
                        );
                    }
                }
            }
        }
        // Now typecheck all statements normally
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    pub(crate) fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Custom(name) => {
                if let Some(aliased) = self.type_aliases.get(name) {
                    self.resolve_type(aliased)
                } else {
                    ty.clone()
                }
            }
            _ => ty.clone(),
        }
    }

    pub(crate) fn assert_type_eq(&self, expected: &Type, actual: &Type) -> Result<(), String> {
        let expected = &self.resolve_type(expected);
        let actual = &self.resolve_type(actual);

        // Handle `type?error_type` resolution from `some()` and `none()` with unknowns.
        match (expected, actual) {
            (
                Type::Result {
                    ok_type: e_ok,
                    err_type: e_err,
                },
                Type::Result {
                    ok_type: a_ok,
                    err_type: a_err,
                },
            ) => {
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
            (Type::Optional(e_inner), Type::Optional(a_inner)) => {
                let inner_is_unknown = if let Type::Basic(name) = &**a_inner {
                    name == "unknown"
                } else {
                    false
                };

                if inner_is_unknown || self.assert_type_eq(e_inner, a_inner).is_ok() {
                    return Ok(());
                }
            }
            (Type::Optional(_), Type::Basic(name)) if name == "null" => return Ok(()),

            // Allow `some(value)` to match `Optional` by treating Result<T, unknown> as Optional<T>
            (
                Type::Optional(e_inner),
                Type::Result {
                    ok_type: a_ok,
                    err_type: a_err,
                },
            ) => {
                let err_is_unknown = if let Type::Basic(name) = &**a_err {
                    name == "unknown"
                } else {
                    false
                };

                if err_is_unknown && self.assert_type_eq(e_inner, a_ok).is_ok() {
                    return Ok(());
                }
            }

            _ => {}
        }

        if expected == actual {
            Ok(())
        } else {
            Err(format!(
                "Strict Type mismatch: Expected '{}', found '{}'",
                expected, actual
            ))
        }
    }
}
