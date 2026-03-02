use auwla_ast::expr::Expr;
use auwla_ast::{ExprKind, Program, Type};
use std::collections::{HashMap, HashSet};

/// Emits JavaScript source code from a type-checked Auwla AST.
/// Returns a tuple of `(main_js_source, extensions_js_source)`.
/// `extensions` maps type_name -> [(method_name, is_static, params, return_ty)].
pub fn emit_js(
    program: &Program,
    extensions: &HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
) -> (String, String) {
    let mut emitter = JsEmitter::new(extensions.clone());
    emitter.emit_program(program);
    (emitter.output, emitter.extensions_output)
}

pub(crate) struct JsEmitter {
    pub(crate) output: String,
    pub(crate) extensions_output: String,
    pub(crate) indent: usize,
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
}

impl JsEmitter {
    fn new(extensions: HashMap<String, Vec<auwla_ast::ExtensionMethod>>) -> Self {
        let ext_methods = extensions
            .iter()
            .map(|(ty, methods)| {
                let names: HashSet<String> = methods.iter().map(|m| m.name.clone()).collect();
                (ty.clone(), names)
            })
            .collect();
        Self {
            output: String::new(),
            extensions_output: String::new(),
            indent: 0,
            temp_counter: 0,
            var_types: HashMap::new(),
            ext_methods,
            in_extension_method: false,
            extensions,
            is_statement_context: false,
        }
    }

    pub(crate) fn fresh_temp(&mut self) -> String {
        let name = format!("__match_{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }

    pub(crate) fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    pub(crate) fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("  ");
        }
    }

    pub(crate) fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    pub(crate) fn write_ext(&mut self, s: &str) {
        self.extensions_output.push_str(s);
    }

    pub(crate) fn write_indent_ext(&mut self) {
        for _ in 0..self.indent {
            self.extensions_output.push_str("  ");
        }
    }

    pub(crate) fn writeln_ext(&mut self, s: &str) {
        self.write_indent_ext();
        self.extensions_output.push_str(s);
        self.extensions_output.push('\n');
    }

    pub(crate) fn emit_expr_to_string(&mut self, expr: &Expr) -> String {
        let old = std::mem::take(&mut self.output);
        self.emit_expr(expr);
        let result = std::mem::take(&mut self.output);
        self.output = old;
        result
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

    // ──────────────────────────── Program ────────────────────────────

    fn emit_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.emit_stmt(stmt);
        }
    }

    // ──────────────────────────── Statements ─────────────────────────
}
