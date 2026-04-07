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
    node_types: &HashMap<auwla_ast::Span, auwla_ast::Type>,
    current_origin: auwla_ast::ExtensionOrigin,
) -> (String, String) {
    let mut emitter = JsEmitter::new(
        extensions.clone(),
        enums.clone(),
        type_attributes.clone(),
        node_types.clone(),
        current_origin,
    );
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
    /// Inferred types for nodes from the typechecker
    pub(crate) node_types: HashMap<auwla_ast::Span, auwla_ast::Type>,
    /// Currently active extension receiver type (e.g. "string" for 'extend string')
    pub(crate) current_receiver_type: Option<String>,
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
    /// Origin of the current source file being emitted (Std or User)
    pub(crate) current_origin: auwla_ast::ExtensionOrigin,
    /// Top-level and imported function names visible in this file.
    pub(crate) known_functions: HashSet<String>,
}

impl JsEmitter {
    fn new(
        extensions: HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
        enums: HashSet<String>,
        type_attributes: HashMap<String, Vec<Attribute>>,
        node_types: HashMap<auwla_ast::Span, auwla_ast::Type>,
        current_origin: auwla_ast::ExtensionOrigin,
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
            ext_methods,
            in_extension_method: false,
            extensions,
            is_statement_context: false,
            enums,
            type_attributes,
            current_origin,
            node_types,
            current_receiver_type: None,
            known_functions: HashSet::new(),
        }
    }

    fn collect_known_functions(&self, program: &Program) -> HashSet<String> {
        let mut known = HashSet::new();
        for stmt in &program.statements {
            match &stmt.node {
                auwla_ast::StmtKind::Fn { name, .. } => {
                    known.insert(name.clone());
                }
                auwla_ast::StmtKind::Import { names, .. } => {
                    for n in names {
                        known.insert(n.clone());
                    }
                }
                auwla_ast::StmtKind::Export { stmt: inner } => {
                    if let auwla_ast::StmtKind::Fn { name, .. } = &inner.node {
                        known.insert(name.clone());
                    }
                }
                _ => {}
            }
        }
        known
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
        if let Some(m) = self.find_extension(type_key, method_name) {
            if let Some(attr) = m.attributes.iter().find(|a| a.name == "external") {
                return Some((attr.clone(), m.return_ty.clone()));
            }
        }
        None
    }

    /// Find the ExtensionMethod object for a given type and method name.
    /// Safely handles generic type strings (e.g., "array<number>" -> "array").
    pub(crate) fn find_extension(
        &self,
        type_key: &str,
        method_name: &str,
    ) -> Option<&auwla_ast::ExtensionMethod> {
        // Try the full key (including generics)
        if let Some(methods) = self.extensions.get(type_key) {
            for m in methods {
                if m.name == method_name {
                    return Some(m);
                }
            }
        }
        // Try the base type if the full key failed
        if let Some(idx) = type_key.find('<') {
            let base = &type_key[..idx];
            if let Some(methods) = self.extensions.get(base) {
                for m in methods {
                    if m.name == method_name {
                        return Some(m);
                    }
                }
            }
        }
        None
    }

    /// Recursively infer the type key of an expression for extension method resolution.
    /// Used for chaining (e.g., `self.double().square()` — need to know `double()` returns `number`).
    pub(crate) fn infer_expr_type(&self, expr: &auwla_ast::expr::Expr) -> Option<String> {
        // High-reliability source: Inferred types from the typechecker
        if let Some(ty) = self.node_types.get(&expr.span) {
            return Some(ty.base_key());
        }

        // Minimal fallback for unreachable cases or std-discovery where typechecker isn't run.
        match &expr.node {
            ExprKind::Identifier(name) if name == "self" || name == "__self" => {
                self.current_receiver_type.clone()
            }
            ExprKind::StringLit(_) | ExprKind::Interpolation(_) => Some("string".to_string()),
            ExprKind::NumberLit(_) => Some("number".to_string()),
            ExprKind::BoolLit(_) => Some("bool".to_string()),
            ExprKind::CharLit(_) => Some("char".to_string()),
            ExprKind::StructInit { name, .. } => Some(name.clone()),
            ExprKind::Array(_) => Some("array".to_string()),
            ExprKind::Dict(_) => Some("dict".to_string()),
            ExprKind::Some(_) | ExprKind::None(_) => Some("optional".to_string()),
            _ => None,
        }
    }

    // ──────────────────────────── Program ────────────────────────────

    fn emit_program(&mut self, program: &Program) {
        self.known_functions = self.collect_known_functions(program);
        for stmt in &program.statements {
            self.emit_stmt(stmt);
        }
    }

    // ──────────────────────────── Statements ─────────────────────────
}
