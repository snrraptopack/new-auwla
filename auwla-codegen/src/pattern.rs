use crate::emitter::JsEmitter;
use auwla_ast::MatchArm;

impl JsEmitter {
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
        match &pattern.node {
            auwla_ast::PatternKind::Wildcard | auwla_ast::PatternKind::Variable(_) => {
                self.write("true");
            }
            auwla_ast::PatternKind::Literal(expr) => {
                self.write(&format!("{} === ", temp));
                self.emit_expr(expr);
            }
            auwla_ast::PatternKind::Variant { name, bindings: _ } => {
                if name == "some" {
                    self.write(&format!("{}.ok", temp));
                } else if name == "none" {
                    self.write(&format!("!{}.ok", temp));
                } else {
                    self.write(&format!("{}.$variant === \"{}\"", temp, name));
                }
            }
            auwla_ast::PatternKind::Range {
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
            auwla_ast::PatternKind::Or(patterns) => {
                self.write("(");
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.write(" || ");
                    }
                    self.emit_pattern_shape(temp, p);
                }
                self.write(")");
            }
            auwla_ast::PatternKind::Struct(_name, fields) => {
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

    pub(crate) fn emit_bound_variables(&mut self, temp: &str, pattern: &auwla_ast::Pattern) {
        match &pattern.node {
            auwla_ast::PatternKind::Variable(name) => {
                self.write_indent();
                self.write(&format!("const {} = {};\n", name, temp));
            }
            auwla_ast::PatternKind::Variant { name, bindings } => {
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
            auwla_ast::PatternKind::Or(patterns) => {
                // For 'Or' patterns, we assume they all bind the same variables if any.
                // Typechecker guarantees they are compatible.
                if let Some(first) = patterns.first() {
                    self.emit_bound_variables(temp, first);
                }
            }
            auwla_ast::PatternKind::Struct(_name, fields) => {
                for (fname, sub_pattern_opt) in fields {
                    if let Some(sub_pattern) = sub_pattern_opt {
                        let inner_temp = format!("{}.{}", temp, fname);
                        self.emit_bound_variables(&inner_temp, sub_pattern);
                    } else {
                        self.write_indent();
                        self.write(&format!("const {} = {}.{};\n", fname, temp, fname));
                    }
                }
            }
            _ => {}
        }
    }
}
