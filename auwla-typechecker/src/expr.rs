use crate::checker::Typechecker;
use crate::scope::Mutability;
use auwla_ast::{Expr, Type};

impl Typechecker {
    pub fn check_expr(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Void => Ok(Type::Basic("void".to_string())),
            Expr::BoolLit(_) => Ok(Type::Basic("bool".to_string())),
            Expr::StringLit(_) => Ok(Type::Basic("string".to_string())),
            Expr::NumberLit(_) => Ok(Type::Basic("number".to_string())),
            Expr::CharLit(_) => Ok(Type::Basic("char".to_string())),
            Expr::Identifier(name) => self
                .lookup_variable(name)
                .ok_or_else(|| format!("Undefined variable: '{}'", name)),
            Expr::Binary { op, left, right } => {
                let left_ty = self.check_expr(left)?;
                let right_ty = self.check_expr(right)?;

                self.assert_type_eq(&left_ty, &right_ty)
                    .map_err(|_| format!("Strict Type inferred error: Operators must have matching types. Left: {:?}, Right: {:?}", left_ty, right_ty))?;

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
            Expr::Some(inner) => {
                let inner_ty = self.check_expr(inner)?;
                Ok(Type::Result {
                    ok_type: Box::new(inner_ty),
                    err_type: Box::new(Type::Basic("unknown".to_string())),
                })
            }
            Expr::None(inner_opt) => {
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
            Expr::Call { name, args } => {
                // Built-in functions
                if name == "print" {
                    // print() accepts any number of arguments of any type
                    for arg in args {
                        self.check_expr(arg)?;
                    }
                    return Ok(Type::Basic("void".to_string()));
                }

                let (params, return_ty) = if let Some(var_ty) = self.lookup_variable(name) {
                    if let Type::Function(p, r) = var_ty {
                        (p, Some(*r))
                    } else {
                        return Err(format!("Type error: variable '{}' is not a function", name));
                    }
                } else {
                    self.lookup_function(name)
                        .ok_or_else(|| format!("Undefined function: '{}'", name))?
                };

                if params.len() != args.len() {
                    return Err(format!(
                        "Function '{}' expects {} arguments, but {} were provided",
                        name,
                        params.len(),
                        args.len()
                    ));
                }

                for (param_ty, arg_expr) in params.iter().zip(args) {
                    let arg_ty = self.check_expr(arg_expr)?;
                    self.assert_type_eq(param_ty, &arg_ty)?;
                }

                Ok(return_ty.unwrap_or(Type::Basic("void".to_string())))
            }
            Expr::Unary { op, expr } => {
                let inner = self.check_expr(expr)?;
                match op {
                    auwla_ast::UnaryOp::Not => {
                        self.assert_type_eq(&Type::Basic("bool".to_string()), &inner)
                            .map_err(|_| {
                                "Type error: '!' requires a bool expression".to_string()
                            })?;
                        Ok(Type::Basic("bool".to_string()))
                    }
                    auwla_ast::UnaryOp::Neg => {
                        self.assert_type_eq(&Type::Basic("number".to_string()), &inner)
                            .map_err(|_| {
                                "Type error: '-' requires a number expression".to_string()
                            })?;
                        Ok(Type::Basic("number".to_string()))
                    }
                }
            }
            Expr::Match { expr, arms } => {
                let result_ty = self.check_expr(expr)?;
                let mut common_return_ty: Option<Type> = None;

                fn collect_variants<'a>(
                    pattern: &'a auwla_ast::Pattern,
                    variants: &mut Vec<&'a auwla_ast::Pattern>,
                ) {
                    match pattern {
                        auwla_ast::Pattern::Or(patterns) => {
                            for p in patterns {
                                collect_variants(p, variants);
                            }
                        }
                        other => variants.push(other),
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
                        match p {
                            auwla_ast::Pattern::Wildcard => {
                                if arm.guard.is_none() {
                                    has_wildcard = true;
                                }
                            }
                            auwla_ast::Pattern::Variable(name) => {
                                if i == 0 {
                                    self.declare_variable(
                                        name.clone(),
                                        result_ty.clone(),
                                        Mutability::Immutable,
                                    )?;
                                }
                                if arm.guard.is_none() {
                                    has_wildcard = true;
                                }
                            }
                            auwla_ast::Pattern::Literal(lit_expr) => {
                                let lit_ty = self.check_expr(lit_expr)?;
                                self.assert_type_eq(&result_ty, &lit_ty).map_err(|_| format!("Type error: match arm pattern literal '{:?}' does not match expected type '{:?}'", lit_ty, result_ty))?;
                            }
                            auwla_ast::Pattern::Range {
                                start,
                                end,
                                inclusive: _,
                            } => {
                                let start_ty = self.check_expr(start)?;
                                let end_ty = self.check_expr(end)?;
                                self.assert_type_eq(&start_ty, &end_ty).map_err(|_| format!("Type error: match arm pattern range bounds have different types: '{:?}' and '{:?}'", start_ty, end_ty))?;
                                self.assert_type_eq(&result_ty, &start_ty).map_err(|_| format!("Type error: match arm pattern range type does not match expected type '{:?}'", result_ty))?;
                            }
                            auwla_ast::Pattern::Variant { name, bindings } => {
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
                                                        binding.clone(),
                                                        *err_type.clone(),
                                                        Mutability::Immutable,
                                                    )?;
                                                }
                                            }
                                        } else if let Type::Optional(_) = &result_ty {
                                            if !bindings.is_empty() {
                                                return Err("Type error: match arm 'none' for Optional type cannot bind arguments".to_string());
                                            }
                                        }
                                    } else {
                                        return Err(format!(
                                            "Type error: matching on a Result expects 'some' or 'none', found '{}'",
                                            name
                                        ));
                                    }
                                } else if let Some(ref def) = enum_def {
                                    if arm.guard.is_none() {
                                        handled_variants.insert(name.clone());
                                    }
                                    let variant_args = def
                                        .iter()
                                        .find(|(n, _)| n == name)
                                        .ok_or_else(|| {
                                            format!("Type error: variant '{}' does not exist", name)
                                        })?
                                        .1
                                        .clone();

                                    if bindings.len() != variant_args.len() {
                                        return Err(format!(
                                            "Type error: match arm '{}' binds {} arguments, but variant has {}",
                                            name,
                                            bindings.len(),
                                            variant_args.len()
                                        ));
                                    }

                                    if i == 0 {
                                        for (binding, expected_ty) in
                                            bindings.iter().zip(variant_args)
                                        {
                                            self.declare_variable(
                                                binding.clone(),
                                                expected_ty,
                                                Mutability::Immutable,
                                            )?;
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "Type error: cannot match variant '{}' on type '{:?}'",
                                        name, result_ty
                                    ));
                                }
                            }
                            auwla_ast::Pattern::Struct(opt_name, fields) => {
                                match &result_ty {
                                    Type::Custom(struct_name) => {
                                        if let Some(name) = opt_name {
                                            if name != struct_name {
                                                return Err(format!(
                                                    "Type error: expected struct '{}', found '{}'",
                                                    struct_name, name
                                                ));
                                            }
                                        }

                                        let struct_def =
                                            self.structs.get(struct_name).cloned().ok_or_else(
                                                || {
                                                    format!(
                                                        "Type error: struct '{}' not found",
                                                        struct_name
                                                    )
                                                },
                                            )?;

                                        for (field_name, sub_pattern_opt) in fields {
                                            let field_ty = struct_def
                                                .iter()
                                                .find(|(f, _)| f == field_name)
                                                .map(|(_, t)| t.clone())
                                                .ok_or_else(|| {
                                                    format!("Type error: field '{}' not found on struct '{}'", field_name, struct_name)
                                                })?;

                                            if let Some(sub_pattern) = sub_pattern_opt {
                                                // If there's a nested pattern, we'd need to recursively typecheck it.
                                                // For now, if it's a variable binding, declare it in scope.
                                                if let auwla_ast::Pattern::Variable(var_name) =
                                                    sub_pattern
                                                {
                                                    if i == 0 {
                                                        self.declare_variable(
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
                                        return Err(format!(
                                            "Type error: cannot match struct '{}' on type '{:?}'",
                                            display_name, result_ty
                                        ));
                                    }
                                }
                            }
                            auwla_ast::Pattern::Or(_) => unreachable!(),
                        }
                    }

                    if let Some(ref guard) = arm.guard {
                        let guard_ty = self.check_expr(guard)?;
                        self.assert_type_eq(&Type::Basic("bool".to_string()), &guard_ty)
                            .map_err(|_| {
                                format!(
                                    "Type error: match guard must be a boolean, got '{:?}'",
                                    guard_ty
                                )
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
                        self.assert_type_eq(prev_ty, &arm_yields).map_err(|_| format!("Type error: match arms return different types — expected '{:?}', found '{:?}'", prev_ty, arm_yields))?;
                    } else {
                        common_return_ty = Some(arm_yields);
                    }
                }

                if is_primitive && !has_wildcard {
                    return Err(format!(
                        "Type error: match on '{:?}' primitive must have a wildcard '_' arm for exhaustiveness",
                        result_ty
                    ));
                }

                if let Type::Result { .. } = &result_ty {
                    if !has_wildcard && (!has_some || !has_none) {
                        return Err("Type error: match on Result must be exhaustive (handle 'some' and 'none' or use '_')".to_string());
                    }
                }

                if let Some(def) = enum_def {
                    if !has_wildcard {
                        let defined_variants: std::collections::HashSet<_> =
                            def.iter().map(|(n, _)| n.clone()).collect();
                        let missing: Vec<_> =
                            defined_variants.difference(&handled_variants).collect();
                        if !missing.is_empty() {
                            return Err(format!(
                                "Type error: match on enum is not exhaustive. Missing matching arms for variants: {:?}",
                                missing
                            ));
                        }
                    }
                }

                Ok(common_return_ty.unwrap_or_else(|| Type::Basic("void".to_string())))
            }
            Expr::Array(elements) => {
                if elements.is_empty() {
                    // Empty array — type must be inferred from context (let binding)
                    // For now return a generic unknown array
                    Ok(Type::Array(Box::new(Type::Basic("unknown".to_string()))))
                } else {
                    let first_ty = self.check_expr(&elements[0])?;
                    for elem in &elements[1..] {
                        let elem_ty = self.check_expr(elem)?;
                        self.assert_type_eq(&first_ty, &elem_ty).map_err(|_| format!(
                            "Type error: array elements must all be the same type. Expected '{:?}', found '{:?}'",
                            first_ty, elem_ty
                        ))?;
                    }
                    Ok(Type::Array(Box::new(first_ty)))
                }
            }
            Expr::Index { expr, index } => {
                let expr_ty = self.check_expr(expr)?;
                let idx_ty = self.check_expr(index)?;
                self.assert_type_eq(&Type::Basic("number".to_string()), &idx_ty)
                    .map_err(|_| {
                        format!(
                            "Type error: array index must be 'number', got '{:?}'",
                            idx_ty
                        )
                    })?;
                match expr_ty {
                    Type::Array(inner) => Ok(*inner),
                    other => Err(format!(
                        "Type error: cannot index into non-array type '{:?}'",
                        other
                    )),
                }
            }
            Expr::Range { start, end, .. } => {
                let start_ty = self.check_expr(start)?;
                let end_ty = self.check_expr(end)?;
                self.assert_type_eq(&start_ty, &end_ty).map_err(|_| format!(
                    "Type error: range endpoints must be the same type. Start: '{:?}', End: '{:?}'",
                    start_ty, end_ty
                ))?;
                match &start_ty {
                    Type::Basic(name) if name == "number" || name == "char" => {
                        Ok(Type::Array(Box::new(start_ty)))
                    }
                    other => Err(format!(
                        "Type error: range endpoints must be 'number' or 'char', got '{:?}'",
                        other
                    )),
                }
            }
            Expr::Interpolation(parts) => {
                // Each part can be any type — they all get coerced to string at runtime
                for part in parts {
                    self.check_expr(part)?;
                }
                Ok(Type::Basic("string".to_string()))
            }
            Expr::Try { expr, error_expr } => {
                let expr_ty = self.check_expr(expr)?;
                let (ok_ty, err_ty) = match &expr_ty {
                    Type::Result { ok_type, err_type } => {
                        ((**ok_type).clone(), (**err_type).clone())
                    }
                    Type::Optional(inner) => ((**inner).clone(), Type::Basic("null".to_string())),
                    _ => {
                        return Err(format!(
                            "Type error: '?' operator requires a Result or Optional type, but got '{:?}'",
                            expr_ty
                        ));
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
                         self.assert_type_eq(fn_err, &source_err_ty).map_err(|_| format!(
                            "Type error: '?' operator error expression type '{:?}' does not match function error return type '{:?}'",
                            source_err_ty, fn_err
                        ))?;
                    }
                    Some(Some(Type::Optional(_))) => {
                         self.assert_type_eq(&Type::Basic("null".to_string()), &source_err_ty).map_err(|_| format!(
                            "Type error: '?' operator on Optional inside an Optional-returning function cannot specify an error value"
                        ))?;
                    }
                    _ => return Err("Type error: '?' operator can only be used inside a function that returns a Result or Optional type".to_string()),
                }

                Ok(ok_ty)
            }
            Expr::StructInit { name, fields } => {
                let struct_def = self
                    .structs
                    .get(name)
                    .ok_or_else(|| format!("Undefined struct '{}'", name))?
                    .clone();

                if fields.len() != struct_def.len() {
                    return Err(format!(
                        "Struct '{}' expects {} fields, but {} were provided",
                        name,
                        struct_def.len(),
                        fields.len()
                    ));
                }

                for (def_name, def_ty) in struct_def.iter() {
                    let mut found = false;
                    for (init_name, init_expr) in fields.iter() {
                        if init_name == def_name {
                            found = true;
                            let init_ty = self.check_expr(init_expr)?;
                            self.assert_type_eq(def_ty, &init_ty).map_err(|_| format!("Type error: struct '{}' field '{}' expects '{:?}', but got '{:?}'", name, def_name, def_ty, init_ty))?;
                            break;
                        }
                    }
                    if !found {
                        return Err(format!(
                            "Type error: missing field '{}' in initialization of struct '{}'",
                            def_name, name
                        ));
                    }
                }
                Ok(Type::Custom(name.clone()))
            }
            Expr::EnumInit {
                enum_name,
                variant_name,
                args,
            } => {
                let enum_def = self
                    .enums
                    .get(enum_name)
                    .ok_or_else(|| format!("Undefined enum '{}'", enum_name))?;

                let mut found_variant = None;
                for (vname, vargs) in enum_def.iter() {
                    if vname == variant_name {
                        found_variant = Some(vargs.clone());
                        break;
                    }
                }

                let variant_args = found_variant.ok_or_else(|| {
                    format!("Enum '{}' has no variant '{}'", enum_name, variant_name)
                })?;

                if args.len() != variant_args.len() {
                    return Err(format!(
                        "Enum variant '{}::{}' expects {} arguments, but {} were provided",
                        enum_name,
                        variant_name,
                        variant_args.len(),
                        args.len()
                    ));
                }

                for (expected_ty, arg_expr) in variant_args.iter().zip(args) {
                    let actual_ty = self.check_expr(arg_expr)?;
                    self.assert_type_eq(expected_ty, &actual_ty)?;
                }

                Ok(Type::Custom(enum_name.clone()))
            }
            Expr::PropertyAccess { expr, property } => {
                let expr_ty = self.check_expr(expr)?;
                match expr_ty {
                    Type::Custom(name) => {
                        let struct_def = self
                            .structs
                            .get(&name)
                            .ok_or_else(|| format!("Undefined struct '{}'", name))?;
                        for (field_name, field_ty) in struct_def.iter() {
                            if field_name == property {
                                return Ok(field_ty.clone());
                            }
                        }
                        return Err(format!(
                            "Type error: struct '{}' has no property '{}'",
                            name, property
                        ));
                    }
                    other => {
                        return Err(format!(
                            "Type error: cannot access property '{}' on non-struct type '{:?}'",
                            property, other
                        ));
                    }
                }
            }
            Expr::MethodCall { expr, method, args } => {
                let expr_ty = self.check_expr(expr)?;
                // Resolve the type name for lookup in the extension registry
                let type_key = match &expr_ty {
                    Type::Custom(name) => name.clone(),
                    Type::Basic(name) => name.clone(),
                    other => format!("{:?}", other),
                };
                let method_sigs = self.extensions.get(&type_key).cloned();
                if let Some(sigs) = method_sigs {
                    if let Some((_, is_static, params, return_ty)) =
                        sigs.iter().find(|(n, _, _, _)| n == method).cloned()
                    {
                        if is_static {
                            return Err(format!(
                                "Type error: '{}::{}' is a static extension — not callable on a value",
                                type_key, method
                            ));
                        }
                        // params[0] is self — skip when validating explicit args
                        let explicit_params =
                            if params.first().map(|(n, _)| n == "self").unwrap_or(false) {
                                &params[1..]
                            } else {
                                &params[..]
                            };
                        if explicit_params.len() != args.len() {
                            return Err(format!(
                                "Method '{}' expects {} arg(s), got {}",
                                method,
                                explicit_params.len(),
                                args.len()
                            ));
                        }
                        for ((_, expected), arg_expr) in explicit_params.iter().zip(args) {
                            let arg_ty = self.check_expr(arg_expr)?;
                            self.assert_type_eq(expected, &arg_ty)?;
                        }
                        return Ok(return_ty.unwrap_or(Type::Basic("void".to_string())));
                    }
                }
                // Not found as extension — might be a closure field call; validate args permissively
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(Type::Basic("unknown".to_string()))
            }
            Expr::Block(stmts, result) => {
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
            Expr::Closure {
                params,
                return_ty,
                body,
            } => {
                let mut param_types = Vec::new();
                self.enter_scope();
                for (name, ty_opt) in params {
                    let ty = ty_opt.clone().ok_or_else(|| {
                        format!(
                            "Type error: parameter '{}' must have a type annotation",
                            name
                        )
                    })?;
                    param_types.push(ty.clone());
                    self.declare_variable(name.clone(), ty, Mutability::Immutable)?;
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
                    self.assert_type_eq(expected_ret, &body_ty)?;
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
