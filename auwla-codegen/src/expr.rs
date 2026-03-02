use crate::emitter::JsEmitter;
use auwla_ast::{BinaryOp, Expr, UnaryOp};

impl JsEmitter {
    pub(crate) fn emit_expr(&mut self, expr: &Expr) {
        match &expr.node {
            auwla_ast::ExprKind::Void => self.write("undefined"),
            auwla_ast::ExprKind::StringLit(s) => self.write(&format!("\"{}\"", s)),
            auwla_ast::ExprKind::NumberLit(n) => {
                // Emit integers without decimal point
                if *n == (*n as i64) as f64 {
                    self.write(&format!("{}", *n as i64));
                } else {
                    self.write(&format!("{}", n));
                }
            }
            auwla_ast::ExprKind::BoolLit(b) => self.write(if *b { "true" } else { "false" }),
            auwla_ast::ExprKind::CharLit(c) => self.write(&format!("\"{}\"", c)),
            auwla_ast::ExprKind::Identifier(name) => {
                if self.in_extension_method && name == "self" {
                    self.write("__self");
                } else {
                    self.write(name);
                }
            }
            auwla_ast::ExprKind::Binary { op, left, right } => {
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
            auwla_ast::ExprKind::Unary { op, expr: inner } => {
                match op {
                    UnaryOp::Not => self.write("!"),
                    UnaryOp::Neg => self.write("-"),
                }
                self.emit_expr(inner);
            }
            auwla_ast::ExprKind::Some(inner) => {
                self.write("({ ok: true, value: ");
                self.emit_expr(inner);
                self.write(" })");
            }
            auwla_ast::ExprKind::None(inner_opt) => {
                if let Some(inner) = inner_opt {
                    self.write("({ ok: false, value: ");
                    self.emit_expr(inner);
                    self.write(" })");
                } else {
                    self.write("({ ok: false })");
                }
            }
            auwla_ast::ExprKind::Call { name, args, .. } => {
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
            auwla_ast::ExprKind::Match {
                expr: matched,
                arms,
            } => {
                self.emit_match_expr(matched, arms);
            }
            auwla_ast::ExprKind::Array(elements) => {
                self.write("[");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                }
                self.write("]");
            }
            auwla_ast::ExprKind::Index { expr, index } => {
                self.emit_expr(expr);
                self.write("[");
                self.emit_expr(index);
                self.write("]");
            }
            auwla_ast::ExprKind::Range {
                start,
                end,
                inclusive,
            } => {
                // Emit a helper that generates the range array
                self.write("__range(");
                self.emit_expr(start);
                self.write(", ");
                self.emit_expr(end);
                self.write(", ");
                self.write(if *inclusive { "true" } else { "false" });
                self.write(")");
            }
            auwla_ast::ExprKind::Interpolation(parts) => {
                // Emit JS template literal: `Hello ${name}!`
                self.write("`");
                for part in parts {
                    match &part.node {
                        auwla_ast::ExprKind::StringLit(s) => self.write(s),
                        _ => {
                            self.write("${");
                            self.emit_expr(part);
                            self.write("}");
                        }
                    }
                }
                self.write("`");
            }
            auwla_ast::ExprKind::Try { expr, error_expr } => {
                self.emit_try_expr(expr, error_expr);
            }
            auwla_ast::ExprKind::StructInit { fields, .. } => {
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
            auwla_ast::ExprKind::PropertyAccess { expr, property } => {
                self.emit_expr(expr);
                self.write(&format!(".{}", property));
            }
            auwla_ast::ExprKind::MethodCall {
                expr, method, args, ..
            } => {
                let receiver_type_key: Option<String> = match &expr.node {
                    auwla_ast::ExprKind::Identifier(name) => self.var_types.get(name).cloned(),
                    auwla_ast::ExprKind::StringLit(_) => Some("string".to_string()),
                    auwla_ast::ExprKind::NumberLit(_) => Some("number".to_string()),
                    auwla_ast::ExprKind::BoolLit(_) => Some("bool".to_string()),
                    auwla_ast::ExprKind::Array(elems) => self.array_literal_type_key(elems),
                    auwla_ast::ExprKind::Range { .. } => Some("array<number>".to_string()),
                    _ => None,
                };

                let mut resolved_key: Option<String> = None;
                if let Some(tk) = receiver_type_key.as_ref() {
                    let mut keys = vec![tk.clone()];
                    if let Some(idx) = tk.find('<') {
                        keys.push(tk[..idx].to_string());
                    }
                    for key in keys {
                        if let Some(methods) = self.ext_methods.get(&key) {
                            if methods.contains(method) {
                                resolved_key = Some(key);
                                break;
                            }
                        }
                    }
                }

                if let Some(type_name) = resolved_key {
                    let safe_type = self.type_key_ident(&type_name);
                    self.write(&format!("_ext_{}_{}(", safe_type, method));
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
            auwla_ast::ExprKind::StaticMethodCall {
                type_name,
                type_args,
                method,
                args,
                ..
            } => {
                let is_extension = self
                    .extensions
                    .get(&self.extend_key(type_name, type_args))
                    .map(|methods| {
                        methods
                            .iter()
                            .any(|m| m.name == method.as_str() && m.is_static)
                    })
                    .unwrap_or(false);

                if is_extension {
                    let type_key = self.extend_key(type_name, type_args);
                    let safe_type = self.type_key_ident(&type_key);
                    self.write(&format!("_ext_{}_{}(", safe_type, method));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.emit_expr(arg);
                    }
                    self.write(")");
                } else {
                    self.write(&format!("{}.{}(", type_name, method));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.emit_expr(arg);
                    }
                    self.write(")");
                }
            }
            auwla_ast::ExprKind::EnumInit {
                variant_name, args, ..
            } => {
                self.write("{ $variant: \"");
                self.write(variant_name);
                self.write("\"");
                if !args.is_empty() {
                    self.write(", $data: [");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.emit_expr(arg);
                    }
                    self.write("]");
                }
                self.write(" }");
            }
            auwla_ast::ExprKind::Closure { params, body, .. } => {
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
            auwla_ast::ExprKind::Block(stmts, result) => {
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
                } else if !stmts.is_empty() {
                    self.write_indent();
                    self.write("return undefined;\n");
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
            }
        }
    }
}
