use crate::writer::CodeWriter;
use auwla_ast::expr::Expr;
use auwla_ast::{Attribute, ExprKind, Program, Type};
use std::collections::{HashMap, HashSet};

/// Emits JavaScript source code from a type-checked Auwla AST.
/// Returns a tuple of `(main_js_source, extensions_js_source)`.
/// `extensions` maps type_name -> [(method_name, is_static, params, return_ty)].
pub fn emit_js(
    program: &Program,
    extensions: &HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
    enums: &HashSet<String>,
    type_attributes: &HashMap<String, Vec<Attribute>>,
) -> (String, String) {
    let mut emitter = JsEmitter::new(extensions.clone(), enums.clone(), type_attributes.clone());
    emitter.emit_program(program);
    (emitter.out.into_string(), emitter.ext.into_string())
}

pub(crate) struct JsEmitter {
    /// Main output buffer for JS source code.
    pub(crate) out: CodeWriter,
    /// Extensions output buffer (separate file).
    pub(crate) ext: CodeWriter,
    /// Counter for generating unique temp variable names (e.g. __match_0, __match_1)
    pub(crate) temp_counter: usize,
    /// variable name -> type name, for resolving extension call sites
    pub(crate) var_types: HashMap<String, String>,
    /// type_name -> set of extension method names (for fast lookup)
    pub(crate) ext_methods: HashMap<String, HashSet<String>>,
    /// Flag to trigger `self` -> `__self` rewriting
    pub(crate) in_extension_method: bool,
    /// Full extension signatures for attribute lookup
    pub(crate) extensions: HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
    /// Flag to prevent `return` injection in standalone blocks/matches
    pub(crate) is_statement_context: bool,
    /// Known enums (to distinguish static methods from variants)
    pub(crate) enums: HashSet<String>,
    /// Type-level attributes (e.g., @external("namespace"), @external("class"))
    #[allow(dead_code)]
    pub(crate) type_attributes: HashMap<String, Vec<Attribute>>,
}

impl JsEmitter {
    fn new(
        extensions: HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
        enums: HashSet<String>,
        type_attributes: HashMap<String, Vec<Attribute>>,
    ) -> Self {
        let ext_methods = extensions
            .iter()
            .map(|(ty, methods)| {
                let names: HashSet<String> = methods.iter().map(|m| m.name.clone()).collect();
                (ty.clone(), names)
            })
            .collect();
        Self {
            out: CodeWriter::new(),
            ext: CodeWriter::new(),
            temp_counter: 0,
            var_types: HashMap::new(),
            ext_methods,
            in_extension_method: false,
            extensions,
            is_statement_context: false,
            enums,
            type_attributes,
        }
    }

