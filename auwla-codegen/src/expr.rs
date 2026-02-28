use crate::emitter::JsEmitter;
use auwla_ast::{BinaryOp, Expr, UnaryOp};

impl JsEmitter {
    pub(crate) fn emit_expr(&mut self, expr: &Expr) {
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
            Expr::Call { name, args, .. } => {
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
            Expr::MethodCall {
                expr, method, args, ..
            } => {
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
                ..
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
