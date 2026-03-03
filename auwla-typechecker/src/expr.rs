use crate::TypeError;
use crate::checker::Typechecker;
use crate::scope::Mutability;
use auwla_ast::{Expr, Type};

impl Typechecker {
    pub fn check_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        let ty = self.check_expr_internal(expr)?;
        self.node_types.insert(expr.span.clone(), ty.clone());
        Ok(ty)
    }

    fn check_expr_internal(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match &expr.node {
            auwla_ast::ExprKind::Void => Ok(Type::Basic("void".to_string())),
            auwla_ast::ExprKind::BoolLit(_) => Ok(Type::Basic("bool".to_string())),
            auwla_ast::ExprKind::StringLit(_) => Ok(Type::Basic("string".to_string())),
            auwla_ast::ExprKind::NumberLit(_) => Ok(Type::Basic("number".to_string())),
            auwla_ast::ExprKind::CharLit(_) => Ok(Type::Basic("char".to_string())),
            auwla_ast::ExprKind::Identifier(name) => {
                self.lookup_variable(name).ok_or_else(|| TypeError {
                    span: expr.span.clone(),
                    message: format!("Undefined variable: '{}'", name),
                })
            }
            auwla_ast::ExprKind::Binary { op, left, right } => {
                let left_ty = self.check_expr(left)?;
                let right_ty = self.check_expr(right)?;

                self.assert_type_eq(&left_ty, &right_ty).map_err(|_| TypeError {
                    span: expr.span.clone(),
                    message: format!(
                        "Strict Type inferred error: Operators must have matching types. Left: {}, Right: {}",
                        left_ty, right_ty
                    ),
                })?;

                // Return boolean for comparative ops, otherwise return the evaluated mathematical type (number/string)
                match op {
                    auwla_ast::BinaryOp::Eq
                    | auwla_ast::BinaryOp::Neq
                    | auwla_ast::BinaryOp::Lt
                    | auwla_ast::BinaryOp::Gt
                    | auwla_ast::BinaryOp::Lte
                    | auwla_ast::BinaryOp::Gte => Ok(Type::Basic("bool".to_string())),
                    _ => Ok(left_ty),
                }
            }
            // If it evaluates `some()`, it wraps the OK branch.
            auwla_ast::ExprKind::Some(inner) => {
                let inner_ty = self.check_expr(inner)?;
                Ok(Type::Result {
                    ok_type: Box::new(inner_ty),
                    err_type: Box::new(Type::Basic("unknown".to_string())),
                })
            }
            auwla_ast::ExprKind::None(inner_opt) => {
                if let Some(inner) = inner_opt {
                    let inner_ty = self.check_expr(inner)?;
                    Ok(Type::Result {
                        ok_type: Box::new(Type::Basic("unknown".to_string())),
                        err_type: Box::new(inner_ty),
                    })
                } else {
                    Ok(Type::Optional(Box::new(Type::Basic("unknown".to_string()))))
                }
            }
            auwla_ast::ExprKind::Call {
                name,
                args,
                type_args,
            } => {
                // Built-in functions
                if name == "print" {
                    // print() accepts any number of arguments of any type
                    for arg in args {
                        self.check_expr(arg)?;
                    }
                    return Ok(Type::Basic("void".to_string()));
                }

                let (type_params, params, return_ty) = if let Some(var_ty) =
                    self.lookup_variable(name)
                {
                    if let Type::Function(p, r) = var_ty {
                        (None, p, Some(*r))
                    } else {
                        return Err(TypeError {
                            span: expr.span.clone(),
                            message: format!("Type error: variable '{}' is not a function", name),
                        });
                    }
                } else {
                    self.lookup_function(name).ok_or_else(|| TypeError {
                        span: expr.span.clone(),
                        message: format!("Undefined function: '{}'", name),
                    })?
                };

                let mut unifier = crate::inference::unify::Unifier::new();

                // 1. If the function is generic, create InferenceVars for its type parameters
                let mut type_env = std::collections::HashMap::new();
                if let Some(t_params) = type_params {
                    if let Some(t_args) = type_args {
                        if t_args.len() != t_params.len() {
                            return Err(TypeError {
                                span: expr.span.clone(),
                                message: format!(
                                    "Function '{}' expects {} type arguments, but {} were provided",
                                    name,
                                    t_params.len(),
                                    t_args.len()
                                ),
                            });
                        }
                        for (tp, ta) in t_params.iter().zip(t_args) {
                            let id = unifier.new_type_var();
                            unifier.bind(id, ta).map_err(|msg| TypeError {
                                span: expr.span.clone(),
                                message: msg,
                            })?;
                            type_env.insert(tp.clone(), id);
                        }
                    } else {
                        for tp in t_params {
                            let id = unifier.new_type_var();
                            type_env.insert(tp.clone(), id);
                        }
                    }
                } else if type_args.is_some() {
                    return self.error(
                        expr.span.clone(),
                        format!(
                            "Function '{}' is not generic but type arguments were provided",
                            name
                        ),
                    );
                }

                // Helper to completely instantiate a type from the signature into unification variables
                fn instantiate(ty: &Type, env: &std::collections::HashMap<String, usize>) -> Type {
                    match ty {
                        Type::TypeVar(name) | Type::Custom(name) => {
                            if let Some(&id) = env.get(name) {
                                Type::InferenceVar(id)
                            } else {
                                ty.clone()
                            }
                        }
                        Type::Array(inner) => Type::Array(Box::new(instantiate(inner, env))),
                        Type::Optional(inner) => Type::Optional(Box::new(instantiate(inner, env))),
                        Type::Result { ok_type, err_type } => Type::Result {
                            ok_type: Box::new(instantiate(ok_type, env)),
                            err_type: Box::new(instantiate(err_type, env)),
                        },
                        Type::Function(p, r) => {
                            let inst_p = p.iter().map(|p| instantiate(p, env)).collect();
                            let inst_r = Box::new(instantiate(r, env));
                            Type::Function(inst_p, inst_r)
                        }
                        Type::Generic(n, args) => {
                            let inst_args = args.iter().map(|a| instantiate(a, env)).collect();
                            Type::Generic(n.clone(), inst_args)
                        }
                        _ => ty.clone(),
                    }
                }

                let inst_params: Vec<Type> =
                    params.iter().map(|p| instantiate(p, &type_env)).collect();
                let inst_return_ty = return_ty.map(|r| instantiate(&r, &type_env));

                if inst_params.len() != args.len() {
                    return Err(TypeError {
                        span: expr.span.clone(),
                        message: format!(
                            "Function '{}' expects {} arguments, but {} were provided",
                            name,
                            inst_params.len(),
                            args.len()
                        ),
                    });
                }

                for (param_ty, arg_expr) in inst_params.iter().zip(args) {
                    let arg_ty = self.check_expr(arg_expr)?;
                    // Unify the passed argument with the generic parameter signature!
                    unifier.unify(param_ty, &arg_ty).map_err(|msg| TypeError {
                        span: arg_expr.span.clone(),
                        message: msg,
                    })?;
                }

                // Resolve the return type with all unified variables bound
                let resolved_return = inst_return_ty
                    .map(|r| unifier.resolve(&r))
                    .unwrap_or(Type::Basic("void".to_string()));

                Ok(resolved_return)
            }
            auwla_ast::ExprKind::Unary { op, expr } => {
                let inner = self.check_expr(expr)?;
                match op {
                    auwla_ast::UnaryOp::Not => {
                        self.assert_type_eq(&Type::Basic("bool".to_string()), &inner)
                            .map_err(|msg| TypeError {
                                span: expr.span.clone(),
                                message: msg,
                            })?;
                        Ok(Type::Basic("bool".to_string()))
                    }
                    auwla_ast::UnaryOp::Neg => {
                        self.assert_type_eq(&Type::Basic("number".to_string()), &inner)
                            .map_err(|msg| TypeError {
                                span: expr.span.clone(),
                                message: msg,
                            })?;
                        Ok(Type::Basic("number".to_string()))
                    }
                }
            }
            auwla_ast::ExprKind::Match { expr, arms } => {
                let result_ty = self.check_expr(expr)?;
                let mut common_return_ty: Option<Type> = None;

                fn collect_variants<'a>(
                    pattern: &'a auwla_ast::Pattern,
                    variants: &mut Vec<&'a auwla_ast::Pattern>,
                ) {
                    match &pattern.node {
                        auwla_ast::PatternKind::Or(patterns) => {
                            for p in patterns {
                                collect_variants(p, variants);
                            }
                        }
                        _ => variants.push(pattern),
                    }
                }

                let mut has_wildcard = false;
                let mut handled_variants = std::collections::HashSet::new();
                let mut has_some = false;
                let mut has_none = false;

                let is_primitive = matches!(result_ty, Type::Basic(ref s) if s == "string" || s == "number" || s == "bool" || s == "char");

                let enum_def = if let Type::Custom(ref enum_name) = result_ty {
                    self.enums.get(enum_name).cloned()
                } else {
                    None
                };

                for arm in arms {
                    let mut arm_yields = Type::Basic("void".to_string());

                    let mut sub_patterns = Vec::new();
                    collect_variants(&arm.pattern, &mut sub_patterns);

                    self.enter_scope();

                    for (i, p) in sub_patterns.iter().enumerate() {
                        match &p.node {
                            auwla_ast::PatternKind::Wildcard => {
                                if arm.guard.is_none() {
                                    has_wildcard = true;
                                }
                            }
                            auwla_ast::PatternKind::Variable(name) => {
                                if i == 0 {
                                    self.declare_variable(
                                        p.span.clone(),
                                        name.clone(),
                                        result_ty.clone(),
                                        Mutability::Immutable,
                                    )?;
                                }
                                if arm.guard.is_none() {
                                    has_wildcard = true;
                                }
                            }
                            auwla_ast::PatternKind::Literal(lit_expr) => {
                                let lit_ty = self.check_expr(lit_expr)?;
                                self.assert_type_eq(&result_ty, &lit_ty).map_err(|msg| {
                                    TypeError {
                                        span: lit_expr.span.clone(),
                                        message: msg,
                                    }
                                })?;
                            }
                            auwla_ast::PatternKind::Range {
                                start,
                                end,
                                inclusive: _,
                            } => {
                                let start_ty = self.check_expr(start)?;
                                let end_ty = self.check_expr(end)?;
                                self.assert_type_eq(&start_ty, &end_ty).map_err(|msg| {
                                    TypeError {
                                        span: end.span.clone(),
                                        message: msg,
                                    }
                                })?;
                                self.assert_type_eq(&result_ty, &start_ty).map_err(|msg| {
                                    TypeError {
                                        span: start.span.clone(),
                                        message: msg,
                                    }
                                })?;
                            }
                            auwla_ast::PatternKind::Variant { name, bindings } => {
                                if let Type::Result {
                                    ok_type,
                                    err_type: _,
                                } = &result_ty
                                {
                                    if name == "some" {
                                        has_some = true;
                                        if i == 0 {
                                            if let Some(binding) = bindings.first() {
                                                self.declare_variable(
                                                    p.span.clone(),
                                                    binding.clone(),
                                                    *ok_type.clone(),
                                                    Mutability::Immutable,
                                                )?;
                                            }
                                        }
                                    } else if name == "none" {
                                        has_none = true;
                                        if let Type::Result { err_type, .. } = &result_ty {
                                            if i == 0 {
                                                if let Some(binding) = bindings.first() {
                                                    self.declare_variable(
                                                        p.span.clone(),
                                                        binding.clone(),
                                                        *err_type.clone(),
                                                        Mutability::Immutable,
                                                    )?;
                                                }
                                            }
                                        } else {
                                            if !bindings.is_empty() {
                                                return Err(TypeError {
                                                    span: p.span.clone(),
                                                    message: "Type error: match arm 'none' for Optional type cannot bind arguments".to_string(),
                                                });
                                            }
                                        }
                                    } else {
                                        return self.error(
                                            p.span.clone(),
                                            format!(
                                                "Type error: matching on a Result expects 'some' or 'none', found '{}'",
                                                name
                                            ),
                                        );
                                    }
                                } else if let Type::Optional(inner) = &result_ty {
                                    if name == "some" {
                                        has_some = true;
                                        if i == 0 {
                                            if let Some(binding) = bindings.first() {
                                                self.declare_variable(
                                                    p.span.clone(),
                                                    binding.clone(),
                                                    (**inner).clone(),
                                                    Mutability::Immutable,
                                                )?;
                                            }
                                        }
                                    } else if name == "none" {
                                        has_none = true;
                                        if !bindings.is_empty() {
                                            return Err(TypeError {
                                                span: p.span.clone(),
                                                message: "Type error: match arm 'none' for Optional type cannot bind arguments".to_string(),
                                            });
                                        }
                                    } else {
                                        return self.error(
                                            p.span.clone(),
                                            format!(
                                                "Type error: matching on an Optional expects 'some' or 'none', found '{}'",
                                                name
                                            ),
                                        );
                                    }
                                } else if let Some(ref def) = enum_def {
                                    if arm.guard.is_none() {
                                        handled_variants.insert(name.clone());
                                    }
                                    let variant_args = def
                                        .iter()
                                        .find(|(n, _)| n == name)
                                        .ok_or_else(|| TypeError {
                                            span: p.span.clone(),
                                            message: format!(
                                                "Type error: variant '{}' does not exist",
                                                name
                                            ),
                                        })?
                                        .1
                                        .clone();

                                    if bindings.len() != variant_args.len() {
                                        return self.error(
                                            p.span.clone(),
                                            format!(
                                                "Type error: match arm '{}' binds {} arguments, but variant has {}",
                                                name,
                                                bindings.len(),
                                                variant_args.len()
                                            ),
                                        );
                                    }

                                    if i == 0 {
                                        for (binding, expected_ty) in
                                            bindings.iter().zip(variant_args)
                                        {
                                            self.declare_variable(
                                                p.span.clone(),
                                                binding.clone(),
                                                expected_ty,
                                                Mutability::Immutable,
                                            )?;
                                        }
                                    }
                                } else {
                                    return self.error(
                                        p.span.clone(),
                                        format!(
                                            "Type error: cannot match variant '{}' on type '{}'",
                                            name, result_ty
                                        ),
                                    );
                                }
                            }
                            auwla_ast::PatternKind::Struct(opt_name, fields) => {
                                match &result_ty {
                                    Type::Custom(struct_name) => {
                                        if let Some(name) = opt_name {
                                            if name != struct_name {
                                                return self.error(
                                                    p.span.clone(),
                                                    format!(
                                                        "Type error: expected struct '{}', found '{}'",
                                                        struct_name, name
                                                    ),
                                                );
                                            }
                                        }

                                        let struct_def =
                                            self.structs.get(struct_name).cloned().ok_or_else(
                                                || TypeError {
                                                    span: p.span.clone(),
                                                    message: format!(
                                                        "Type error: struct '{}' not found",
                                                        struct_name
                                                    ),
                                                },
                                            )?;

                                        for (field_name, sub_pattern_opt) in fields {
                                            let field_ty = struct_def
                                                .iter()
                                                .find(|(f, _)| f == field_name)
                                                .map(|(_, t)| t.clone())
                                                .ok_or_else(|| TypeError {
                                                    span: p.span.clone(),
                                                    message: format!("Type error: field '{}' not found on struct '{}'", field_name, struct_name),
                                                })?;

                                            if let Some(sub_pattern) = sub_pattern_opt {
                                                // If there's a nested pattern, we'd need to recursively typecheck it.
                                                // For now, if it's a variable binding, declare it in scope.
                                                if let auwla_ast::PatternKind::Variable(var_name) =
                                                    &sub_pattern.node
                                                {
                                                    if i == 0 {
                                                        self.declare_variable(
                                                            sub_pattern.span.clone(),
                                                            var_name.clone(),
                                                            field_ty,
                                                            Mutability::Immutable,
                                                        )?;
                                                    }
                                                }
                                            } else {
                                                // Shorthand: `{ role }` acts as `{ role: role }` binding a variable
                                                if i == 0 {
                                                    self.declare_variable(
                                                        p.span.clone(),
                                                        field_name.clone(),
                                                        field_ty,
                                                        Mutability::Immutable,
                                                    )?;
                                                }
                                            }
                                        }

                                        if arm.guard.is_none() {
                                            has_wildcard = true; // Struct patterns are considered exhaustive if shape matches (no full data exhaustiveness yet)
                                        }
                                    }
                                    _ => {
                                        let display_name = opt_name
                                            .clone()
                                            .unwrap_or_else(|| "anonymous struct".to_string());
                                        return self.error(
                                            p.span.clone(),
                                            format!(
                                                "Type error: cannot match struct '{}' on type '{}'",
                                                display_name, result_ty
                                            ),
                                        );
                                    }
                                }
                            }
                            auwla_ast::PatternKind::Or(_) => unreachable!(),
                        }
                    }

                    if let Some(ref guard) = arm.guard {
                        let guard_ty = self.check_expr(guard)?;
                        self.assert_type_eq(&Type::Basic("bool".to_string()), &guard_ty)
                            .map_err(|msg| TypeError {
                                span: guard.span.clone(),
                                message: msg,
                            })?;
                    }

                    for stmt in &arm.stmts {
                        self.check_stmt(stmt)?;
                    }
                    if let Some(res) = &arm.result {
                        arm_yields = self.check_expr(res)?;
                    }
                    self.exit_scope();

                    if let Some(ref prev_ty) = common_return_ty {
                        self.assert_type_eq(prev_ty, &arm_yields)
                            .map_err(|msg| TypeError {
                                span: arm
                                    .result
                                    .as_ref()
                                    .map(|r| r.span.clone())
                                    .unwrap_or_else(|| expr.span.clone()),
                                message: msg,
                            })?;
                    } else {
                        common_return_ty = Some(arm_yields);
                    }
                }

                if is_primitive && !has_wildcard {
                    return Err(TypeError {
                        span: expr.span.clone(),
                        message: format!(
                            "Type error: match on '{}' primitive must have a wildcard '_' arm for exhaustiveness",
                            result_ty
                        ),
                    });
                }

                if let Type::Result { .. } = &result_ty {
                    if !has_wildcard && (!has_some || !has_none) {
                        return Err(TypeError {
                            span: expr.span.clone(),
                            message: "Type error: match on Result must be exhaustive (handle 'some' and 'none' or use '_')".to_string(),
                        });
                    }
                }

                if let Some(def) = enum_def {
                    if !has_wildcard {
                        let defined_variants: std::collections::HashSet<_> =
                            def.iter().map(|(n, _)| n.clone()).collect();
                        let missing: Vec<_> =
                            defined_variants.difference(&handled_variants).collect();
                        if !missing.is_empty() {
                            return Err(TypeError {
                                span: expr.span.clone(),
                                message: format!(
                                    "Type error: match on enum is not exhaustive. Missing matching arms for variants: {:?}",
                                    missing
                                ),
                            });
                        }
                    }
                }

                Ok(common_return_ty.unwrap_or_else(|| Type::Basic("void".to_string())))
            }
            auwla_ast::ExprKind::Array(elements) => {
                if elements.is_empty() {
                    // Empty array — type must be inferred from context (let binding)
                    // For now return a generic unknown array
                    Ok(Type::Array(Box::new(Type::Basic("unknown".to_string()))))
                } else {
                    let first_ty = self.check_expr(&elements[0])?;
                    for elem in &elements[1..] {
                        let elem_ty = self.check_expr(elem)?;
                        self.assert_type_eq(&first_ty, &elem_ty)
                            .map_err(|msg| TypeError {
                                span: elem.span.clone(),
                                message: msg,
                            })?;
                    }
                    Ok(Type::Array(Box::new(first_ty)))
                }
            }
            auwla_ast::ExprKind::Index {
                expr: arr_expr,
                index,
            } => {
                let expr_ty = self.check_expr(arr_expr)?;
                let idx_ty = self.check_expr(index)?;
                self.assert_type_eq(&Type::Basic("number".to_string()), &idx_ty)
                    .map_err(|msg| TypeError {
                        span: index.span.clone(),
                        message: msg,
                    })?;
                match expr_ty {
                    Type::Array(inner) => Ok(*inner),
                    other => Err(TypeError {
                        span: arr_expr.span.clone(),
                        message: format!(
                            "Type error: cannot index into non-array type '{}'",
                            other
                        ),
                    }),
                }
            }
            auwla_ast::ExprKind::Range { start, end, .. } => {
                let start_ty = self.check_expr(start)?;
                let end_ty = self.check_expr(end)?;
                self.assert_type_eq(&start_ty, &end_ty)
                    .map_err(|msg| TypeError {
                        span: end.span.clone(),
                        message: msg,
                    })?;
                match &start_ty {
                    Type::Basic(name) if name == "number" || name == "char" => {
                        Ok(Type::Array(Box::new(start_ty)))
                    }
                    other => Err(TypeError {
                        span: start.span.clone(),
                        message: format!(
                            "Type error: range endpoints must be 'number' or 'char', got '{}'",
                            other
                        ),
                    }),
                }
            }
            auwla_ast::ExprKind::Interpolation(parts) => {
                // Each part can be any type — they all get coerced to string at runtime
                for part in parts {
                    self.check_expr(part)?;
                }
                Ok(Type::Basic("string".to_string()))
            }
            auwla_ast::ExprKind::Try { expr, error_expr } => {
                let expr_ty = self.check_expr(expr)?;
                let (ok_ty, err_ty) = match &expr_ty {
                    Type::Result { ok_type, err_type } => {
                        ((**ok_type).clone(), (**err_type).clone())
                    }
                    Type::Optional(inner) => ((**inner).clone(), Type::Basic("null".to_string())),
                    _ => {
                        return Err(TypeError {
                            span: expr.span.clone(),
                            message: format!(
                                "Type error: '?' operator requires a Result or Optional type, but got '{}'",
                                expr_ty
                            ),
                        });
                    }
                };

                let source_err_ty = if let Some(err_expr) = error_expr {
                    self.check_expr(err_expr)?
                } else {
                    err_ty
                };

                // Ensure we are inside a function that returns a Result or Optional
                match &self.current_return_type {
                    Some(Some(Type::Result { err_type: fn_err, .. })) => {
                         self.assert_type_eq(fn_err, &source_err_ty).map_err(|msg| TypeError {
                            span: error_expr.as_ref().map(|e| e.span.clone()).unwrap_or_else(|| expr.span.clone()),
                            message: msg,
                        })?;
                    }
                    Some(Some(Type::Optional(_))) => {
                         self.assert_type_eq(&Type::Basic("null".to_string()), &source_err_ty).map_err(|msg| TypeError {
                            span: error_expr.as_ref().map(|e| e.span.clone()).unwrap_or_else(|| expr.span.clone()),
                            message: msg,
                        })?;
                    }
                    _ => return Err(TypeError {
                        span: expr.span.clone(),
                        message: "Type error: '?' operator can only be used inside a function that returns a Result or Optional type".to_string(),
                    }),
                }

                Ok(ok_ty)
            }
            auwla_ast::ExprKind::StructInit { name, fields, .. } => {
                let struct_def_raw = self.structs.get(name).cloned().ok_or_else(|| TypeError {
                    span: expr.span.clone(),
                    message: format!("Undefined struct '{}'", name),
                })?;

                if self.is_namespace(name) {
                    return Err(TypeError {
                        span: expr.span.clone(),
                        message: format!("Type error: cannot instantiate namespace '{}'", name),
                    });
                }

                let struct_def = struct_def_raw;

                if fields.len() != struct_def.len() {
                    return Err(TypeError {
                        span: expr.span.clone(),
                        message: format!(
                            "Struct '{}' expects {} fields, but {} were provided",
                            name,
                            struct_def.len(),
                            fields.len()
                        ),
                    });
                }

                for (def_name, def_ty) in struct_def.iter() {
                    let mut found = false;
                    for (init_name, init_expr) in fields.iter() {
                        if init_name == def_name {
                            found = true;
                            let init_ty = self.check_expr(init_expr)?;
                            self.assert_type_eq(def_ty, &init_ty)
                                .map_err(|msg| TypeError {
                                    span: init_expr.span.clone(),
                                    message: msg,
                                })?;
                            break;
                        }
                    }
                    if !found {
                        return Err(TypeError {
                            span: expr.span.clone(),
                            message: format!(
                                "Type error: missing field '{}' in initialization of struct '{}'",
                                def_name, name
                            ),
                        });
                    }
                }
                Ok(Type::Custom(name.clone()))
            }
            auwla_ast::ExprKind::EnumInit {
                enum_name,
                variant_name,
                args,
                ..
            } => {
                let enum_def = self.enums.get(enum_name).ok_or_else(|| TypeError {
                    span: expr.span.clone(),
                    message: format!("Undefined enum '{}'", enum_name),
                })?;

                let mut found_variant = None;
                for (vname, vargs) in enum_def.iter() {
                    if vname == variant_name {
                        found_variant = Some(vargs.clone());
                        break;
                    }
                }

                let variant_args = found_variant.ok_or_else(|| TypeError {
                    span: expr.span.clone(),
                    message: format!("Enum '{}' has no variant '{}'", enum_name, variant_name),
                })?;

                if args.len() != variant_args.len() {
                    return Err(TypeError {
                        span: expr.span.clone(),
                        message: format!(
                            "Enum variant '{}::{}' expects {} arguments, but {} were provided",
                            enum_name,
                            variant_name,
                            variant_args.len(),
                            args.len()
                        ),
                    });
                }

                for (expected_ty, arg_expr) in variant_args.iter().zip(args) {
                    let actual_ty = self.check_expr(arg_expr)?;
                    self.assert_type_eq(expected_ty, &actual_ty)
                        .map_err(|msg| TypeError {
                            span: arg_expr.span.clone(),
                            message: msg,
                        })?;
                }

                Ok(Type::Custom(enum_name.clone()))
            }
            auwla_ast::ExprKind::PropertyAccess {
                expr: obj_expr,
                property,
            } => {
                let expr_ty = self.check_expr(obj_expr)?;
                match &expr_ty {
                    Type::Custom(name) => {
                        let struct_def = self.structs.get(name).ok_or_else(|| TypeError {
                            span: obj_expr.span.clone(),
                            message: format!("Undefined struct '{}'", name),
                        })?;
                        for (field_name, field_ty) in struct_def.iter() {
                            if field_name == property {
                                return Ok(field_ty.clone());
                            }
                        }
                        return Err(TypeError {
                            span: expr.span.clone(),
                            message: format!(
                                "Type error: struct '{}' has no property '{}'",
                                name, property
                            ),
                        });
                    }
                    other => {
                        let mut keys = vec![self.type_to_key(&expr_ty)];
                        if let Type::Array(_) = expr_ty {
                            keys.push("array".to_string());
                        }
                        if let Type::Generic(name, _) = &expr_ty {
                            keys.push(name.clone());
                        }
                        for key in keys {
                            if let Some(methods) = self.extensions.get(&key) {
                                for method in methods {
                                    if let Some(attr) =
                                        method.attributes.iter().find(|a| a.name == "external")
                                    {
                                        if attr.args.get(0).map(|s| s.as_str()) == Some("js")
                                            && attr.args.get(1).map(|s| s.as_str())
                                                == Some("property")
                                        {
                                            let target = attr
                                                .args
                                                .get(2)
                                                .map(|s| s.as_str())
                                                .unwrap_or(method.name.as_str());
                                            if target == property {
                                                return Ok(method.return_ty.clone().unwrap_or(
                                                    Type::Basic("unknown".to_string()),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        return Err(TypeError {
                            span: obj_expr.span.clone(),
                            message: format!(
                                "Type error: cannot access property '{}' on non-struct type '{}'",
                                property, other
                            ),
                        });
                    }
                }
            }
            auwla_ast::ExprKind::MethodCall {
                expr,
                method,
                args,
                type_args,
            } => {
                let expr_ty = self.check_expr(expr)?;
                // Resolve the type name for lookup in the extension registry
                let mut keys = vec![self.type_to_key(&expr_ty)];
                if let Type::Array(_) = expr_ty {
                    keys.push("array".to_string());
                }
                if let Type::Generic(name, _) = &expr_ty {
                    keys.push(name.clone());
                }
                let mut found_method = None;
                for key in keys {
                    if let Some(sigs) = self.extensions.get(&key) {
                        if let Some(method_sig) =
                            sigs.iter().find(|m| m.name == method.as_str()).cloned()
                        {
                            found_method = Some(method_sig);
                            break;
                        }
                    }
                }
                if let Some(method_sig) = found_method {
                    if method_sig.is_static {
                        return Err(TypeError {
                            span: expr.span.clone(),
                            message: format!(
                                "Type error: '{}::{}' is a static extension — not callable on a value",
                                self.type_to_key(&expr_ty),
                                method
                            ),
                        });
                    }
                    // method_sig.params[0] is self — skip when validating explicit args
                    let explicit_params = if method_sig
                        .params
                        .first()
                        .map(|(n, _)| n == "self")
                        .unwrap_or(false)
                    {
                        &method_sig.params[1..]
                    } else {
                        &method_sig.params[..]
                    };

                    let mut unifier = crate::inference::unify::Unifier::new();

                    let mut type_env = std::collections::HashMap::new();
                    if let Some(t_params) = &method_sig.type_params {
                        if let Some(t_args) = type_args {
                            if t_args.len() != t_params.len() {
                                return Err(TypeError {
                                    span: expr.span.clone(),
                                    message: format!(
                                        "Method '{}' expects {} type arguments, but {} were provided",
                                        method,
                                        t_params.len(),
                                        t_args.len()
                                    ),
                                });
                            }
                            for (tp, ta) in t_params.iter().zip(t_args) {
                                let id = unifier.new_type_var();
                                unifier.bind(id, ta).map_err(|msg| TypeError {
                                    span: expr.span.clone(),
                                    message: msg,
                                })?;
                                type_env.insert(tp.clone(), id);
                            }
                        } else {
                            for tp in t_params {
                                let id = unifier.new_type_var();
                                type_env.insert(tp.clone(), id);
                            }
                        }
                    } else if type_args.is_some() {
                        return Err(TypeError {
                            span: expr.span.clone(),
                            message: format!(
                                "Method '{}' is not generic but type arguments were provided",
                                method
                            ),
                        });
                    }

                    fn instantiate(
                        ty: &Type,
                        env: &std::collections::HashMap<String, usize>,
                    ) -> Type {
                        match ty {
                            Type::TypeVar(name) | Type::Custom(name) => {
                                if let Some(&id) = env.get(name) {
                                    Type::InferenceVar(id)
                                } else {
                                    ty.clone()
                                }
                            }
                            Type::Array(inner) => Type::Array(Box::new(instantiate(inner, env))),
                            Type::Optional(inner) => {
                                Type::Optional(Box::new(instantiate(inner, env)))
                            }
                            Type::Result { ok_type, err_type } => Type::Result {
                                ok_type: Box::new(instantiate(ok_type, env)),
                                err_type: Box::new(instantiate(err_type, env)),
                            },
                            Type::Function(p, r) => {
                                let inst_p = p.iter().map(|p| instantiate(p, env)).collect();
                                let inst_r = Box::new(instantiate(r, env));
                                Type::Function(inst_p, inst_r)
                            }
                            Type::Generic(n, args) => {
                                let inst_args = args.iter().map(|a| instantiate(a, env)).collect();
                                Type::Generic(n.clone(), inst_args)
                            }
                            _ => ty.clone(),
                        }
                    }

                    let inst_params: Vec<Type> = explicit_params
                        .iter()
                        .map(|(_, p)| instantiate(p, &type_env))
                        .collect();
                    let inst_return_ty = method_sig
                        .return_ty
                        .as_ref()
                        .map(|r| instantiate(r, &type_env));

                    if inst_params.len() != args.len() {
                        return Err(TypeError {
                            span: expr.span.clone(),
                            message: format!(
                                "Method '{}' expects {} arg(s), got {}",
                                method,
                                inst_params.len(),
                                args.len()
                            ),
                        });
                    }

                    if let Some(first_param) = method_sig.params.first() {
                        if first_param.0 == "self" {
                            let inst_self = instantiate(&first_param.1, &type_env);
                            unifier
                                .unify(&inst_self, &expr_ty)
                                .map_err(|msg| TypeError {
                                    span: expr.span.clone(),
                                    message: msg,
                                })?;
                        }
                    }

                    for (param_ty, arg_expr) in inst_params.iter().zip(args) {
                        let arg_ty = self.check_expr(arg_expr)?;
                        unifier.unify(param_ty, &arg_ty).map_err(|msg| TypeError {
                            span: arg_expr.span.clone(),
                            message: msg,
                        })?;
                    }

                    let resolved_return = inst_return_ty
                        .map(|r| unifier.resolve(&r))
                        .unwrap_or(Type::Basic("void".to_string()));

                    return Ok(resolved_return);
                }

                return Err(TypeError {
                    span: expr.span.clone(),
                    message: format!(
                        "Type error: method '{}' not found on type '{}' (if this is an extension, make sure it is defined and imported)",
                        method, expr_ty
                    ),
                });
            }
            auwla_ast::ExprKind::StaticMethodCall {
                type_name,
                type_args,
                method,
                args,
                ..
            } => {
                let mut keys = vec![self.extend_key(type_name, type_args)];
                keys.push(type_name.clone());
                let mut method_sig = None;
                for key in keys {
                    if let Some(methods) = self.extensions.get(&key) {
                        if let Some(found) = methods
                            .iter()
                            .find(|m| m.name == method.as_str() && m.is_static)
                            .cloned()
                        {
                            method_sig = Some(found);
                            break;
                        }
                    }
                }
                let method_sig = if let Some(sig) = method_sig {
                    sig
                } else {
                    // Fallback to Enum Variant
                    let maybe_variant_args = self.enums.get(type_name).and_then(|enum_def| {
                        enum_def
                            .iter()
                            .find(|(vn, _)| vn == method)
                            .map(|(_, va)| va.clone())
                    });

                    if let Some(variant_args) = maybe_variant_args {
                        if args.len() != variant_args.len() {
                            return Err(TypeError {
                                span: expr.span.clone(),
                                message: format!(
                                    "Enum variant '{}::{}' expects {} arguments, but {} were provided",
                                    type_name,
                                    method,
                                    variant_args.len(),
                                    args.len()
                                ),
                            });
                        }

                        for (expected_ty, arg_expr) in variant_args.iter().zip(args) {
                            let actual_ty = self.check_expr(arg_expr)?;
                            self.assert_type_eq(expected_ty, &actual_ty)
                                .map_err(|msg| TypeError {
                                    span: arg_expr.span.clone(),
                                    message: msg,
                                })?;
                        }

                        return Ok(Type::Custom(type_name.clone()));
                    }

                    return Err(TypeError {
                        span: expr.span.clone(),
                        message: format!(
                            "Type error: static method or enum variant '{}' not found for type '{}'",
                            method, type_name
                        ),
                    });
                };

                if args.len() != method_sig.params.len() {
                    return Err(TypeError {
                        span: expr.span.clone(),
                        message: format!(
                            "Type error: expected {} arguments for static method '{}', found {}",
                            method_sig.params.len(),
                            method,
                            args.len()
                        ),
                    });
                }

                for (arg, (_, expected_ty)) in args.iter().zip(method_sig.params.iter()) {
                    let arg_ty = self.check_expr(arg)?;
                    self.assert_type_eq(expected_ty, &arg_ty)
                        .map_err(|msg| TypeError {
                            span: arg.span.clone(),
                            message: msg,
                        })?;
                }

                Ok(method_sig
                    .return_ty
                    .clone()
                    .unwrap_or(Type::Basic("void".to_string())))
            }
            auwla_ast::ExprKind::Block(stmts, result) => {
                self.enter_scope();
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
                let ty = if let Some(res) = result {
                    self.check_expr(res)?
                } else {
                    // If we're inside a return context and have no trailing expr,
                    // the block's type is the declared return type (body uses `return`).
                    self.current_return_type
                        .as_ref()
                        .and_then(|r| r.clone())
                        .unwrap_or(Type::Basic("void".to_string()))
                };
                self.exit_scope();
                Ok(ty)
            }
            auwla_ast::ExprKind::Closure {
                params,
                return_ty,
                body,
                ..
            } => {
                let mut param_types = Vec::new();
                self.enter_scope();
                for (name, ty_opt) in params {
                    let ty = ty_opt.clone().ok_or_else(|| TypeError {
                        span: expr.span.clone(),
                        message: format!(
                            "Type error: parameter '{}' must have a type annotation",
                            name
                        ),
                    })?;
                    param_types.push(ty.clone());
                    self.declare_variable(
                        expr.span.clone(),
                        name.clone(),
                        ty,
                        Mutability::Immutable,
                    )?;
                }
                // Set return context so `return` stmts inside the body are valid
                let saved_return_type = self.current_return_type.take();
                let saved_fn_name = self.current_function_name.take();
                self.current_return_type = Some(return_ty.clone());
                self.current_function_name = Some("closure".to_string());
                let body_ty = self.check_expr(body)?;
                self.current_return_type = saved_return_type;
                self.current_function_name = saved_fn_name;
                let final_return_ty = if let Some(expected_ret) = return_ty {
                    self.assert_type_eq(expected_ret, &body_ty)
                        .map_err(|msg| TypeError {
                            span: expr.span.clone(),
                            message: msg,
                        })?;
                    expected_ret.clone()
                } else {
                    body_ty
                };
                self.exit_scope();
                Ok(Type::Function(param_types, Box::new(final_return_ty)))
            }
        }
    }
}
