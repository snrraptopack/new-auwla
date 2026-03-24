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
                let init_ty = self.check_expr_expected(initializer, ty.as_ref())?;
                let final_ty = if let Some(declared_ty) = ty {
                    self.assert_type_eq(declared_ty, &init_ty)
                        .map_err(|msg| TypeError {
                            span: initializer.span.clone(),
                            message: msg,
                        })?;
                    declared_ty.clone()
                } else {
                    if self.contains_unknown(&init_ty) {
                        return Err(TypeError {
                            span: initializer.span.clone(),
                            message: format!(
                                "Type error: type must be explicitly annotated for empty collection, found '{}'",
                                init_ty
                            ),
                        });
                    }
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
                let resolved_init = self.resolve_type(&init_ty);

                match resolved_init {
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
            auwla_ast::StmtKind::TupleDestructureLet {
                bindings,
                initializer,
            } => {
                let init_ty = self.check_expr(initializer)?;
                let resolved_init = self.resolve_type(&init_ty);

                match resolved_init {
                    Type::Tuple(types) => {
                        if bindings.len() != types.len() {
                            return self.error(
                                initializer.span.clone(),
                                format!(
                                    "Type error: tuple has {} elements but {} bindings were provided",
                                    types.len(),
                                    bindings.len()
                                ),
                            );
                        }

                        for (binding, ty) in bindings.iter().zip(types.iter()) {
                            self.declare_variable(
                                stmt.span.clone(),
                                binding.clone(),
                                ty.clone(),
                                Mutability::Immutable,
                            )?;
                        }
                    }
                    _ => {
                        return self.error(
                            initializer.span.clone(),
                            format!(
                                "Type error: expected tuple for destructuring, found '{}'",
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
                let init_ty = self.check_expr_expected(initializer, ty.as_ref())?;
                let final_ty = if let Some(declared_ty) = ty {
                    self.assert_type_eq(declared_ty, &init_ty)
                        .map_err(|msg| TypeError {
                            span: initializer.span.clone(),
                            message: msg,
                        })?;
                    declared_ty.clone()
                } else {
                    if self.contains_unknown(&init_ty) {
                        return Err(TypeError {
                            span: initializer.span.clone(),
                            message: format!(
                                "Type error: type must be explicitly annotated for empty collection, found '{}'",
                                init_ty
                            ),
                        });
                    }
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
                let target_ty = self.check_expr(target)?;
                let val_ty = self.check_expr_expected(value, Some(&target_ty))?;

                match &target.node {
                    auwla_ast::ExprKind::Identifier(name) => {
                        let var_ty = self.resolve_type(&target_ty); // Use resolved target_ty

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
                        let resolved_expr = self.resolve_type(&expr_ty);
                        match resolved_expr {
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
                        let resolved_expr = self.resolve_type(&expr_ty);

                        match resolved_expr {
                            Type::Array(inner) => {
                                self.assert_type_eq(&Type::Basic("number".to_string()), &idx_ty)
                                    .map_err(|_| TypeError {
                                        span: index.span.clone(),
                                        message: format!(
                                            "Type error: array index must be 'number', got '{}'",
                                            idx_ty
                                        ),
                                    })?;
                                self.assert_type_eq(&inner, &val_ty)
                                    .map_err(|msg| TypeError {
                                        span: value.span.clone(),
                                        message: msg,
                                    })?;
                            }
                            Type::Dict(k, v) => {
                                self.assert_type_eq(&k, &idx_ty)
                                    .map_err(|_| TypeError {
                                        span: index.span.clone(),
                                        message: format!(
                                            "Type error: dict index must be '{}', got '{}'",
                                            self.type_to_key(&k), self.type_to_key(&idx_ty)
                                        ),
                                    })?;
                                self.assert_type_eq(&v, &val_ty)
                                    .map_err(|msg| TypeError {
                                        span: value.span.clone(),
                                        message: msg,
                                    })?;
                            }
                            other => {
                                return self.error(
                                    expr.span.clone(),
                                    format!(
                                        "Type error: cannot index into non-array/dict type '{}'",
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
            auwla_ast::StmtKind::CompoundAssign { target, op, value } => {
                let target_ty = self.check_expr(target)?;
                let val_ty = self.check_expr_expected(value, Some(&target_ty))?;

                // Check mutability
                match &target.node {
                    auwla_ast::ExprKind::Identifier(name) => {
                        if !self.is_mutable(name) {
                            return self.error(
                                target.span.clone(),
                                format!(
                                    "Cannot reassign '{}' using compound assignment — it was declared with `let` (immutable). Use `var` to allow reassignment.",
                                    name
                                ),
                            );
                        }
                    }
                    auwla_ast::ExprKind::PropertyAccess { .. } | auwla_ast::ExprKind::Index { .. } => {
                        // Properties and indices are assumed mutable in Auwla for now
                    }
                    _ => {
                        return self.error(
                            target.span.clone(),
                            "Type error: invalid compound assignment target".to_string(),
                        );
                    }
                }

                // Verify types based on operator
                match op {
                    auwla_ast::BinaryOp::Add => {
                        // Allow number += number or string += string
                        if target_ty == Type::Basic("string".to_string()) {
                            self.assert_type_eq(&Type::Basic("string".to_string()), &val_ty)
                                .map_err(|msg| TypeError {
                                    span: value.span.clone(),
                                    message: msg,
                                })?;
                        } else {
                            self.assert_type_eq(&Type::Basic("number".to_string()), &target_ty)
                                .map_err(|_| TypeError {
                                    span: target.span.clone(),
                                    message: format!(
                                        "Compound assignment '+=' requires numeric or string target, got '{}'",
                                        target_ty
                                    ),
                                })?;
                            self.assert_type_eq(&Type::Basic("number".to_string()), &val_ty)
                                .map_err(|_| TypeError {
                                    span: value.span.clone(),
                                    message: format!(
                                        "Compound assignment '+=' requires numeric value for numeric target, got '{}'",
                                        val_ty
                                    ),
                                })?;
                        }
                    }
                    auwla_ast::BinaryOp::Sub
                    | auwla_ast::BinaryOp::Mul
                    | auwla_ast::BinaryOp::Div
                    | auwla_ast::BinaryOp::Mod => {
                        self.assert_type_eq(&Type::Basic("number".to_string()), &target_ty)
                            .map_err(|_| TypeError {
                                span: target.span.clone(),
                                message: format!(
                                    "Compound assignment '{}=' requires numeric target, got '{}'",
                                    op, target_ty
                                ),
                            })?;
                        self.assert_type_eq(&Type::Basic("number".to_string()), &val_ty)
                            .map_err(|_| TypeError {
                                span: value.span.clone(),
                                message: format!(
                                    "Compound assignment '{}=' requires numeric value, got '{}'",
                                    op, val_ty
                                ),
                            })?;
                    }
                    _ => {
                        // Fallback: strict equality
                        self.assert_type_eq(&target_ty, &val_ty)
                            .map_err(|msg| TypeError {
                                span: value.span.clone(),
                                message: msg,
                            })?;
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
                let mut all_tps = Vec::new();
                if let Some(tps) = type_params.as_ref() {
                    all_tps.extend(tps.clone());
                }

                let param_types: Vec<(Type, bool)> = params
                    .iter()
                    .map(|(_, ty, is_v)| (self.genericize_type(ty, &all_tps), *is_v))
                    .collect();
                let return_ty_gen = return_ty
                    .as_ref()
                    .map(|ty| self.genericize_type(ty, &all_tps));

                let signature = self.register_function(
                    name.clone(),
                    type_params.clone(),
                    param_types.clone(),
                    return_ty_gen.clone(),
                );

                self.declare_function(stmt.span.clone(), name.clone(), signature);

                let prev_return = self.current_return_type.take();
                let prev_func_name = self.current_function_name.take();
                self.current_return_type = Some(return_ty_gen.clone());
                self.current_function_name = Some(name.clone());

                self.enter_scope();
                // Fn params are always immutable within their scope
                for ((param_name, _, _), (ty, _)) in params.iter().zip(param_types) {
                    self.declare_variable(
                        stmt.span.clone(),
                        param_name.clone(),
                        ty.clone(),
                        Mutability::Immutable,
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
                let expected_ty = self.current_return_type.as_ref().and_then(|t| t.as_ref()).cloned();
                let actual_ty = if let Some(expr) = expr_opt {
                    Some(self.check_expr_expected(expr, expected_ty.as_ref())?)
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
                bindings,
                iterable,
                step,
                body,
            } => {
                let iter_ty = self.check_expr(iterable)?;
                if let Some(s_expr) = step {
                    let s_ty = self.check_expr(s_expr)?;
                    self.assert_type_eq(&Type::Basic("number".to_string()), &s_ty)
                        .map_err(|msg| TypeError { span: s_expr.span.clone(), message: msg })?;
                }
                let (key_ty, val_ty) = match iter_ty {
                    Type::Array(inner) => (None, Some(*inner)),
                    Type::Basic(name) if name == "string" => {
                        (None, Some(Type::Basic("char".to_string())))
                    }
                    Type::Dict(k, v) => (Some(*k), Some(*v)),
                    other => {
                        return self.error(
                            iterable.span.clone(),
                            format!(
                                "Type error: 'for..in' requires an array, range, string, or dict, but got '{}'",
                                other
                            ),
                        );
                    }
                };

                self.enter_scope();
                if bindings.len() == 1 {
                    if let Some(v_ty) = val_ty {
                        self.declare_variable(
                            stmt.span.clone(),
                            bindings[0].clone(),
                            v_ty,
                            Mutability::Immutable,
                        )?;
                    } else if let Some(k_ty) = key_ty {
                        self.declare_variable(
                            stmt.span.clone(),
                            bindings[0].clone(),
                            k_ty,
                            Mutability::Immutable,
                        )?;
                    }
                } else if bindings.len() == 2 {
                    if let (Some(k_ty), Some(v_ty)) = (key_ty, val_ty) {
                        self.declare_variable(
                            stmt.span.clone(),
                            bindings[0].clone(),
                            k_ty,
                            Mutability::Immutable,
                        )?;
                        self.declare_variable(
                            stmt.span.clone(),
                            bindings[1].clone(),
                            v_ty,
                            Mutability::Immutable,
                        )?;
                    } else {
                        return self.error(
                            stmt.span.clone(),
                            "Type error: 'for (k, v)' destructuring only supported for dictionaries"
                                .to_string(),
                        );
                    }
                } else {
                    return self.error(
                        stmt.span.clone(),
                        "Type error: too many bindings in 'for' loop".to_string(),
                    );
                }
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.exit_scope();
                Ok(())
            }
            auwla_ast::StmtKind::StructDecl {
                name,
                fields,
                attributes,
                ..
            } => {
                if self.structs.contains_key(name) {
                    return self.error(
                        stmt.span.clone(),
                        format!("Struct '{}' is already defined", name),
                    );
                }
                self.structs.insert(name.clone(), fields.clone());
                self.definitions.insert(name.clone(), stmt.span.clone());
                self.type_attributes
                    .insert(name.clone(), attributes.clone());
                Ok(())
            }
            auwla_ast::StmtKind::TypeAlias {
                name,
                type_params,
                aliased_type,
            } => {
                self.type_aliases.insert(name.clone(), aliased_type.clone());
                if let Some(params) = type_params {
                    self.type_alias_params.insert(name.clone(), params.clone());
                }
                Ok(())
            }
            auwla_ast::StmtKind::EnumDecl {
                name,
                variants,
                attributes,
                ..
            } => {
                if self.enums.contains_key(name) {
                    return self.error(
                        stmt.span.clone(),
                        format!("Enum '{}' is already defined", name),
                    );
                }
                self.enums.insert(name.clone(), variants.clone());
                self.definitions.insert(name.clone(), stmt.span.clone());
                self.type_attributes
                    .insert(name.clone(), attributes.clone());
                Ok(())
            }
            // Imports are pre-resolved in check_program_with_imports before check_stmt is called.
            auwla_ast::StmtKind::Import { .. } => Ok(()),
            // Export is transparent — the inner stmt is what matters for type-checking.
            auwla_ast::StmtKind::Export { stmt: inner } => self.check_stmt(inner),
            auwla_ast::StmtKind::Extend {
                type_params,
                target_type,
                methods,
            } => {
                // Collect all type-param names so we can genericise the target
                let mut base_tps: Vec<String> = Vec::new();
                if let Some(tps) = type_params.as_ref() {
                    base_tps.extend(tps.clone());
                }

                // The `self_type` is the target with TypeVars already baked in
                // (the parser already converted single-upper Custom → TypeVar).
                // We just need to genericize it with respect to base_tps so that
                // TypeVar("T") is preserved correctly during method registration.
                let self_type = self.genericize_type(target_type, &base_tps);

                // Derive the extension registry base key from the self type.
                // e.g. Optional<T> / number? / T?E all map to their respective base keys ("optional", "result")
                let ext_key = self_type.base_key();

                // Derive a display name for error messages
                let type_display = format!("{}", self_type);

                let mut method_sigs = Vec::new();
                let mut method_infos: Vec<(&auwla_ast::Method, Vec<(String, Type, bool)>, Option<Type>)> =
                    Vec::new();
                for method in methods {
                    let mut method_tps = base_tps.clone();
                    if let Some(mtps) = method.type_params.as_ref() {
                        method_tps.extend(mtps.clone());
                    }

                    let full_params: Vec<(String, Type, bool)> = method
                        .params
                        .iter()
                        .map(|(n, ty_opt, is_vararg)| {
                            let t = if n == "self" {
                                if let Some(explicit_ty) = ty_opt {
                                    let generic_ty = self.genericize_type(explicit_ty, &method_tps);
                                    self.resolve_self_type(&generic_ty, &self_type)
                                } else {
                                    self_type.clone()
                                }
                            } else {
                                let ty =
                                    ty_opt.clone().unwrap_or(Type::Basic("unknown".to_string()));
                                let generic_ty = self.genericize_type(&ty, &method_tps);
                                self.resolve_self_type(&generic_ty, &self_type)
                            };
                            (n.clone(), t, *is_vararg)
                        })
                        .collect();

                    let return_ty_gen = method.return_ty.as_ref().map(|ty| {
                        let generic_ty = self.genericize_type(ty, &method_tps);
                        self.resolve_self_type(&generic_ty, &self_type)
                    });

                    let combined_tps = if method_tps.is_empty() {
                        None
                    } else {
                        Some(method_tps.clone())
                    };

                    method_sigs.push(auwla_ast::ExtensionMethod {
                        type_params: combined_tps,
                        name: method.name.clone(),
                        is_static: method.is_static,
                        params: full_params.clone(),
                        return_ty: return_ty_gen.clone(),
                        attributes: method.attributes.clone(),
                        span: method.span.clone(),
                        origin: Default::default(),
                    });
                    method_infos.push((method, full_params, return_ty_gen));
                }
                self.extensions
                    .entry(ext_key)
                    .or_default()
                    .extend(method_sigs);
                for (method, full_params, return_ty_gen) in method_infos {
                    self.enter_scope();
                    let saved_return = self.current_return_type.take();
                    let saved_fn = self.current_function_name.take();
                    self.current_return_type = Some(return_ty_gen);
                    self.current_function_name =
                        Some(format!("{}::{}", type_display, method.name));
                    for (pname, pty, _) in &full_params {
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
                Ok(())
            }
            auwla_ast::StmtKind::TypeDecl {
                name,
                attributes,
                methods,
                ..
            } => {
                self.type_attributes
                    .insert(name.clone(), attributes.clone());
                let mut method_sigs = Vec::new();
                let mut method_infos: Vec<(&auwla_ast::Method, Vec<(String, Type, bool)>, Option<Type>)> =
                    Vec::new();
                for method in methods {
                    let custom_type = Type::Custom(name.clone());
                    let full_params: Vec<(String, Type, bool)> = method
                        .params
                        .iter()
                        .map(|(n, ty_opt, is_vararg)| {
                            let ty = ty_opt.clone().unwrap_or(Type::Basic("unknown".to_string()));
                            let resolved = self.resolve_self_type(&ty, &custom_type);
                            (n.clone(), resolved, *is_vararg)
                        })
                        .collect();

                    let ret = method
                        .return_ty
                        .as_ref()
                        .map(|t| self.resolve_self_type(t, &custom_type));
                    method_sigs.push(auwla_ast::ExtensionMethod {
                        type_params: method.type_params.clone(),
                        name: method.name.clone(),
                        is_static: method.is_static,
                        params: full_params.clone(),
                        return_ty: ret.clone(),
                        attributes: method.attributes.clone(),
                        span: method.span.clone(),
                        origin: Default::default(),
                    });
                    method_infos.push((method, full_params, ret));
                }
                self.extensions.insert(name.clone(), method_sigs);

                for (method, full_params, ret) in method_infos {
                    // Ambient methods don't need body checking if they are @external
                    if self.has_attribute(&method.attributes, "external", None) {
                        continue;
                    }

                    self.enter_scope();
                    let saved_return = self.current_return_type.take();
                    let saved_fn = self.current_function_name.take();
                    self.current_return_type = Some(ret);
                    self.current_function_name = Some(format!("{}::{}", name, method.name));
                    for (pname, pty, _) in &full_params {
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
                Ok(())
            }
        }
    }
}
