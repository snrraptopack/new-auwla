use auwla_ast::expr::{BinaryOp, Expr, MatchArm, UnaryOp};
use auwla_ast::stmt::Stmt;
use auwla_ast::Program;

/// Emits JavaScript source code from a type-checked Auwla AST.
pub fn emit_js(program: &Program) -> String {
    let mut emitter = JsEmitter::new();
    emitter.emit_program(program);
    emitter.output
}

struct JsEmitter {
    output: String,
    indent: usize,
    /// Counter for generating unique temp variable names (e.g. __match_0, __match_1)
    temp_counter: usize,
}

impl JsEmitter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            temp_counter: 0,
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

    fn emit_expr_to_string(&mut self, expr: &Expr) -> String {
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

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                name, initializer, ..
            } => {
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
                name, initializer, ..
            } => {
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
        }
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
            Expr::Identifier(name) => self.write(name),
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
            Expr::None(inner) => {
                self.write("({ ok: false, value: ");
                self.emit_expr(inner);
                self.write(" })");
            }
            Expr::Call { name, args } => {
                // Map built-in functions
                let js_name = if name == "print" {
                    "console.log"
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
        }
    }
}
