use crate::emitter::JsEmitter;
use auwla_ast::{Expr, MatchArm};

impl JsEmitter {
    pub(crate) fn emit_match_assign(
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

    pub(crate) fn emit_pattern_condition(&mut self, temp: &str, arm: &MatchArm) {
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

    pub(crate) fn emit_pattern_shape(&mut self, temp: &str, pattern: &auwla_ast::Pattern) {
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
    pub(crate) fn emit_match_standalone(&mut self, matched_expr: &Expr, arms: &Vec<MatchArm>) {
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
    pub(crate) fn emit_try_assign(
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
    pub(crate) fn emit_try_standalone(&mut self, expr: &Expr, error_expr: &Option<Box<Expr>>) {
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

    pub(crate) fn emit_bound_variables(&mut self, temp: &str, pattern: &auwla_ast::Pattern) {
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
    pub(crate) fn emit_arm_body(&mut self, temp: &str, target: &str, arm: &MatchArm) {
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
    pub(crate) fn emit_arm_body_standalone(&mut self, temp: &str, arm: &MatchArm) {
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
}
