use crate::scope::{Mutability, Scope};
use auwla_ast::{Expr, Program, Stmt, Type};

use std::collections::HashMap;

pub struct Typechecker {
    scopes: Vec<Scope>,
    current_return_type: Option<Option<Type>>,
    structs: HashMap<String, Vec<(String, Type)>>,
    enums: HashMap<String, Vec<(String, Vec<Type>)>>,
}

impl Default for Typechecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Typechecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            current_return_type: None,
            structs: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop().expect("Cannot pop the global scope");
    }

    pub fn declare_variable(
        &mut self,
        name: String,
        ty: Type,
        mutability: Mutability,
    ) -> Result<(), String> {
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.variables.contains_key(&name) {
            return Err(format!(
                "Variable '{}' is already defined in this scope.",
                name
            ));
        }
        current_scope.mutability.insert(name.clone(), mutability);
        current_scope.variables.insert(name, ty);
        Ok(())
    }

    pub fn declare_function(&mut self, name: String, params: Vec<Type>, ret: Option<Type>) {
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.functions.insert(name, (params, ret));
    }

    pub fn is_mutable(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if let Some(m) = scope.mutability.get(name) {
                return *m == Mutability::Mutable;
            }
        }
        false // unknown vars treated as immutable
    }

    pub fn lookup_variable(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.variables.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub fn lookup_function(&self, name: &str) -> Option<(Vec<Type>, Option<Type>)> {
        for scope in self.scopes.iter().rev() {
            if let Some(sig) = scope.functions.get(name) {
                return Some(sig.clone());
            }
        }
        None
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let {
                name,
                ty,
                initializer,
            } => {
                let init_ty = self.check_expr(initializer)?;
                let final_ty = if let Some(declared_ty) = ty {
                    self.assert_type_eq(declared_ty, &init_ty)?;
                    declared_ty.clone()
                } else {
                    init_ty
                };
                self.declare_variable(name.clone(), final_ty, Mutability::Immutable)?;
                Ok(())
            }
            Stmt::DestructureLet {
                bindings,
                initializer,
            } => {
                let init_ty = self.check_expr(initializer)?;

                match init_ty {
                    Type::Custom(struct_name) => {
                        let struct_def =
                            self.structs.get(&struct_name).cloned().ok_or_else(|| {
                                format!("Type error: struct '{}' not found", struct_name)
                            })?;

                        for binding in bindings {
                            let field_ty = struct_def
                                .iter()
                                .find(|(f, _)| f == binding)
                                .map(|(_, t)| t.clone())
                                .ok_or_else(|| {
                                    format!(
                                        "Type error: field '{}' not found on struct '{}'",
                                        binding, struct_name
                                    )
                                })?;

                            self.declare_variable(
                                binding.clone(),
                                field_ty,
                                Mutability::Immutable,
                            )?;
                        }
                    }
                    _ => {
                        return Err(format!(
                            "Type error: expected struct for destructuring, found '{:?}'",
                            init_ty
                        ));
                    }
                }
                Ok(())
            }
            Stmt::Var {
                name,
                ty,
                initializer,
            } => {
                let init_ty = self.check_expr(initializer)?;
                let final_ty = if let Some(declared_ty) = ty {
                    self.assert_type_eq(declared_ty, &init_ty)?;
                    declared_ty.clone()
                } else {
                    init_ty
                };
                self.declare_variable(name.clone(), final_ty, Mutability::Mutable)?;
                Ok(())
            }
            Stmt::Assign { target, value } => {
                let val_ty = self.check_expr(value)?;

                match target {
                    Expr::Identifier(name) => {
                        let var_ty = self.lookup_variable(name).ok_or_else(|| {
                            format!(
                                "Undefined variable '{}' — declare it with `var` first",
                                name
                            )
                        })?;

                        if !self.is_mutable(name) {
                            return Err(format!(
                                "Cannot reassign '{}' — it was declared with `let` (immutable). Use `var` to allow reassignment.",
                                name
                            ));
                        }

                        self.assert_type_eq(&var_ty, &val_ty)?;
                    }
                    Expr::PropertyAccess { expr, property } => {
                        let expr_ty = self.check_expr(expr)?;
                        match expr_ty {
                            Type::Custom(name) => {
                                let struct_def = self
                                    .structs
                                    .get(&name)
                                    .ok_or_else(|| format!("Undefined struct '{}'", name))?;
                                let mut found = false;
                                for (field_name, field_ty) in struct_def.iter() {
                                    if field_name == property {
                                        found = true;
                                        self.assert_type_eq(field_ty, &val_ty).map_err(|_| format!("Type error: struct '{}' field '{}' expects '{:?}', but got '{:?}'", name, property, field_ty, val_ty))?;
                                        break;
                                    }
                                }
                                if !found {
                                    return Err(format!(
                                        "Type error: struct '{}' has no property '{}'",
                                        name, property
                                    ));
                                }
                            }
                            other => {
                                return Err(format!(
                                    "Type error: cannot assign property '{}' on non-struct type '{:?}'",
                                    property, other
                                ));
                            }
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
                            Type::Array(inner) => {
                                self.assert_type_eq(&inner, &val_ty)?;
                            }
                            other => {
                                return Err(format!(
                                    "Type error: cannot index into non-array type '{:?}'",
                                    other
                                ));
                            }
                        }
                    }
                    other => {
                        return Err(format!(
                            "Type error: invalid assignment target '{:?}'",
                            other
                        ));
                    }
                }
                Ok(())
            }
            Stmt::Fn {
                name,
                params,
                return_ty,
                body,
            } => {
                let param_types = params.iter().map(|(_, ty)| ty.clone()).collect();
                self.declare_function(name.clone(), param_types, return_ty.clone());

                let prev_return = self.current_return_type.take();
                self.current_return_type = Some(return_ty.clone());

                self.enter_scope();
                // Fn params are always mutable within their scope
                for (param_name, ty) in params {
                    self.declare_variable(param_name.clone(), ty.clone(), Mutability::Mutable)?;
                }
                for body_stmt in body {
                    self.check_stmt(body_stmt)?;
                }
                self.exit_scope();

                self.current_return_type = prev_return;
                Ok(())
            }
            Stmt::If {
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
            Stmt::Return(expr_opt) => {
                let actual_ty = if let Some(expr) = expr_opt {
                    Some(self.check_expr(expr)?)
                } else {
                    None
                };

                if let Some(expected_ty_opt) = &self.current_return_type {
                    match (expected_ty_opt, actual_ty) {
                        (Some(expected), Some(actual)) => {
                            self.assert_type_eq(expected, &actual).map_err(|_| {
                                format!(
                                    "Strict Type error: Function expects to return '{:?}', but returned '{:?}'",
                                    expected, actual
                                )
                            })?;
                        }
                        (None, Some(actual)) => {
                            if actual != Type::Basic("void".to_string()) {
                                return Err(format!(
                                    "Strict Type error: Function expected to return nothing, but returned '{:?}'",
                                    actual
                                ));
                            }
                        }
                        (Some(expected), None) => {
                            return Err(format!(
                                "Strict Type error: Function expects to return '{:?}', but returned nothing",
                                expected
                            ));
                        }
                        (None, None) => {}
                    }
                } else {
                    return Err(
                        "Strict Type error: 'return' statement outside of function".to_string()
                    );
                }

                Ok(())
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::While { condition, body } => {
                self.check_expr(condition)?;
                self.enter_scope();
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.exit_scope();
                Ok(())
            }
            Stmt::For {
                binding,
                iterable,
                body,
            } => {
                let iter_ty = self.check_expr(iterable)?;
                let elem_ty = match iter_ty {
                    Type::Array(inner) => *inner,
                    other => {
                        return Err(format!(
                            "Type error: 'for..in' requires an array or range, but got '{:?}'",
                            other
                        ));
                    }
                };
                self.enter_scope();
                self.declare_variable(binding.clone(), elem_ty, Mutability::Immutable)?;
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.exit_scope();
                Ok(())
            }
            Stmt::StructDecl { name, fields } => {
                if self.structs.contains_key(name) {
                    return Err(format!("Struct '{}' is already defined", name));
                }
                self.structs.insert(name.clone(), fields.clone());
                Ok(())
            }
            Stmt::EnumDecl { name, variants } => {
                if self.enums.contains_key(name) {
                    return Err(format!("Enum '{}' is already defined", name));
                }
                self.enums.insert(name.clone(), variants.clone());
                Ok(())
            }
        }
    }

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
            // If it evaluates `none()`, it wraps the ERR branch.
            Expr::None(inner) => {
                let inner_ty = self.check_expr(inner)?;
                Ok(Type::Result {
                    ok_type: Box::new(Type::Basic("unknown".to_string())),
                    err_type: Box::new(inner_ty),
                })
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

                let (params, return_ty) = self
                    .lookup_function(name)
                    .ok_or_else(|| format!("Undefined function: '{}'", name))?;

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
                                if let Type::Result { ok_type, err_type } = &result_ty {
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
                                        if i == 0 {
                                            if let Some(binding) = bindings.first() {
                                                self.declare_variable(
                                                    binding.clone(),
                                                    *err_type.clone(),
                                                    Mutability::Immutable,
                                                )?;
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
                    _ => {
                        return Err(format!(
                            "Type error: '?' operator requires a Result type, but got '{:?}'",
                            expr_ty
                        ));
                    }
                };

                let source_err_ty = if let Some(err_expr) = error_expr {
                    self.check_expr(err_expr)?
                } else {
                    err_ty
                };

                // Ensure we are inside a function that returns a Result
                match &self.current_return_type {
                    Some(Some(Type::Result { err_type: fn_err, .. })) => {
                         self.assert_type_eq(fn_err, &source_err_ty).map_err(|_| format!(
                            "Type error: '?' operator error expression type '{:?}' does not match function error return type '{:?}'",
                            source_err_ty, fn_err
                        ))?;
                    }
                    _ => return Err("Type error: '?' operator can only be used inside a function that returns a Result type".to_string()),
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
        }
    }

    fn assert_type_eq(&self, expected: &Type, actual: &Type) -> Result<(), String> {
        // Handle `type?error_type` resolution from `some()` and `none()` with unknowns.
        if let (
            Type::Result {
                ok_type: e_ok,
                err_type: e_err,
            },
            Type::Result {
                ok_type: a_ok,
                err_type: a_err,
            },
        ) = (expected, actual)
        {
            let ok_match = if let Type::Basic(name) = &**a_ok {
                name == "unknown" || e_ok == a_ok
            } else {
                e_ok == a_ok
            };

            let err_match = if let Type::Basic(name) = &**a_err {
                name == "unknown" || e_err == a_err
            } else {
                e_err == a_err
            };

            if ok_match && err_match {
                return Ok(());
            }
        }

        if expected == actual {
            Ok(())
        } else {
            Err(format!(
                "Strict Type mismatch: Expected '{:?}', found '{:?}'",
                expected, actual
            ))
        }
    }
}
