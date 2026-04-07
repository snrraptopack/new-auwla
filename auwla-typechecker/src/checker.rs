use crate::TypeError;
use crate::scope::{Mutability, Scope};
use auwla_ast::{Program, Span, Type};

use std::collections::HashMap;

pub struct Typechecker {
    pub scopes: Vec<Scope>,
    pub(crate) current_return_type: Option<Option<Type>>,
    pub(crate) current_function_name: Option<String>,
    pub structs: HashMap<String, Vec<(String, Type)>>,
    pub struct_type_params: HashMap<String, Vec<String>>,
    pub enums: HashMap<String, Vec<(String, Vec<Type>)>>,
    pub enum_type_params: HashMap<String, Vec<String>>,
    pub type_aliases: HashMap<String, Type>,
    pub type_alias_params: HashMap<String, Vec<String>>,
    /// type_name -> [(type_params, method_name, is_static, params_with_types, return_ty)]
    pub extensions: HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
    /// Meta-information about types (e.g., attributes like @external)
    pub type_attributes: HashMap<String, Vec<auwla_ast::Attribute>>,
    /// Mapping from AST node span to its evaluated Type for LSP services
    pub node_types: HashMap<std::ops::Range<usize>, Type>,
    /// Mapping from name to its declaration token span for go-to-definition
    pub definitions: HashMap<String, Span>,
    /// Origin of the current source being checked
    pub current_origin: auwla_ast::ExtensionOrigin,
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
            struct_type_params: HashMap::new(),
            enums: HashMap::new(),
            enum_type_params: HashMap::new(),
            type_aliases: HashMap::new(),
            type_alias_params: HashMap::new(),
            extensions: HashMap::new(),
            type_attributes: HashMap::new(),
            node_types: HashMap::new(),
            definitions: HashMap::new(),
            current_origin: auwla_ast::ExtensionOrigin::User,
        }
    }

    /// Returns a reference to the extension method registry.
    /// Used by the code generator to identify extension call sites.
    pub fn get_extensions(&self) -> &HashMap<String, Vec<auwla_ast::ExtensionMethod>> {
        &self.extensions
    }

    pub fn get_enum_names(&self) -> std::collections::HashSet<String> {
        self.enums.keys().cloned().collect()
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn type_to_key(&self, ty: &Type) -> String {
        match ty {
            Type::Basic(name) => name.clone(),
            Type::Custom(name) => name.clone(),
            Type::Array(inner) => format!("array<{}>", self.type_to_key(inner)),
            Type::Dict(k, v) => format!("dict<{}, {}>", self.type_to_key(k), self.type_to_key(v)),
            Type::Optional(inner) => format!("{}?", self.type_to_key(inner)),
            Type::Result { ok_type, err_type } => {
                format!(
                    "{}?{}",
                    self.type_to_key(ok_type),
                    self.type_to_key(err_type)
                )
            }
            Type::Generic(name, args) => {
                let parts: Vec<String> = args.iter().map(|a| self.type_to_key(a)).collect();
                format!("{}<{}>", name, parts.join(", "))
            }
            Type::Function(_, _) => "fn".to_string(),
            Type::TypeVar(name) => name.clone(),
            Type::InferenceVar(id) => format!("_{}", id),
            Type::SelfType => "Self".to_string(),
            Type::Wrapper(inner) => format!("wrapper<{}>", self.type_to_key(inner)),
            Type::Tuple(types) => {
                let parts: Vec<String> = types.iter().map(|t| self.type_to_key(t)).collect();
                format!("({})", parts.join(", "))
            }
        }
    }

    pub(crate) fn extend_key(&self, type_name: &str, type_args: &Option<Vec<Type>>) -> String {
        if let Some(args) = type_args {
            let parts: Vec<String> = args.iter().map(|a| self.type_to_key(a)).collect();
            format!("{}<{}>", type_name, parts.join(", "))
        } else {
            type_name.to_string()
        }
    }

    pub(crate) fn has_attribute(
        &self,
        attributes: &[auwla_ast::Attribute],
        name: &str,
        arg: Option<&str>,
    ) -> bool {
        attributes.iter().any(|attr| {
            if attr.name != name {
                return false;
            }
            if let Some(expected_arg) = arg {
                attr.args.iter().any(|a| a == expected_arg)
            } else {
                true
            }
        })
    }

    pub(crate) fn is_namespace(&self, type_name: &str) -> bool {
        if let Some(attrs) = self.type_attributes.get(type_name) {
            self.has_attribute(attrs, "external", Some("namespace"))
        } else {
            false
        }
    }
    #[allow(dead_code)]
    pub(crate) fn is_external_class(&self, type_name: &str) -> bool {
        if let Some(attrs) = self.type_attributes.get(type_name) {
            self.has_attribute(attrs, "external", Some("class"))
        } else {
            false
        }
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
        // Reserved word validation
        const RESERVED_WORDS: &[&str] = &[
            // Keywords
            "let", "var", "fn", "return", "if", "else", "match", "while", "for", "in", "struct",
            "enum", "import", "export", "from", "extend", "type", "break", "continue", "true",
            "false", "some", "none", // Built-in types
            "number", "string", "bool", "char", "void", "array",
        ];
        if RESERVED_WORDS.contains(&name.as_str()) {
            return self.error(
                _span,
                format!(
                    "'{}' is a reserved word and cannot be used as a variable name.",
                    name
                ),
            );
        }

        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.variables.contains_key(&name) {
            return self.error(
                _span,
                format!("Variable '{}' is already defined in this scope.", name),
            );
        }
        current_scope.mutability.insert(name.clone(), mutability);
        current_scope.variables.insert(name.clone(), ty);
        self.definitions.insert(name, _span);
        Ok(())
    }

    pub(crate) fn declare_function(
        &mut self,
        span: Span,
        name: String,
        signature: (Option<Vec<String>>, Vec<(Type, bool)>, Option<Type>),
    ) {
        self.scopes
            .last_mut()
            .unwrap()
            .functions
            .insert(name.clone(), signature);
        self.definitions.insert(name, span);
    }

    pub(crate) fn register_function(
        &mut self,
        _name: String,
        type_params: Option<Vec<String>>,
        param_sigs: Vec<(Type, bool)>,
        return_ty_gen: Option<Type>,
    ) -> (Option<Vec<String>>, Vec<(Type, bool)>, Option<Type>) {
        (type_params, param_sigs, return_ty_gen)
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
    ) -> Option<(Option<Vec<String>>, Vec<(Type, bool)>, Option<Type>)> {
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
                            stmt.span.clone(),
                            name.clone(),
                            sig.clone(),
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
                    } else if let Some(attrs) = export_map.type_attributes.get(name) {
                        self.type_attributes.insert(name.clone(), attrs.clone());
                    } else {
                        return self.error(
                            stmt.span.clone(),
                            format!("Import error: '{}' not found in module '{}'", name, path),
                        );
                    }
                }
                // Automatically import ALL extensions from the module
                // Because extensions are 'global' survivors in Auwla
                for (type_key, methods) in &export_map.extensions {
                    self.extensions
                        .entry(type_key.clone())
                        .or_default()
                        .extend(methods.clone());
                }
            }
        }
        // Actually, we've already injected all extensions globally in the CLI,
        // so we don't need to re-import them here. The CLI approach is more "magical".
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
            Type::Generic(name, args) => {
                if let Some(aliased) = self.type_aliases.get(name) {
                    if let Some(params) = self.type_alias_params.get(name) {
                        // Perform substitution: map params[i] -> args[i]
                        let mut substituted = aliased.clone();
                        for (i, param_name) in params.iter().enumerate() {
                            if let Some(arg) = args.get(i) {
                                substituted = self.substitute_type_var(&substituted, param_name, arg);
                            }
                        }
                        return self.resolve_type(&substituted);
                    }
                    // Non-generic alias using Generic syntax (unlikely but possible)
                    self.resolve_type(aliased)
                } else {
                    ty.clone()
                }
            }
            _ => ty.clone(),
        }
    }

    pub(crate) fn genericize_type(&self, ty: &Type, type_params: &[String]) -> Type {
        match ty {
            Type::Custom(name) if type_params.contains(name) => Type::TypeVar(name.clone()),
            Type::Array(inner) => Type::Array(Box::new(self.genericize_type(inner, type_params))),
            Type::Dict(k, v) => Type::Dict(
                Box::new(self.genericize_type(k, type_params)),
                Box::new(self.genericize_type(v, type_params)),
            ),
            Type::Optional(inner) => {
                Type::Optional(Box::new(self.genericize_type(inner, type_params)))
            }
            Type::Result { ok_type, err_type } => Type::Result {
                ok_type: Box::new(self.genericize_type(ok_type, type_params)),
                err_type: Box::new(self.genericize_type(err_type, type_params)),
            },
            Type::Generic(name, args) => {
                let gen_args = args
                    .iter()
                    .map(|a| self.genericize_type(a, type_params))
                    .collect();
                Type::Generic(name.clone(), gen_args)
            }
            Type::Function(params, ret) => {
                let gen_params = params
                    .iter()
                    .map(|(p, is_v)| (self.genericize_type(p, type_params), *is_v))
                    .collect();
                Type::Function(gen_params, Box::new(self.genericize_type(ret, type_params)))
            }
            Type::Tuple(types) => {
                let gen_types = types
                    .iter()
                    .map(|t| self.genericize_type(t, type_params))
                    .collect();
                Type::Tuple(gen_types)
            }
            _ => ty.clone(),
        }
    }

    /// Resolve `Self` within a type to the given concrete type.
    pub(crate) fn resolve_self_type(&self, ty: &Type, self_ty: &Type) -> Type {
        self.substitute_type_var(ty, "Self", self_ty)
    }

    pub(crate) fn substitute_type_var(&self, ty: &Type, var_name: &str, replacement: &Type) -> Type {
        match ty {
            Type::Custom(name) if name == var_name => replacement.clone(),
            Type::TypeVar(name) if name == var_name => replacement.clone(),
            Type::Array(inner) => {
                Type::Array(Box::new(self.substitute_type_var(inner, var_name, replacement)))
            }
            Type::Dict(k, v) => Type::Dict(
                Box::new(self.substitute_type_var(k, var_name, replacement)),
                Box::new(self.substitute_type_var(v, var_name, replacement)),
            ),
            Type::Optional(inner) => {
                Type::Optional(Box::new(self.substitute_type_var(inner, var_name, replacement)))
            }
            Type::Result { ok_type, err_type } => Type::Result {
                ok_type: Box::new(self.substitute_type_var(ok_type, var_name, replacement)),
                err_type: Box::new(self.substitute_type_var(err_type, var_name, replacement)),
            },
            Type::Generic(name, args) => {
                let sub_args = args
                    .iter()
                    .map(|a| self.substitute_type_var(a, var_name, replacement))
                    .collect();
                Type::Generic(name.clone(), sub_args)
            }
            Type::Function(params, ret) => {
                let sub_params = params
                    .iter()
                    .map(|(p, is_v)| (self.substitute_type_var(p, var_name, replacement), *is_v))
                    .collect();
                Type::Function(
                    sub_params,
                    Box::new(self.substitute_type_var(ret, var_name, replacement)),
                )
            }
            Type::Tuple(types) => {
                let sub_types = types
                    .iter()
                    .map(|t| self.substitute_type_var(t, var_name, replacement))
                    .collect();
                Type::Tuple(sub_types)
            }
            Type::Wrapper(inner) => {
                Type::Wrapper(Box::new(self.substitute_type_var(inner, var_name, replacement)))
            }
            Type::SelfType if var_name == "Self" => replacement.clone(),
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
            
            // Wrapper (some(val)) compatibility
            (Type::Wrapper(w_inner), Type::Optional(o_inner))
            | (Type::Optional(o_inner), Type::Wrapper(w_inner)) => {
                return self.assert_type_eq(o_inner, w_inner);
            }
            (Type::Wrapper(w_inner), Type::Result { ok_type, .. })
            | (Type::Result { ok_type, .. }, Type::Wrapper(w_inner)) => {
                return self.assert_type_eq(w_inner, ok_type);
            }
            (Type::Wrapper(w1), Type::Wrapper(w2)) => {
                return self.assert_type_eq(w1, w2);
            }
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                if p1.len() != p2.len() {
                    return Err(format!("Arg count mismatch: {} vs {}", p1.len(), p2.len()));
                }
                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.assert_type_eq(&a.0, &b.0)?;
                    if a.1 != b.1 {
                        return Err("Vararg mismatch".to_string());
                    }
                }
                self.assert_type_eq(r1, r2)?;
                return Ok(());
            }
            (Type::Dict(e_k, e_v), Type::Dict(a_k, a_v)) => {
                if self.assert_type_eq(e_k, a_k).is_ok() {
                    if self.assert_type_eq(e_v, a_v).is_ok() {
                        return Ok(());
                    }
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

    pub(crate) fn contains_unknown(&self, ty: &Type) -> bool {
        match ty {
            Type::Basic(name) => name == "unknown",
            Type::Array(inner) => self.contains_unknown(inner),
            Type::Dict(k, v) => self.contains_unknown(k) || self.contains_unknown(v),
            Type::Optional(inner) => self.contains_unknown(inner),
            Type::Result { ok_type, err_type } => {
                self.contains_unknown(ok_type) || self.contains_unknown(err_type)
            }
            Type::Generic(_, args) => args.iter().any(|a| self.contains_unknown(a)),
            Type::Function(params, ret) => {
                params.iter().any(|(p, _)| self.contains_unknown(p)) || self.contains_unknown(ret)
            }
            Type::Tuple(types) => types.iter().any(|t| self.contains_unknown(t)),
            _ => false,
        }
    }
}
