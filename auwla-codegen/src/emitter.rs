use auwla_ast::expr::Expr;

use auwla_ast::Program;
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

    // ──────────────────────────── Program ────────────────────────────

    fn emit_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.emit_stmt(stmt);
        }
    }

    // ──────────────────────────── Statements ─────────────────────────
}
