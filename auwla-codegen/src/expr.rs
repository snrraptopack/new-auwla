use crate::emitter::JsEmitter;
use auwla_ast::{BinaryOp, Expr, Type, UnaryOp};

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
                let receiver_type_key: Option<String> = self.infer_expr_type(expr);

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
                    // Check if this method has an @external attribute — if so, inline it
                    if let Some((attr, return_ty)) = self.find_external_attr(&type_name, method) {
                        let needs_optional_wrap = matches!(&return_ty, Some(Type::Optional(_)));

                        if attr.args.first().map(|s| s.as_str()) == Some("js") {
                            let mapping = attr.args.get(1).map(|s| s.as_str());
                            match mapping {
                                Some("property") => {
                                    let js_name =
                                        attr.args.get(2).map(|s| s.as_str()).unwrap_or(method);
                                    if needs_optional_wrap {
                                        self.write("((_r = ");
                                        self.emit_expr(expr);
                                        self.write(&format!(".{}", js_name));
                                        self.write(
                                            ") != null ? { ok: true, value: _r } : { ok: false })",
                                        );
                                    } else {
                                        self.emit_expr(expr);
                                        self.write(&format!(".{}", js_name));
                                    }
                                }
                                Some("method") => {
                                    let js_name =
                                        attr.args.get(2).map(|s| s.as_str()).unwrap_or(method);
                                    if needs_optional_wrap {
                                        self.write("((_r = ");
                                        self.emit_expr(expr);
                                        self.write(&format!(".{}(", js_name));
                                        for (i, arg) in args.iter().enumerate() {
                                            if i > 0 {
                                                self.write(", ");
                                            }
                                            self.emit_expr(arg);
                                        }
                                        self.write(
                                            ")) != null ? { ok: true, value: _r } : { ok: false })",
                                        );
                                    } else {
                                        self.emit_expr(expr);
                                        self.write(&format!(".{}(", js_name));
                                        for (i, arg) in args.iter().enumerate() {
                                            if i > 0 {
                                                self.write(", ");
                                            }
                                            self.emit_expr(arg);
                                        }
                                        self.write(")");
                                    }
                                }
                                _ => {
                                    // Unknown @external mapping — fall through to _ext_ wrapper
                                    let safe_type = self.type_key_ident(&type_name);
                                    self.write(&format!("_ext_{}_{}(", safe_type, method));
                                    self.emit_expr(expr);
                                    for arg in args {
                                        self.write(", ");
                                        self.emit_expr(arg);
                                    }
                                    self.write(")");
                                }
                            }
                        } else {
                            // Non-JS @external — use _ext_ wrapper
                            let safe_type = self.type_key_ident(&type_name);
                            self.write(&format!("_ext_{}_{}(", safe_type, method));
                            self.emit_expr(expr);
                            for arg in args {
                                self.write(", ");
                                self.emit_expr(arg);
                            }
                            self.write(")");
                        }
                    } else {
                        // Pure Auwla extension method — use _ext_ wrapper
                        let safe_type = self.type_key_ident(&type_name);
                        self.write(&format!("_ext_{}_{}(", safe_type, method));
                        self.emit_expr(expr);
                        for arg in args {
                            self.write(", ");
                            self.emit_expr(arg);
                        }
                        self.write(")");
                    }
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
                if self.enums.contains(type_name) {
                    // Emit as enum initialization
                    self.write("{ $variant: \"");
                    self.write(method);
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
                    return;
                }

                let type_key = self.extend_key(type_name, type_args);

                // Look up the method and its @external attribute
                let method_info = self
                    .extensions
                    .get(&type_key)
                    .or_else(|| self.extensions.get(type_name))
                    .and_then(|methods| {
                        methods
                            .iter()
                            .find(|m| m.name == method.as_str() && m.is_static)
                    });

                if let Some(method_sig) = method_info {
                    let ext_attr = method_sig.attributes.iter().find(|a| a.name == "external");

                    if let Some(attr) = ext_attr {
                        // Check for @external("js", "constructor")
                        let is_js_constructor = attr.args.first().map(|s| s.as_str()) == Some("js")
                            && attr.args.get(1).map(|s| s.as_str()) == Some("constructor");
                        // Backwards compat: @external("constructor")
                        let is_old_constructor =
                            attr.args.first().map(|s| s.as_str()) == Some("constructor");

                        if is_js_constructor || is_old_constructor {
                            self.write(&format!("new {}(", type_name));
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.write(", ");
                                }
                                self.emit_expr(arg);
                            }
                            self.write(")");
                        } else if attr.args.first().map(|s| s.as_str()) == Some("js") {
                            // Namespace / class static inline:
                            // @external("js", "method", "floor") on Math → Math.floor(args)
                            let mapping = attr.args.get(1).map(|s| s.as_str());
                            match mapping {
                                Some("method") | Some("static") => {
                                    // Infer JS object name from type_name, JS method from 3rd arg or fallback
                                    let js_method =
                                        attr.args.get(2).map(|s| s.as_str()).unwrap_or(method);
                                    // For @external("js","static","Obj","method") (old 4-arg pattern)
                                    let (js_obj, js_fn) = if attr.args.len() >= 4 {
                                        (attr.args[2].as_str(), attr.args[3].as_str())
                                    } else {
                                        (type_name.as_str(), js_method)
                                    };
                                    self.write(&format!("{}.{}(", js_obj, js_fn));
                                    for (i, arg) in args.iter().enumerate() {
                                        if i > 0 {
                                            self.write(", ");
                                        }
                                        self.emit_expr(arg);
                                    }
                                    self.write(")");
                                }
                                Some("property") => {
                                    let js_prop =
                                        attr.args.get(2).map(|s| s.as_str()).unwrap_or(method);
                                    let (js_obj, js_fn) = if attr.args.len() >= 4 {
                                        (attr.args[2].as_str(), attr.args[3].as_str())
                                    } else {
                                        (type_name.as_str(), js_prop)
                                    };
                                    self.write(&format!("{}.{}", js_obj, js_fn));
                                }
                                Some("const") => {
                                    let js_name =
                                        attr.args.get(2).map(|s| s.as_str()).unwrap_or(method);
                                    let (js_obj, js_fn) = if attr.args.len() >= 4 {
                                        (attr.args[2].as_str(), attr.args[3].as_str())
                                    } else {
                                        (type_name.as_str(), js_name)
                                    };
                                    self.write(&format!("{}.{}", js_obj, js_fn));
                                }
                                _ => {
                                    // Fallback to _ext_ wrapper
                                    let safe_type = self.type_key_ident(&type_key);
                                    self.write(&format!("_ext_{}_{}(", safe_type, method));
                                    for (i, arg) in args.iter().enumerate() {
                                        if i > 0 {
                                            self.write(", ");
                                        }
                                        self.emit_expr(arg);
                                    }
                                    self.write(")");
                                }
                            }
                        } else {
                            // Unknown @external target — use _ext_ wrapper
                            let safe_type = self.type_key_ident(&type_key);
                            self.write(&format!("_ext_{}_{}(", safe_type, method));
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.write(", ");
                                }
                                self.emit_expr(arg);
                            }
                            self.write(")");
                        }
                    } else {
                        // No @external — pure Auwla static, use _ext_ wrapper
                        let safe_type = self.type_key_ident(&type_key);
                        self.write(&format!("_ext_{}_{}(", safe_type, method));
                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 {
                                self.write(", ");
                            }
                            self.emit_expr(arg);
                        }
                        self.write(")");
                    }
                } else {
                    // Not a known extension — emit as plain static call (e.g., JS interop)
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
                self.out.indent();
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
                self.out.dedent();
                self.write_indent();
                self.write("}");
            }
        }
    }
}
