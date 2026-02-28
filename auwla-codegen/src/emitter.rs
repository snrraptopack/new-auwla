use auwla_ast::expr::Expr;

use auwla_ast::Program;
use std::collections::{HashMap, HashSet};

/// Emits JavaScript source code from a type-checked Auwla AST.
/// Returns a tuple of `(main_js_source, extensions_js_source)`.
/// `extensions` maps type_name -> [(method_name, is_static, params, return_ty)].
pub fn emit_js(
    program: &Program,
    extensions: &HashMap<
        String,
        Vec<(
            String,
            bool,
            Vec<(String, auwla_ast::Type)>,
            Option<auwla_ast::Type>,
        )>,
    >,
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
}

impl JsEmitter {
    fn new(
        extensions: HashMap<
            String,
            Vec<(
                String,
                bool,
                Vec<(String, auwla_ast::Type)>,
                Option<auwla_ast::Type>,
            )>,
        >,
    ) -> Self {
        let ext_methods = extensions
            .iter()
            .map(|(ty, methods)| {
                let names: HashSet<String> = methods.iter().map(|(n, _, _, _)| n.clone()).collect();
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
        // Inject custom print formatter for rich CLI debugging (handles T?, Optionals, Structs)
        self.output.push_str("function __print(...args) {\n");
        self.output
            .push_str("  const format = (val, top = false) => {\n");
        self.output
            .push_str("    if (val && typeof val === 'object' && 'ok' in val) {\n");
        self.output
            .push_str("      if (val.ok) return `some(${format(val.value)})`;\n");
        self.output
            .push_str("      if ('value' in val) return `none(${format(val.value)})`;\n");
        self.output.push_str("      return 'none';\n");
        self.output.push_str("    }\n");
        self.output.push_str(
            "    if (Array.isArray(val)) return `[${val.map(v => format(v)).join(', ')}]`;\n",
        );
        self.output
            .push_str("    if (typeof val === 'string' && !top) return `\"${val}\"`;\n");
        self.output
            .push_str("    if (typeof val === 'object' && val !== null) {\n");
        self.output.push_str("      const props = Object.entries(val).map(([k, v]) => `${k}: ${format(v)}`).join(', ');\n");
        self.output.push_str("      return `{ ${props} }`;\n");
        self.output.push_str("    }\n");
        self.output.push_str("    return val;\n");
        self.output.push_str("  };\n");
        self.output
            .push_str("  console.log(...args.map(a => format(a, true)));\n");
        self.output.push_str("}\n\n");

        for stmt in &program.statements {
            self.emit_stmt(stmt);
        }
    }

    // ──────────────────────────── Statements ─────────────────────────
}
