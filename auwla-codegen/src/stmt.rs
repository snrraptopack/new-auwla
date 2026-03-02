use crate::emitter::JsEmitter;
use auwla_ast::{Method, Stmt};

impl JsEmitter {
    pub(crate) fn emit_stmt(&mut self, stmt: &Stmt) {
        self.emit_stmt_inner(stmt, false);
    }

    /// Core statement emitter. When `export` is true, the appropriate
    /// `export` keyword is prepended — no string-replace hacks needed.
    fn emit_stmt_inner(&mut self, stmt: &Stmt, export: bool) {
        match &stmt.node {
            auwla_ast::StmtKind::Let {
                name,
                ty,
                initializer,
                ..
            } => self.emit_binding_decl("const", name, ty, initializer, export),

            auwla_ast::StmtKind::Var {
                name,
                ty,
                initializer,
                ..
            } => self.emit_binding_decl("let", name, ty, initializer, export),

            auwla_ast::StmtKind::DestructureLet {
                bindings,
                initializer,
            } => {
                self.write_indent();
                if export {
                    self.write("export ");
                }
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
                        auwla_ast::Type::Array(_) => "array".to_string(),
                        auwla_ast::Type::Optional(_) => "optional".to_string(),
                        _ => format!("{:?}", ty),
                    };
                    self.var_types.insert(param_name.clone(), type_name);
                }
                self.write_indent();
                if export {
                    self.write("export ");
                }
                let param_names: Vec<&str> = params.iter().map(|(n, _)| n.as_str()).collect();
                self.write(&format!(
                    "function {}({}) {{\n",
                    name,
                    param_names.join(", ")
                ));
                self.out.indent();
                for s in body {
                    self.emit_stmt(s);
                }
                self.out.dedent();
                self.writeln("}");
            }
            auwla_ast::StmtKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    if let auwla_ast::ExprKind::Match {
                        expr: matched,
                        arms,
                    } = &expr.node
                    {
                        self.emit_match_return(matched, arms);
                    } else if let auwla_ast::ExprKind::Try {
                        expr: tried,
                        error_expr,
                    } = &expr.node
                    {
                        self.emit_try_standalone(tried, error_expr);
                        self.write_indent();
                        self.write("return ");
                        self.emit_expr(expr);
                        self.write(";\n");
                    } else {
                        self.write_indent();
                        self.write("return ");
                        self.emit_expr(expr);
                        self.write(";\n");
                    }
                } else {
                    self.write_indent();
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
                self.out.indent();
                for s in then_branch {
                    self.emit_stmt(s);
                }
                self.out.dedent();
                if let Some(els) = else_branch {
                    self.writeln("} else {");
                    self.out.indent();
                    for s in els {
                        self.emit_stmt(s);
                    }
                    self.out.dedent();
                }
                self.writeln("}");
            }
            auwla_ast::StmtKind::While { condition, body } => {
                self.write_indent();
                self.write("while (");
                self.emit_expr(condition);
                self.write(") {\n");
                self.out.indent();
                for s in body {
                    self.emit_stmt(s);
                }
                self.out.dedent();
                self.writeln("}");
            }
            auwla_ast::StmtKind::For {
                binding,
                iterable,
                body,
            } => {
                if let auwla_ast::ExprKind::Range {
                    start,
                    end,
                    inclusive,
                } = &iterable.node
                {
                    // Optimized number range loop
                    self.write_indent();
                    let start_str = self.emit_expr_to_string(start);
                    let end_str = self.emit_expr_to_string(end);
                    let op = if *inclusive { "<=" } else { "<" };
                    self.write(&format!(
                        "for (let {} = {}; {} {} {}; {}++) {{\n",
                        binding, start_str, binding, op, end_str, binding
                    ));
                } else {
                    self.write_indent();
                    self.write(&format!("for (const {} of ", binding));
                    self.emit_expr(iterable);
                    self.write(") {\n");
                }
                self.out.indent();
                for s in body {
                    self.emit_stmt(s);
                }
                self.out.dedent();
                self.writeln("}");
            }
            auwla_ast::StmtKind::Expr(expr) => {
                if let auwla_ast::ExprKind::Match { expr, arms } = &expr.node {
                    self.emit_match_standalone(expr, arms);
                } else if let auwla_ast::ExprKind::Try { expr, error_expr } = &expr.node {
                    self.emit_try_standalone(expr, error_expr);
                } else {
                    self.write_indent();
                    self.emit_expr(expr);
                    self.write(";\n");
                }
            }
            auwla_ast::StmtKind::StructDecl { .. }
            | auwla_ast::StmtKind::EnumDecl { .. }
            | auwla_ast::StmtKind::TypeAlias { .. } => {
                // Struct/Enum declarations vanish in JS, they are purely for compile-time typechecking.
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
                    auwla_ast::StmtKind::StructDecl { .. }
                    | auwla_ast::StmtKind::EnumDecl { .. }
                    | auwla_ast::StmtKind::TypeAlias { .. } => {
                        // types vanish in JS output — no-op
                    }
                    _ => {
                        // Re-enter emit_stmt_inner with export=true
                        self.emit_stmt_inner(inner, true);
                    }
                }
            }
            auwla_ast::StmtKind::Extend {
                type_name,
                type_args,
                methods,
                ..
            } => {
                let type_key = self.extend_key(type_name, type_args);
                self.emit_method_block(&type_key, methods, true);
            }
            auwla_ast::StmtKind::TypeDecl { name, methods, .. } => {
                self.emit_method_block(name, methods, false);
            }
        }
    }

    // ── Unified binding declaration (replaces duplicated Let/Var) ──

    /// Emit a `const`/`let` binding, handling Match/Try initializers,
    /// type inference, and range detection in one place.
    fn emit_binding_decl(
        &mut self,
        kw: &str,
        name: &str,
        ty: &Option<auwla_ast::Type>,
        initializer: &auwla_ast::expr::Expr,
        export: bool,
    ) {
        // Register the variable's type for extension method resolution.
        self.register_var_type(name, ty, initializer);

        // Special-case: match-as-initializer
        if let auwla_ast::ExprKind::Match { expr, arms } = &initializer.node {
            // For exported match-init we still need the decl prefix from emit_match_assign
            self.emit_match_assign(kw, name, expr, arms);
            return;
        }

        // Special-case: try-as-initializer
        if let auwla_ast::ExprKind::Try { expr, error_expr } = &initializer.node {
            self.emit_try_assign(kw, name, expr, error_expr);
            return;
        }

        self.write_indent();
        if export {
            self.write("export ");
        }
        self.write(&format!("{} {} = ", kw, name));
        self.emit_expr(initializer);
        self.write(";\n");
    }

    // ── Unified method-block emission (replaces duplicated Extend/TypeDecl) ──

    /// Emit all methods for a type as standalone `_ext_TypeKey_method(...)` functions
    /// into the extensions output buffer.
    ///
    /// `wrap_optional` controls whether external-attribute property/method returns
    /// are wrapped in `{ ok, value }` for `Optional<T>` return types (Extend does
    /// this; TypeDecl does not).
    fn emit_method_block(&mut self, type_key: &str, methods: &[Method], wrap_optional: bool) {
        let safe_type_key = self.type_key_ident(type_key);

        for method in methods {
            // Register method parameters in var_types
            for (param_name, ty_opt) in &method.params {
                if param_name == "self" {
                    self.var_types
                        .insert("__self".to_string(), type_key.to_string());
                    self.var_types
                        .insert("self".to_string(), type_key.to_string());
                } else if let Some(ty) = ty_opt {
                    let t_name = self.type_to_key(ty);
                    self.var_types.insert(param_name.clone(), t_name);
                }
            }

            // Emit function signature
            if method.is_static {
                self.write_indent_ext();
                self.write_ext(&format!(
                    "export function _ext_{}_{}(",
                    safe_type_key, method.name
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
                self.write_indent_ext();
                self.write_ext(&format!(
                    "export function _ext_{}_{}(__self",
                    safe_type_key, method.name
                ));
                for (pname, _) in method.params.iter().filter(|(n, _)| n != "self") {
                    self.write_ext(", ");
                    self.write_ext(pname);
                }
                self.write_ext(") {\n");
            }

            self.ext.indent();

            // Check for @external attribute
            let external_attr = method
                .attributes
                .iter()
                .find(|a| a.name == "external")
                .cloned();

            if let Some(attr) = external_attr {
                self.emit_external_body(&attr, method, wrap_optional);
            } else {
                // Emit user-defined method body with self→__self renaming
                let old_output = std::mem::replace(&mut self.out, crate::writer::CodeWriter::new());
                // Synchronize indent level
                self.out.set_indent(self.ext.indent_level());

                for s in &method.body {
                    self.emit_stmt_with_self_rename(s);
                }

                let body_writer = std::mem::replace(&mut self.out, old_output);
                self.write_ext(&body_writer.into_string());
            }

            self.ext.dedent();
            self.writeln_ext("}\n");
        }
    }

    /// Emit a statement, replacing identifier `self` with `__self` for method bodies.
    pub(crate) fn emit_stmt_with_self_rename(&mut self, stmt: &Stmt) {
        let old = self.in_extension_method;
        self.in_extension_method = true;
        self.emit_stmt(stmt);
        self.in_extension_method = old;
    }
}
