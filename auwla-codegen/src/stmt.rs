use crate::emitter::JsEmitter;
use auwla_ast::Stmt;

impl JsEmitter {
    pub(crate) fn emit_stmt(&mut self, stmt: &Stmt) {
        match &stmt.node {
            auwla_ast::StmtKind::Let {
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
                if let auwla_ast::ExprKind::Match { expr, arms } = &initializer.node {
                    self.emit_match_assign("const", name, expr, arms);
                } else if let auwla_ast::ExprKind::Try { expr, error_expr } = &initializer.node {
                    self.emit_try_assign("const", name, expr, error_expr);
                } else {
                    self.write_indent();
                    self.write(&format!("const {} = ", name));
                    self.emit_expr(initializer);
                    self.write(";\n");
                }
            }
            auwla_ast::StmtKind::Var {
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
                if let auwla_ast::ExprKind::Match { expr, arms } = &initializer.node {
                    self.emit_match_assign("let", name, expr, arms);
                } else if let auwla_ast::ExprKind::Try { expr, error_expr } = &initializer.node {
                    self.emit_try_assign("let", name, expr, error_expr);
                } else {
                    self.write_indent();
                    self.write(&format!("let {} = ", name));
                    self.emit_expr(initializer);
                    self.write(";\n");
                }
            }
            auwla_ast::StmtKind::DestructureLet {
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
            auwla_ast::StmtKind::Assign { target, value } => {
                let target_str = self.emit_expr_to_string(target);
                if let auwla_ast::ExprKind::Match { expr, arms } = &value.node {
                    self.emit_match_assign("", &target_str, expr, arms);
                } else if let auwla_ast::ExprKind::Try { expr, error_expr } = &value.node {
                    self.emit_try_assign("", &target_str, expr, error_expr);
                } else {
                    self.write_indent();
                    self.write(&format!("{} = ", target_str));
                    self.emit_expr(value);
                    self.write(";\n");
                }
            }
            auwla_ast::StmtKind::Fn {
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
            auwla_ast::StmtKind::Return(expr_opt) => {
                self.write_indent();
                if let Some(expr) = expr_opt {
                    self.write("return ");
                    self.emit_expr(expr);
                    self.write(";\n");
                } else {
                    self.write("return;\n");
                }
            }
            auwla_ast::StmtKind::If {
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
            auwla_ast::StmtKind::While { condition, body } => {
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
            auwla_ast::StmtKind::For {
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
            auwla_ast::StmtKind::Expr(expr) => {
                // Standalone match expression (used as statement)
                if let auwla_ast::ExprKind::Match {
                    expr: matched,
                    arms,
                } = &expr.node
                {
                    self.emit_match_standalone(matched, arms);
                } else if let auwla_ast::ExprKind::Try {
                    expr: tried,
                    error_expr,
                } = &expr.node
                {
                    self.emit_try_standalone(tried, error_expr);
                } else {
                    self.write_indent();
                    self.emit_expr(expr);
                    self.write(";\n");
                }
            }
            auwla_ast::StmtKind::StructDecl { .. }
            | auwla_ast::StmtKind::EnumDecl { .. }
            | auwla_ast::StmtKind::TypeAlias { .. } => {
                // Struct/Enum declarations vanish in JS, they are purely for compile-time typechecking
                // We emit nothing to keep it zero-cost.
            }
            auwla_ast::StmtKind::Import { names, path } => {
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
            auwla_ast::StmtKind::Export { stmt: inner } => {
                match &inner.node {
                    auwla_ast::StmtKind::Fn { name: _, .. } => {
                        // Temporarily emit the fn, then prefix with `export `
                        let saved_len = self.output.len();
                        self.emit_stmt(inner);
                        // Find where `function` keyword starts and insert `export `
                        let emitted = &self.output[saved_len..];
                        let new_emitted = emitted.replacen("function ", "export function ", 1);
                        self.output.truncate(saved_len);
                        self.output.push_str(&new_emitted);
                    }
                    auwla_ast::StmtKind::Let { .. } | auwla_ast::StmtKind::Var { .. } => {
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
                    auwla_ast::StmtKind::StructDecl { .. }
                    | auwla_ast::StmtKind::EnumDecl { .. }
                    | auwla_ast::StmtKind::TypeAlias { .. } => {
                        // types vanish in JS output — no-op
                    }
                    _ => {
                        // For anything else (e.g., exported block expressions), emit as-is
                        self.emit_stmt(inner);
                    }
                }
            }
            auwla_ast::StmtKind::Extend {
                type_name, methods, ..
            } => {
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
    pub(crate) fn emit_stmt_with_self_rename(&mut self, stmt: &Stmt) {
        let old = self.in_extension_method;
        self.in_extension_method = true;
        self.emit_stmt(stmt);
        self.in_extension_method = old;
    }

    // ──────────────────────────── Match helpers ──────────────────────

    // Emit: `const/let name = match expr { some(val) => ... none(err) => ... };`
    // Becomes:
    // ```js
    // const __match_N = <expr>;
    // let target;   // or const target, depending on decl_kw
    // if (__match_N.ok) { const val = __match_N.value; ... target = <result>; }
    // else { const err = __match_N.value; ... target = <result>; }
    // ```
}