    pub(crate) fn fresh_temp(&mut self) -> String {
        let name = format!("__match_{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }

    // ── Convenience delegations to `self.out` ────────────────────
    // These keep call-site code concise for the main output buffer.

    pub(crate) fn write(&mut self, s: &str) {
        self.out.write(s);
    }

    pub(crate) fn write_indent(&mut self) {
        self.out.write_indent();
    }

    pub(crate) fn writeln(&mut self, s: &str) {
        self.out.writeln(s);
    }

    // ── Convenience delegations to `self.ext` ────────────────────

    pub(crate) fn write_ext(&mut self, s: &str) {
        self.ext.write(s);
    }

    pub(crate) fn write_indent_ext(&mut self) {
        self.ext.write_indent();
    }

    pub(crate) fn writeln_ext(&mut self, s: &str) {
        self.ext.writeln(s);
    }

    // ── Shared indent helpers (synchronized across both buffers) ──

    /// Increase indent on both main and ext buffers.
    #[allow(dead_code)]
    pub(crate) fn indent_both(&mut self) {
        self.out.indent();
        self.ext.indent();
    }

    /// Decrease indent on both main and ext buffers.
    #[allow(dead_code)]
    pub(crate) fn dedent_both(&mut self) {
        self.out.dedent();
        self.ext.dedent();
    }

    // ── Utilities ────────────────────────────────────────────────

    pub(crate) fn emit_expr_to_string(&mut self, expr: &Expr) -> String {
        self.out.capture(|w| {
            // We need to temporarly work with just the writer, but emit_expr
            // needs &mut self. So we swap the writer out, call emit_expr, swap back.
            let _ = w; // unused — we do the swap trick at the JsEmitter level instead
        });
        // Use the traditional swap approach since emit_expr needs &mut self:
        let old = std::mem::replace(&mut self.out, CodeWriter::new());
        self.emit_expr(expr);
        let result_writer = std::mem::replace(&mut self.out, old);
        result_writer.into_string()
    }

    pub(crate) fn type_to_key(&self, ty: &Type) -> String {
        match ty {
            Type::Basic(name) => name.clone(),
            Type::Custom(name) => name.clone(),
            Type::Array(inner) => format!("array<{}>", self.type_to_key(inner)),
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

    pub(crate) fn type_key_ident(&self, key: &str) -> String {
        let mut result = String::new();
        let mut last_was_underscore = false;

        for c in key.chars() {
            if c.is_ascii_alphanumeric() {
                result.push(c);
                last_was_underscore = false;
            } else {
                if !last_was_underscore && !result.is_empty() {
                    result.push('_');
                    last_was_underscore = true;
                }
            }
        }

        // Trim trailing underscore
        if result.ends_with('_') {
            result.pop();
        }
        result
    }

    #[allow(dead_code)]
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

    /// Check if a type is declared as `@external("namespace")`.
    #[allow(dead_code)]
    pub(crate) fn is_namespace(&self, type_name: &str) -> bool {
        self.type_attributes
            .get(type_name)
            .map(|attrs| self.has_attribute(attrs, "external", Some("namespace")))
            .unwrap_or(false)
    }

    /// Check if a type is declared as `@external("class")`.
    #[allow(dead_code)]
    pub(crate) fn is_external_class(&self, type_name: &str) -> bool {
        self.type_attributes
            .get(type_name)
            .map(|attrs| self.has_attribute(attrs, "external", Some("class")))
            .unwrap_or(false)
    }

    /// Find the @external attribute on an extension method for a given type+method.
    /// Returns the attribute and the method's return type (for Optional wrapping).
    pub(crate) fn find_external_attr(
        &self,
        type_key: &str,
        method_name: &str,
    ) -> Option<(auwla_ast::Attribute, Option<Type>)> {
        // Check the exact key first, then try the base type name
        let keys_to_try = [type_key.to_string()];
        for key in &keys_to_try {
            if let Some(methods) = self.extensions.get(key) {
                for m in methods {
                    if m.name == method_name {
                        if let Some(attr) = m.attributes.iter().find(|a| a.name == "external") {
                            return Some((attr.clone(), m.return_ty.clone()));
                        }
                    }
                }
            }
        }
        None
    }

    pub(crate) fn array_literal_type_key(&self, elems: &[Expr]) -> Option<String> {
        if elems.is_empty() {
            return None;
        }
        let mut kind: Option<String> = None;
        for e in elems {
            let k = match &e.node {
                ExprKind::NumberLit(_) => "number".to_string(),
                ExprKind::StringLit(_) => "string".to_string(),
                ExprKind::BoolLit(_) => "bool".to_string(),
                ExprKind::CharLit(_) => "char".to_string(),
                ExprKind::StructInit { name, .. } => name.clone(),
                _ => return None,
            };
            if let Some(prev) = kind.as_ref() {
                if prev != &k {
                    return None;
                }
            } else {
                kind = Some(k);
            }
        }
        kind.map(|k| format!("array<{}>", k))
    }

    /// Infer a type key string from an expression's AST shape.
    ///
    /// Used to register variable types in `var_types` so extension method
    /// calls can be resolved. Centralizes the repeated initializer-sniffing
    /// logic that was duplicated across Let/Var handlers.
    pub(crate) fn infer_type_key_from_expr(&self, expr: &auwla_ast::expr::Expr) -> Option<String> {
        match &expr.node {
            ExprKind::Array(elems) => Some(
                self.array_literal_type_key(elems)
                    .unwrap_or_else(|| "array".to_string()),
            ),
            ExprKind::StringLit(_) => Some("string".to_string()),
            ExprKind::NumberLit(_) => Some("number".to_string()),
            ExprKind::BoolLit(_) => Some("bool".to_string()),
            ExprKind::CharLit(_) => Some("char".to_string()),
            ExprKind::StructInit { name, .. } => Some(name.clone()),
            ExprKind::Range { .. } => Some("array<number>".to_string()),
            _ => None,
        }
    }

    /// Register a variable's type from an explicit annotation or by
    /// inferring from the initializer expression.
    pub(crate) fn register_var_type(
        &mut self,
        name: &str,
        ty: &Option<Type>,
        initializer: &auwla_ast::expr::Expr,
    ) {
        if let Some(t) = ty {
            self.var_types.insert(name.to_string(), self.type_to_key(t));
        } else if let Some(key) = self.infer_type_key_from_expr(initializer) {
            self.var_types.insert(name.to_string(), key);
        }
    }

    // ──────────────────────────── Program ────────────────────────────

    fn emit_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.emit_stmt(stmt);
        }
    }

    // ──────────────────────────── Statements ─────────────────────────
}
