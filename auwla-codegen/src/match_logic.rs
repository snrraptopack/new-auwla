use crate::emitter::JsEmitter;
use auwla_ast::{Expr, MatchArm};

impl JsEmitter {
    /// Emit a match expression as an IIFE or inline block.
    pub(crate) fn emit_match_expr(&mut self, matched: &Expr, arms: &[MatchArm]) {
        self.write("(() => {\n");
        self.indent += 1;
        let temp = self.fresh_temp();
        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(matched);
        self.write(";\n");

        self.emit_match_arms(&temp, arms, None);

        self.indent -= 1;
        self.write_indent();
        self.write("})()");
    }

    /// Emit match arms logic. If `target_var` is Some, it's an assignment match.
    pub(crate) fn emit_match_arms(
        &mut self,
        temp: &str,
        arms: &[MatchArm],
        target_var: Option<&str>,
    ) {
        // Optimization: Try to emit a switch statement if all patterns are basic (Enum or Literal)
        if self.can_emit_switch(arms) {
            self.emit_match_switch(temp, arms, target_var);
            return;
        }

        // Fallback to if-else logic
        for (i, arm) in arms.iter().enumerate() {
            self.write_indent();
            if i > 0 {
                self.write("else ");
            }

            self.emit_pattern_condition(temp, arm);
            self.write(" {\n");
            self.indent += 1;

            self.emit_bound_variables(temp, &arm.pattern);

            for s in &arm.stmts {
                self.emit_stmt(s);
            }

            if let Some(target) = target_var {
                if let Some(result) = &arm.result {
                    self.write_indent();
                    if target == "return" {
                        self.write("return ");
                        self.emit_expr(result);
                    } else if target.is_empty() {
                        self.emit_expr(result);
                    } else {
                        self.write(&format!("{} = ", target));
                        self.emit_expr(result);
                    }
                    self.write(";\n");
                } else if target == "return" {
                    self.write_indent();
                    self.write("return;\n");
                }
            } else {
                // Expr-style match (IIFE)
                self.write_indent();
                self.write("return ");
                if let Some(result) = &arm.result {
                    self.emit_expr(result);
                } else {
                    self.write("undefined");
                }
                self.write(";\n");
            }

            self.indent -= 1;
            self.writeln("}");
        }
    }

    pub(crate) fn emit_match_standalone(&mut self, matched: &Expr, arms: &[MatchArm]) {
        let temp = self.fresh_temp();
        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(matched);
        self.write(";\n");

        self.emit_match_arms(&temp, arms, Some(""));
    }

    pub(crate) fn emit_match_assign(
        &mut self,
        decl_kw: &str,
        name: &str,
        matched: &Expr,
        arms: &[MatchArm],
    ) {
        let temp = self.fresh_temp();
        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(matched);
        self.write(";\n");

        if !decl_kw.is_empty() {
            self.write_indent();
            // Match assignment MUST use 'let' if declared separately from assignment
            self.write(&format!("let {};\n", name));
        }

        self.emit_match_arms(&temp, arms, Some(name));
    }

    pub(crate) fn emit_match_return(&mut self, matched: &Expr, arms: &[MatchArm]) {
        let temp = self.fresh_temp();
        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(matched);
        self.write(";\n");

        self.emit_match_arms(&temp, arms, Some("return"));
    }

    fn can_emit_switch(&self, arms: &[MatchArm]) -> bool {
        if arms.is_empty() {
            return false;
        }
        let mut first_kind = None;
        for arm in arms {
            if arm.guard.is_some() {
                return false;
            }
            let kind = match &arm.pattern.node {
                auwla_ast::PatternKind::Literal(_) => "literal",
                auwla_ast::PatternKind::Variant { name, .. } => {
                    if name == "some" || name == "none" {
                        return false;
                    } // some/none use .ok
                    "variant"
                }
                auwla_ast::PatternKind::Wildcard => "any",
                _ => return false,
            };
            if kind == "any" {
                continue;
            }
            if let Some(prev) = first_kind {
                if prev != kind {
                    return false;
                }
            } else {
                first_kind = Some(kind);
            }
        }
        true
    }

    fn emit_match_switch(&mut self, temp: &str, arms: &[MatchArm], target_var: Option<&str>) {
        self.write_indent();
        let switch_on = match &arms[0].pattern.node {
            auwla_ast::PatternKind::Variant { .. } => format!("{}.$variant", temp),
            _ => temp.to_string(),
        };
        self.write(&format!("switch ({}) {{\n", switch_on));
        self.indent += 1;

        for arm in arms {
            self.write_indent();
            match &arm.pattern.node {
                auwla_ast::PatternKind::Literal(lit) => {
                    self.write("case ");
                    self.emit_expr(lit);
                    self.write(":\n");
                }
                auwla_ast::PatternKind::Variant { name, .. } => {
                    self.write(&format!("case \"{}\":\n", name));
                }
                auwla_ast::PatternKind::Wildcard => {
                    self.write("default:\n");
                }
                _ => unreachable!(),
            }
            self.indent += 1;

            self.emit_bound_variables(temp, &arm.pattern);

            for s in &arm.stmts {
                self.emit_stmt(s);
            }

            if let Some(target) = target_var {
                if let Some(result) = &arm.result {
                    self.write_indent();
                    if target == "return" {
                        self.write("return ");
                        self.emit_expr(result);
                    } else if target.is_empty() {
                        self.emit_expr(result);
                    } else {
                        self.write(&format!("{} = ", target));
                        self.emit_expr(result);
                    }
                    self.write(";\n");
                } else if target == "return" {
                    self.write_indent();
                    self.write("return;\n");
                }
                if target != "return" {
                    self.write_indent();
                    self.write("break;\n");
                }
            } else {
                // Expr-style match (IIFE)
                self.write_indent();
                self.write("return ");
                if let Some(result) = &arm.result {
                    self.emit_expr(result);
                } else {
                    self.write("undefined");
                }
                self.write(";\n");
            }

            self.indent -= 1;
        }

        self.indent -= 1;
        self.writeln("}");
    }
}
