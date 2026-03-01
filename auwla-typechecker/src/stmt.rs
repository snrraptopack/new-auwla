use crate::TypeError;
use crate::checker::Typechecker;
use crate::scope::Mutability;
use auwla_ast::{Stmt, Type};

impl Typechecker {
    pub fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        match &stmt.node {
            auwla_ast::StmtKind::Let {
                name,
                ty,
                initializer,
            } => {
                let init_ty = self.check_expr(initializer)?;
                let final_ty = if let Some(declared_ty) = ty {
                    self.assert_type_eq(declared_ty, &init_ty)
                        .map_err(|msg| TypeError {
                            span: initializer.span.clone(),
                            message: msg,
                        })?;
                    declared_ty.clone()
                } else {
                    init_ty
                };
                self.declare_variable(
                    stmt.span.clone(),
                    name.clone(),
                    final_ty,
                    Mutability::Immutable,
                )?;
                Ok(())
            }
            auwla_ast::StmtKind::DestructureLet {
                bindings,
                initializer,
            } => {
                let init_ty = self.check_expr(initializer)?;

                match init_ty {
                    Type::Custom(struct_name) => {
                        let struct_def =
                            self.structs
                                .get(&struct_name)
                                .cloned()
                                .ok_or_else(|| TypeError {
                                    span: initializer.span.clone(),
                                    message: format!(
                                        "Type error: struct '{}' not found",
                                        struct_name
                                    ),
                                })?;

                        for binding in bindings {
                            let field_ty = struct_def
                                .iter()
                                .find(|(f, _)| f == binding)
                                .map(|(_, t)| t.clone())
                                .ok_or_else(|| TypeError {
                                    span: initializer.span.clone(),
                                    message: format!(
                                        "Type error: field '{}' not found on struct '{}'",
                                        binding, struct_name
                                    ),
                                })?;

                            self.declare_variable(
                                stmt.span.clone(),
                                binding.clone(),
                                field_ty,
                                Mutability::Immutable,
                            )?;
                        }
                    }
                    _ => {
                        return self.error(
                            initializer.span.clone(),
                            format!(
                                "Type error: expected struct for destructuring, found '{}'",
                                init_ty
                            ),
                        );
                    }
                }
                Ok(())
            }
            auwla_ast::StmtKind::Var {
                name,
                ty,
                initializer,
            } => {
                let init_ty = self.check_expr(initializer)?;
                let final_ty = if let Some(declared_ty) = ty {
                    self.assert_type_eq(declared_ty, &init_ty)
                        .map_err(|msg| TypeError {
                            span: initializer.span.clone(),
                            message: msg,
                        })?;
                    declared_ty.clone()
                } else {
                    init_ty
                };
                self.declare_variable(
                    stmt.span.clone(),
                    name.clone(),
                    final_ty,
                    Mutability::Mutable,
                )?;
                Ok(())
            }
            auwla_ast::StmtKind::Assign { target, value } => {
                let val_ty = self.check_expr(value)?;

                match &target.node {
                    auwla_ast::ExprKind::Identifier(name) => {
                        let var_ty = self.lookup_variable(name).ok_or_else(|| TypeError {
                            span: target.span.clone(),
                            message: format!(
                                "Undefined variable '{}' — declare it with `var` first",
                                name
                            ),
                        })?;

                        if !self.is_mutable(name) {
                            return self.error(
                                target.span.clone(),
                                format!(
                                    "Cannot reassign '{}' — it was declared with `let` (immutable). Use `var` to allow reassignment.",
                                    name
                                ),
                            );
                        }

                        self.assert_type_eq(&var_ty, &val_ty)
                            .map_err(|msg| TypeError {
                                span: value.span.clone(),
                                message: msg,
                            })?;
                    }
                    auwla_ast::ExprKind::PropertyAccess { expr, property } => {
                        let expr_ty = self.check_expr(expr)?;
                        match expr_ty {
                            Type::Custom(name) => {
                                let struct_def =
                                    self.structs.get(&name).ok_or_else(|| TypeError {
                                        span: target.span.clone(),
                                        message: format!("Undefined struct '{}'", name),
                                    })?;
                                let mut found = false;
                                for (field_name, field_ty) in struct_def.iter() {
                                    if field_name == property {
                                        found = true;
                                        self.assert_type_eq(field_ty, &val_ty).map_err(|_| TypeError {
                                            span: value.span.clone(),
                                            message: format!("Type error: struct '{}' field '{}' expects '{}', but got '{}'", name, property, field_ty, val_ty),
                                        })?;
                                        break;
                                    }
                                }
                                if !found {
                                    return self.error(
                                        target.span.clone(),
                                        format!(
                                            "Type error: struct '{}' has no property '{}'",
                                            name, property
                                        ),
                                    );
                                }
                            }
                            other => {
                                return self.error(
                                    target.span.clone(),
                                    format!(
                                        "Type error: cannot assign property '{}' on non-struct type '{}'",
                                        property, other
                                    ),
                                );
                            }
                        }
                    }
                    auwla_ast::ExprKind::Index { expr, index } => {
                        let expr_ty = self.check_expr(expr)?;
                        let idx_ty = self.check_expr(index)?;
                        self.assert_type_eq(&Type::Basic("number".to_string()), &idx_ty)
                            .map_err(|_| TypeError {
                                span: index.span.clone(),
                                message: format!(
                                    "Type error: array index must be 'number', got '{}'",
                                    idx_ty
                                ),
                            })?;

                        match expr_ty {
                            Type::Array(inner) => {
                                self.assert_type_eq(&inner, &val_ty)
                                    .map_err(|msg| TypeError {
                                        span: value.span.clone(),
                                        message: msg,
                                    })?;
                            }
                            other => {
                                return self.error(
                                    expr.span.clone(),
                                    format!(
                                        "Type error: cannot index into non-array type '{}'",
                                        other
                                    ),
                                );
                            }
                        }
                    }
                    other => {
                        return self.error(
                            target.span.clone(),
                            format!("Type error: invalid assignment target '{:?}'", other),
                        );
                    }
                }
                Ok(())
            }
            auwla_ast::StmtKind::Fn {
                name,
                type_params,
                params,
                return_ty,
                body,
                ..
            } => {
                let param_types = params.iter().map(|(_, ty)| ty.clone()).collect();
                self.declare_function(
                    name.clone(),
                    type_params.clone(),
                    param_types,
                    return_ty.clone(),
                );

                let prev_return = self.current_return_type.take();
                let prev_func_name = self.current_function_name.take();
                self.current_return_type = Some(return_ty.clone());
                self.current_function_name = Some(name.clone());

                self.enter_scope();
                // Fn params are always mutable within their scope
                for (param_name, ty) in params {
                    self.declare_variable(
                        stmt.span.clone(),
                        param_name.clone(),
                        ty.clone(),
                        Mutability::Mutable,
                    )?;
                }
                for body_stmt in body {
                    self.check_stmt(body_stmt)?;
                }
                self.exit_scope();

                self.current_return_type = prev_return;
                self.current_function_name = prev_func_name;
                Ok(())
            }
            auwla_ast::StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let _cond_ty = self.check_expr(condition)?;

                // Condition must strictly evaluate to a boolean expression
                // Currently our language binary expressions evaluate to the LHS type.
                // We will enforce this loosely for now but catch invalid types later when we add Booleans explicitly.
                // self.assert_type_eq(&Type::Basic("bool".to_string()), &cond_ty)?;

                self.enter_scope();
                for stmt in then_branch {
                    self.check_stmt(stmt)?;
                }
                self.exit_scope();

                if let Some(els) = else_branch {
                    self.enter_scope();
                    for stmt in els {
                        self.check_stmt(stmt)?;
                    }
                    self.exit_scope();
                }
                Ok(())
            }
            auwla_ast::StmtKind::Return(expr_opt) => {
                let actual_ty = if let Some(expr) = expr_opt {
                    Some(self.check_expr(expr)?)
                } else {
                    None
                };

                let func_ctx = self.current_function_name.as_deref().unwrap_or("anon");

                if let Some(expected_ty_opt) = &self.current_return_type {
                    match (expected_ty_opt, actual_ty) {
                        (Some(expected), Some(actual)) => {
                            self.assert_type_eq(expected, &actual).map_err(|_| TypeError {
                                span: stmt.span.clone(),
                                message: format!(
                                    "Strict Type error: Function '{}' expects to return '{}', but returned '{}'",
                                    func_ctx, expected, actual
                                ),
                            })?;
                        }
                        (None, Some(actual)) => {
                            if actual != Type::Basic("void".to_string()) {
                                return self.error(
                                    stmt.span.clone(),
                                    format!(
                                        "Strict Type error: Function '{}' expected to return nothing, but returned '{}'",
                                        func_ctx, actual
                                    ),
                                );
                            }
                        }
                        (Some(expected), None) => {
                            return self.error(
                                stmt.span.clone(),
                                format!(
                                    "Strict Type error: Function '{}' expects to return '{}', but returned nothing",
                                    func_ctx, expected
                                ),
                            );
                        }
                        (None, None) => {}
                    }
                } else {
                    return self.error(
                        stmt.span.clone(),
                        "Strict Type error: 'return' statement outside of function",
                    );
                }

                Ok(())
            }
            auwla_ast::StmtKind::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            auwla_ast::StmtKind::While { condition, body } => {
                self.check_expr(condition)?;
                self.enter_scope();
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.exit_scope();
                Ok(())
            }
            auwla_ast::StmtKind::For {
                binding,
                iterable,
                body,
            } => {
                let iter_ty = self.check_expr(iterable)?;
                let elem_ty = match iter_ty {
                    Type::Array(inner) => *inner,
                    other => {
                        return self.error(
                            iterable.span.clone(),
                            format!(
                                "Type error: 'for..in' requires an array or range, but got '{}'",
                                other
                            ),
                        );
                    }
                };
                self.enter_scope();
                self.declare_variable(
                    stmt.span.clone(),
                    binding.clone(),
                    elem_ty,
                    Mutability::Immutable,
                )?;
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.exit_scope();
                Ok(())
            }
            auwla_ast::StmtKind::StructDecl { name, fields, .. } => {
                if self.structs.contains_key(name) {
                    return self.error(
                        stmt.span.clone(),
                        format!("Struct '{}' is already defined", name),
                    );
                }
                self.structs.insert(name.clone(), fields.clone());
                Ok(())
            }
            auwla_ast::StmtKind::TypeAlias {
                name, aliased_type, ..
            } => {
                self.type_aliases.insert(name.clone(), aliased_type.clone());
                Ok(())
            }
            auwla_ast::StmtKind::EnumDecl { name, variants, .. } => {
                if self.enums.contains_key(name) {
                    return self.error(
                        stmt.span.clone(),
                        format!("Enum '{}' is already defined", name),
                    );
                }
                self.enums.insert(name.clone(), variants.clone());
                Ok(())
            }
            // Imports are pre-resolved in check_program_with_imports before check_stmt is called.
            auwla_ast::StmtKind::Import { .. } => Ok(()),
            // Export is transparent — the inner stmt is what matters for type-checking.
            auwla_ast::StmtKind::Export { stmt: inner } => self.check_stmt(inner),
            auwla_ast::StmtKind::Extend {
                type_name, methods, ..
            } => {
                let self_type = match type_name.as_str() {
                    "number" | "string" | "boolean" => Type::Basic(type_name.clone()),
                    _ => Type::Custom(type_name.clone()),
                };
                let mut method_sigs = Vec::new();
                for method in methods {
                    // Build fully-typed params: inject self type for instance methods
                    let full_params: Vec<(String, Type)> = method
                        .params
                        .iter()
                        .map(|(n, ty_opt)| {
                            let t = if n == "self" {
                                self_type.clone()
                            } else {
                                ty_opt.clone().unwrap_or(Type::Basic("unknown".to_string()))
                            };
                            (n.clone(), t)
                        })
                        .collect();

                    method_sigs.push((
                        method.type_params.clone(),
                        method.name.clone(),
                        method.is_static,
                        full_params.clone(),
                        method.return_ty.clone(),
                    ));

                    // Type-check the method body
                    self.enter_scope();
                    let saved_return = self.current_return_type.take();
                    let saved_fn = self.current_function_name.take();
                    self.current_return_type = Some(method.return_ty.clone());
                    self.current_function_name = Some(format!("{}::{}", type_name, method.name));
                    for (pname, pty) in &full_params {
                        self.declare_variable(
                            stmt.span.clone(),
                            pname.clone(),
                            pty.clone(),
                            Mutability::Immutable,
                        )?;
                    }
                    for s in &method.body {
                        self.check_stmt(s)?;
                    }
                    self.current_return_type = saved_return;
                    self.current_function_name = saved_fn;
                    self.exit_scope();
                }
                self.extensions
                    .entry(type_name.clone())
                    .or_default()
                    .extend(method_sigs);
                Ok(())
            }
        }
    }
}
