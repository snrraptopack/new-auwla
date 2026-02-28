use auwla_ast::expr::{BinaryOp, Expr, MatchArm, UnaryOp};
use auwla_ast::stmt::Stmt;
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

struct JsEmitter {
    output: String,
    extensions_output: String,
    indent: usize,
    /// Counter for generating unique temp variable names (e.g. __match_0, __match_1)
    temp_counter: usize,
    /// variable name -> type name, for resolving extension call sites
    var_types: HashMap<String, String>,
    /// type_name -> set of extension method names (for fast lookup)
    ext_methods: HashMap<String, HashSet<String>>,
    /// Flag to trigger `self` -> `__self` rewriting
    in_extension_method: bool,
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

    fn fresh_temp(&mut self) -> String {
        let name = format!("__match_{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("  ");
        }
    }

    fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn write_ext(&mut self, s: &str) {
        self.extensions_output.push_str(s);
    }

    fn write_indent_ext(&mut self) {
        for _ in 0..self.indent {
            self.extensions_output.push_str("  ");
        }
    }

    fn writeln_ext(&mut self, s: &str) {
        self.write_indent_ext();
        self.extensions_output.push_str(s);
        self.extensions_output.push('\n');
    }

    fn emit_expr_to_string(&mut self, expr: &Expr) -> String {
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

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                name,
                ty,
                initializer,
                ..
            } => {
                if let Some(t) = ty {
                    let type_name = match t {
                        auwla_ast::Type::Custom(n) => n.clone(),
                        auwla_ast::Type::Basic(n) => n.clone(),
                        _ => format!("{:?}", t),
                    };
                    self.var_types.insert(name.clone(), type_name);
                }
                if let Expr::Match { expr, arms } = initializer {
                    self.emit_match_assign("const", name, expr, arms);
                } else if let Expr::Try { expr, error_expr } = initializer {
                    self.emit_try_assign("const", name, expr, error_expr);
                } else {
                    self.write_indent();
                    self.write(&format!("const {} = ", name));
                    self.emit_expr(initializer);
                    self.write(";\n");
                }
            }
            Stmt::Var {
                name,
                ty,
                initializer,
                ..
            } => {
                if let Some(t) = ty {
                    let type_name = match t {
                        auwla_ast::Type::Custom(n) => n.clone(),
                        auwla_ast::Type::Basic(n) => n.clone(),
                        _ => format!("{:?}", t),
                    };
                    self.var_types.insert(name.clone(), type_name);
                }
                if let Expr::Match { expr, arms } = initializer {
                    self.emit_match_assign("let", name, expr, arms);
                } else if let Expr::Try { expr, error_expr } = initializer {
                    self.emit_try_assign("let", name, expr, error_expr);
                } else {
                    self.write_indent();
                    self.write(&format!("let {} = ", name));
                    self.emit_expr(initializer);
                    self.write(";\n");
                }
            }
            Stmt::DestructureLet {
                bindings,
                initializer,
            } => {
                self.write_indent();
                self.write("const { ");
                for (i, b) in bindings.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(b);
                }
                self.write(" } = ");
                self.emit_expr(initializer);
                self.write(";\n");
            }
            Stmt::Assign { target, value } => {
                let target_str = self.emit_expr_to_string(target);
                if let Expr::Match { expr, arms } = value {
                    self.emit_match_assign("", &target_str, expr, arms);
                } else if let Expr::Try { expr, error_expr } = value {
                    self.emit_try_assign("", &target_str, expr, error_expr);
                } else {
                    self.write_indent();
                    self.write(&format!("{} = ", target_str));
                    self.emit_expr(value);
                    self.write(";\n");
                }
            }
            Stmt::Fn {
                name, params, body, ..
            } => {
                for (param_name, ty) in params {
                    let type_name = match ty {
                        auwla_ast::Type::Custom(n) => n.clone(),
                        auwla_ast::Type::Basic(n) => n.clone(),
                        _ => format!("{:?}", ty),
                    };
                    self.var_types.insert(param_name.clone(), type_name);
                }
                self.write_indent();
                let param_names: Vec<&str> = params.iter().map(|(n, _)| n.as_str()).collect();
                self.write(&format!(
                    "function {}({}) {{\n",
                    name,
                    param_names.join(", ")
                ));
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::Return(expr_opt) => {
                self.write_indent();
                if let Some(expr) = expr_opt {
                    self.write("return ");
                    self.emit_expr(expr);
                    self.write(";\n");
                } else {
                    self.write("return;\n");
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.write_indent();
                self.write("if (");
                self.emit_expr(condition);
                self.write(") {\n");
                self.indent += 1;
                for s in then_branch {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                if let Some(els) = else_branch {
                    self.writeln("} else {");
                    self.indent += 1;
                    for s in els {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                }
                self.writeln("}");
            }
            Stmt::While { condition, body } => {
                self.write_indent();
                self.write("while (");
                self.emit_expr(condition);
                self.write(") {\n");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::For {
                binding,
                iterable,
                body,
            } => {
                self.write_indent();
                self.write(&format!("for (const {} of ", binding));
                self.emit_expr(iterable);
                self.write(") {\n");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::Expr(expr) => {
                // Standalone match expression (used as statement)
                if let Expr::Match {
                    expr: matched,
                    arms,
                } = expr
                {
                    self.emit_match_standalone(matched, arms);
                } else if let Expr::Try {
                    expr: tried,
                    error_expr,
                } = expr
                {
                    self.emit_try_standalone(tried, error_expr);
                } else {
                    self.write_indent();
                    self.emit_expr(expr);
                    self.write(";\n");
                }
            }
            Stmt::StructDecl { .. } | Stmt::EnumDecl { .. } => {
                // Struct/Enum declarations vanish in JS, they are purely for compile-time typechecking
                // We emit nothing to keep it zero-cost.
            }
            Stmt::Import { names, path } => {
                // Rewrite Auwla relative path to .js extension
                let js_path = if path.ends_with(".aw") {
                    format!("{}.js", &path[..path.len() - 3])
                } else {
                    format!("{}.js", path)
                };
                self.write_indent();
                self.write("import { ");
                for (i, name) in names.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(name);
                }
                self.write(&format!(" }} from '{}';\n", js_path));
            }
            Stmt::Export { stmt: inner } => {
                match inner.as_ref() {
                    Stmt::Fn { name: _, .. } => {
                        // Temporarily emit the fn, then prefix with `export `
                        let saved_len = self.output.len();
                        self.emit_stmt(inner);
                        // Find where `function` keyword starts and insert `export `
                        let emitted = &self.output[saved_len..];
                        let new_emitted = emitted.replacen("function ", "export function ", 1);
                        self.output.truncate(saved_len);
                        self.output.push_str(&new_emitted);
                    }
                    Stmt::Let { .. } | Stmt::Var { .. } => {
                        let saved_len = self.output.len();
                        self.emit_stmt(inner);
                        let emitted = &self.output[saved_len..];
                        // prefix `const ` or `let ` with `export `
                        let new_emitted = if emitted.trim_start().starts_with("const ") {
                            emitted.replacen("const ", "export const ", 1)
                        } else {
                            emitted.replacen("let ", "export let ", 1)
                        };
                        self.output.truncate(saved_len);
                        self.output.push_str(&new_emitted);
                    }
                    Stmt::StructDecl { .. } | Stmt::EnumDecl { .. } => {
                        // types vanish in JS output — no-op
                    }
                    _ => {
                        // For anything else (e.g., exported block expressions), emit as-is
                        self.emit_stmt(inner);
                    }
                }
            }
            Stmt::Extend { type_name, methods } => {
                // Emit each method as a standalone function: __ext_TypeName_methodName
                for method in methods {
                    // Register method parameters in var_types
                    for (param_name, ty_opt) in &method.params {
                        if param_name == "self" {
                            self.var_types
                                .insert("__self".to_string(), type_name.clone());
                        } else if let Some(ty) = ty_opt {
                            let t_name = match ty {
                                auwla_ast::Type::Custom(n) => n.clone(),
                                auwla_ast::Type::Basic(n) => n.clone(),
                                _ => format!("{:?}", ty),
                            };
                            self.var_types.insert(param_name.clone(), t_name);
                        }
                    }

                    if method.is_static {
                        // Static methods don't have a receiver — emit as plain function
                        self.write_indent_ext();
                        self.write_ext(&format!(
                            "export function __ext_{}_{}(",
                            type_name, method.name
                        ));
                        let params: Vec<_> = method.params.iter().collect();
                        for (i, (pname, _)) in params.iter().enumerate() {
                            if i > 0 {
                                self.write_ext(", ");
                            }
                            self.write_ext(pname);
                        }
                        self.write_ext(") {\n");
                    } else {
                        // Instance methods: first param is `self` → rename to `__self`
                        self.write_indent_ext();
                        self.write_ext(&format!(
                            "export function __ext_{}_{}(__self",
                            type_name, method.name
                        ));
                        for (pname, _) in method.params.iter().filter(|(n, _)| n != "self") {
                            self.write_ext(", ");
                            self.write_ext(pname);
                        }
                        self.write_ext(") {\n");
                    }
                    self.indent += 1;
                    // Emit body, rewriting `self` identifiers to `__self`
                    let old_output = std::mem::take(&mut self.output);

                    for s in &method.body {
                        self.emit_stmt_with_self_rename(s);
                    }

                    let body_output = std::mem::take(&mut self.output);
                    self.output = old_output;
                    self.write_ext(&body_output);

                    self.indent -= 1;
                    self.writeln_ext("}\n");
                }
            }
        }
    }

    /// Emit a statement, replacing identifier `self` with `__self` for method bodies.
    fn emit_stmt_with_self_rename(&mut self, stmt: &Stmt) {
        let old = self.in_extension_method;
        self.in_extension_method = true;
        self.emit_stmt(stmt);
        self.in_extension_method = old;
    }

    // ──────────────────────────── Match helpers ──────────────────────

    /// Emit: `const/let name = match expr { some(val) => ... none(err) => ... };`
    /// Becomes:
    /// ```js
    /// const __match_N = <expr>;
    /// let target;   // or const target, depending on decl_kw
    /// if (__match_N.ok) { const val = __match_N.value; ... target = <result>; }
    /// else { const err = __match_N.value; ... target = <result>; }
    /// ```
    fn emit_match_assign(
        &mut self,
        decl_kw: &str,
        target: &str,
        matched_expr: &Expr,
        arms: &Vec<MatchArm>,
    ) {
        let temp = self.fresh_temp();

        // const __match_N = <matched_expr>;
        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(matched_expr);
        self.write(";\n");

        if !decl_kw.is_empty() {
            self.writeln(&format!("let {};", target));
        }

        for (i, arm) in arms.iter().enumerate() {
            self.write_indent();
            if i > 0 {
                self.write("else ");
            }

            self.emit_pattern_condition(&temp, arm);

            self.write(" {\n");
            self.indent += 1;
            self.emit_arm_body(&temp, target, arm);
            self.indent -= 1;
            self.writeln("}");
        }
    }

    fn emit_pattern_condition(&mut self, temp: &str, arm: &MatchArm) {
        self.write("if (");

        // 1. Check shape
        self.emit_pattern_shape(temp, &arm.pattern);

        // 2. Check guard condition in IIFE
        if let Some(ref guard) = arm.guard {
            self.write(" && (() => {\n");
            self.indent += 1;
            self.emit_bound_variables(temp, &arm.pattern);
            self.write_indent();
            self.write("return ");
            self.emit_expr(guard);
            self.write(";\n");
            self.indent -= 1;
            self.write_indent();
            self.write("})()");
        }

        self.write(")");
    }

    fn emit_pattern_shape(&mut self, temp: &str, pattern: &auwla_ast::Pattern) {
        match pattern {
            auwla_ast::Pattern::Wildcard | auwla_ast::Pattern::Variable(_) => {
                self.write("true");
            }
            auwla_ast::Pattern::Literal(expr) => {
                self.write(&format!("{} === ", temp));
                self.emit_expr(expr);
            }
            auwla_ast::Pattern::Variant { name, bindings: _ } => {
                if name == "some" {
                    self.write(&format!("{}.ok", temp));
                } else if name == "none" {
                    self.write(&format!("!{}.ok", temp));
                } else {
                    self.write(&format!("{}.$variant === \"{}\"", temp, name));
                }
            }
            auwla_ast::Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let op = if *inclusive { "<=" } else { "<" };
                self.write(&format!("({} >= ", temp));
                self.emit_expr(start);
                self.write(&format!(" && {} {} ", temp, op));
                self.emit_expr(end);
                self.write(")");
            }
            auwla_ast::Pattern::Or(patterns) => {
                self.write("(");
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.write(" || ");
                    }
                    self.emit_pattern_shape(temp, p);
                }
                self.write(")");
            }
            auwla_ast::Pattern::Struct(_name, fields) => {
                self.write("(");
                for (i, (fname, sub_pattern_opt)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(" && ");
                    }
                    if let Some(sub_pattern) = sub_pattern_opt {
                        let inner_temp = format!("{}.{}", temp, fname);
                        self.emit_pattern_shape(&inner_temp, sub_pattern);
                    } else {
                        self.write(&format!("{}.{} !== undefined", temp, fname));
                    }
                }
                if fields.is_empty() {
                    self.write("true");
                }
                self.write(")");
            }
        }
    }

    /// Emit a standalone match (not assigned to anything).
    fn emit_match_standalone(&mut self, matched_expr: &Expr, arms: &Vec<MatchArm>) {
        let temp = self.fresh_temp();

        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(matched_expr);
        self.write(";\n");

        for (i, arm) in arms.iter().enumerate() {
            self.write_indent();
            if i > 0 {
                self.write("else ");
            }

            self.emit_pattern_condition(&temp, arm);

            self.write(" {\n");
            self.indent += 1;
            self.emit_arm_body_standalone(&temp, arm);
            self.indent -= 1;
            self.writeln("}");
        }
    }

    /// Emit: `const/let name = try expr(error_expr);`
    fn emit_try_assign(
        &mut self,
        decl_kw: &str,
        target: &str,
        expr: &Expr,
        error_expr: &Option<Box<Expr>>,
    ) {
        let temp = self.fresh_temp();
        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(expr);
        self.write(";\n");

        self.write_indent();
        if let Some(err) = error_expr {
            self.write(&format!("if (!{}.ok) return {{ ok: false, value: ", temp));
            self.emit_expr(err);
            self.write(" };\n");
        } else {
            self.write(&format!("if (!{}.ok) return {};\n", temp, temp));
        }

        self.write_indent();
        if !decl_kw.is_empty() {
            self.write(&format!("{} {} = {}.value;\n", decl_kw, target, temp));
        } else {
            self.write(&format!("{} = {}.value;\n", target, temp));
        }
    }

    /// Emit a standalone try.
    fn emit_try_standalone(&mut self, expr: &Expr, error_expr: &Option<Box<Expr>>) {
        let temp = self.fresh_temp();
        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(expr);
        self.write(";\n");

        self.write_indent();
        if let Some(err) = error_expr {
            self.write(&format!("if (!{}.ok) return {{ ok: false, value: ", temp));
            self.emit_expr(err);
            self.write(" };\n");
        } else {
            self.write(&format!("if (!{}.ok) return {};\n", temp, temp));
        }
    }

    fn emit_bound_variables(&mut self, temp: &str, pattern: &auwla_ast::Pattern) {
        match pattern {
            auwla_ast::Pattern::Variable(name) => {
                self.write_indent();
                self.write(&format!("const {} = {};\n", name, temp));
            }
            auwla_ast::Pattern::Variant { name, bindings } => {
                if name == "some" || name == "none" {
                    if let Some(binding) = bindings.first() {
                        self.write_indent();
                        self.write(&format!("const {} = {}.value;\n", binding, temp));
                    }
                } else {
                    for (i, binding) in bindings.iter().enumerate() {
                        self.write_indent();
                        self.write(&format!("const {} = {}.$data[{}];\n", binding, temp, i));
                    }
                }
            }
            auwla_ast::Pattern::Or(patterns) => {
                if let Some(first) = patterns.first() {
                    self.emit_bound_variables(temp, first);
                }
            }
            auwla_ast::Pattern::Struct(_name, fields) => {
                for (fname, sub_pattern_opt) in fields {
                    if let Some(sub_pattern) = sub_pattern_opt {
                        let inner_temp = format!("{}.{}", temp, fname);
                        self.emit_bound_variables(&inner_temp, sub_pattern);
                    } else {
                        // shorthand binding e.g `{ role }`
                        self.write_indent();
                        self.write(&format!("const {} = {}.{};\n", fname, temp, fname));
                    }
                }
            }
            _ => {}
        }
    }

    /// Emit arm body for an assigned match: bind the inner value, run stmts, assign result to target.
    fn emit_arm_body(&mut self, temp: &str, target: &str, arm: &MatchArm) {
        self.emit_bound_variables(temp, &arm.pattern);

        for s in &arm.stmts {
            self.emit_stmt(s);
        }
        if let Some(result) = &arm.result {
            self.write_indent();
            self.write(&format!("{} = ", target));
            self.emit_expr(result);
            self.write(";\n");
        }
    }

    /// Emit arm body for a standalone match.
    fn emit_arm_body_standalone(&mut self, temp: &str, arm: &MatchArm) {
        self.emit_bound_variables(temp, &arm.pattern);

        for s in &arm.stmts {
            self.emit_stmt(s);
        }
        if let Some(result) = &arm.result {
            self.write_indent();
            self.emit_expr(result);
            self.write(";\n");
        }
    }

    // ──────────────────────────── Expressions ────────────────────────

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Void => self.write("undefined"),
            Expr::StringLit(s) => self.write(&format!("\"{}\"", s)),
            Expr::NumberLit(n) => {
                // Emit integers without decimal point
                if *n == (*n as i64) as f64 {
                    self.write(&format!("{}", *n as i64));
                } else {
                    self.write(&format!("{}", n));
                }
            }
            Expr::BoolLit(b) => self.write(if *b { "true" } else { "false" }),
            Expr::CharLit(c) => self.write(&format!("\"{}\"", c)),
            Expr::Identifier(name) => {
                if self.in_extension_method && name == "self" {
                    self.write("__self");
                } else {
                    self.write(name);
                }
            }
            Expr::Binary { op, left, right } => {
                self.write("(");
                self.emit_expr(left);
                let op_str = match op {
                    BinaryOp::Add => " + ",
                    BinaryOp::Sub => " - ",
                    BinaryOp::Mul => " * ",
                    BinaryOp::Div => " / ",
                    BinaryOp::Eq => " === ",
                    BinaryOp::Neq => " !== ",
                    BinaryOp::Lt => " < ",
                    BinaryOp::Gt => " > ",
                    BinaryOp::Lte => " <= ",
                    BinaryOp::Gte => " >= ",
                    BinaryOp::And => " && ",
                    BinaryOp::Or => " || ",
                };
                self.write(op_str);
                self.emit_expr(right);
                self.write(")");
            }
            Expr::Unary { op, expr: inner } => {
                match op {
                    UnaryOp::Not => self.write("!"),
                    UnaryOp::Neg => self.write("-"),
                }
                self.emit_expr(inner);
            }
            Expr::Some(inner) => {
                self.write("({ ok: true, value: ");
                self.emit_expr(inner);
                self.write(" })");
            }
            Expr::None(inner_opt) => {
                if let Some(inner) = inner_opt {
                    self.write("({ ok: false, value: ");
                    self.emit_expr(inner);
                    self.write(" })");
                } else {
                    self.write("({ ok: false })");
                }
            }
            Expr::Call { name, args } => {
                // Map built-in functions
                let js_name = if name == "print" {
                    "__print"
                } else {
                    name.as_str()
                };
                self.write(js_name);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
            Expr::Match {
                expr: matched,
                arms,
            } => {
                // Match used inline as an expression (e.g. inside another expr).
                // This is rare — most match exprs are caught at the Stmt level.
                // Emit an IIFE for safety.
                self.write("(() => {\n");
                self.indent += 1;
                let temp = self.fresh_temp();
                self.write_indent();
                self.write(&format!("const {} = ", temp));
                self.emit_expr(matched);
                self.write(";\n");

                for (i, arm) in arms.iter().enumerate() {
                    self.write_indent();
                    if i > 0 {
                        self.write("else ");
                    }

                    self.emit_pattern_condition(&temp, arm);

                    self.write(" {\n");
                    self.indent += 1;

                    self.emit_bound_variables(&temp, &arm.pattern);

                    for s in &arm.stmts {
                        self.emit_stmt(s);
                    }
                    if let Some(result) = &arm.result {
                        self.write_indent();
                        self.write("return ");
                        self.emit_expr(result);
                        self.write(";\n");
                    } else {
                        self.writeln("return undefined;");
                    }
                    self.indent -= 1;
                    self.writeln("}");
                }

                self.indent -= 1;
                self.write("})()");
            }
            Expr::Array(elements) => {
                self.write("[");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                }
                self.write("]");
            }
            Expr::Index { expr, index } => {
                self.emit_expr(expr);
                self.write("[");
                self.emit_expr(index);
                self.write("]");
            }
            Expr::Range {
                start,
                end,
                inclusive,
            } => {
                // Emit a helper that generates the range array
                // For numbers: Array.from({length: end - start + (inclusive ? 1 : 0)}, (_, i) => i + start)
                // For chars: same but with String.fromCharCode
                self.write("((__s, __e) => {");
                self.write("if (typeof __s === 'number') ");
                if *inclusive {
                    self.write("return Array.from({length: __e - __s + 1}, (_, i) => i + __s); ");
                } else {
                    self.write("return Array.from({length: __e - __s}, (_, i) => i + __s); ");
                }
                self.write("else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); ");
                if *inclusive {
                    self.write("return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); ");
                } else {
                    self.write("return Array.from({length: ec - sc}, (_, i) => String.fromCharCode(i + sc)); ");
                }
                self.write("}})(");
                self.emit_expr(start);
                self.write(", ");
                self.emit_expr(end);
                self.write(")");
            }
            Expr::Interpolation(parts) => {
                // Emit JS template literal: `Hello ${name}!`
                self.write("`");
                for part in parts {
                    match part {
                        Expr::StringLit(s) => self.write(s),
                        other => {
                            self.write("${");
                            self.emit_expr(other);
                            self.write("}");
                        }
                    }
                }
                self.write("`");
            }
            Expr::Try { expr, error_expr } => {
                // Nested Try expression - using an IIFE.
                // Note: This won't early-return from the parent function if nested.
                // Parent-return only works for top-level stmt try (handled in emit_stmt).
                self.write("(() => { ");
                let temp = self.fresh_temp();
                self.write(&format!("const {} = ", temp));
                self.emit_expr(expr);
                self.write(&format!("; if (!{}.ok) throw new Error(", temp));
                if let Some(err) = error_expr {
                    self.emit_expr(err);
                } else {
                    self.write(&format!("{}.value", temp));
                }
                self.write("); return ");
                self.write(&temp);
                self.write(".value; })()");
            }
            Expr::StructInit { fields, .. } => {
                self.write("{ ");
                for (i, (field_name, field_expr)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&format!("{}: ", field_name));
                    self.emit_expr(field_expr);
                }
                self.write(" }");
            }
            Expr::PropertyAccess { expr, property } => {
                self.emit_expr(expr);
                self.write(&format!(".{}", property));
            }
            Expr::MethodCall { expr, method, args } => {
                // Resolve whether this is an extension call by looking up the receiver type.
                let receiver_type_key: Option<String> = match expr.as_ref() {
                    Expr::Identifier(name) => self.var_types.get(name).cloned(),
                    _ => None,
                };
                let is_extension = receiver_type_key
                    .as_ref()
                    .and_then(|tk| self.ext_methods.get(tk))
                    .map(|methods| methods.contains(method))
                    .unwrap_or(false);

                if is_extension {
                    let type_name = receiver_type_key.unwrap();
                    self.write(&format!("__ext_{}_{}(", type_name, method));
                    self.emit_expr(expr);
                    for arg in args {
                        self.write(", ");
                        self.emit_expr(arg);
                    }
                    self.write(")");
                } else {
                    // Regular JS method call (closure field, interop, etc.)
                    self.emit_expr(expr);
                    self.write(&format!(".{}(", method));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.emit_expr(arg);
                    }
                    self.write(")");
                }
            }
            Expr::EnumInit {
                enum_name: _,
                variant_name,
                args,
            } => {
                self.write("{ $variant: \"");
                self.write(variant_name);
                self.write("\", $data: [");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write("] }");
            }
            Expr::Closure { params, body, .. } => {
                self.write("(");
                for (i, (name, _)) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(name);
                }
                self.write(") => ");
                self.emit_expr(body);
            }
            Expr::Block(stmts, result) => {
                // If this is a block expression, we might need a different emitting strategy
                // depending on context. For now, we emit as a block.
                // If it's used as a closure body, it works perfectly.
                self.write("{\n");
                self.indent += 1;
                for stmt in stmts {
                    self.write_indent();
                    self.emit_stmt(stmt);
                }
                if let Some(res) = result {
                    self.write_indent();
                    self.write("return ");
                    self.emit_expr(res);
                    self.write(";\n");
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
            }
        }
    }
}
