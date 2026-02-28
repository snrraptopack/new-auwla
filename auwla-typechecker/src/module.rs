use auwla_ast::{Program, Stmt, Type};
use std::collections::HashMap;

/// The public API surface of a single Auwla source file.
/// Built by `collect_exports()` and consumed by the typechecker when
/// resolving `import { ... } from '...'` statements in other files.
#[derive(Debug, Clone, Default)]
pub struct ExportMap {
    /// Exported top-level functions: name -> (type_params, param_types, return_type)
    pub functions: HashMap<String, (Option<Vec<String>>, Vec<Type>, Option<Type>)>,
    /// Exported variables / constants: name -> type
    pub variables: HashMap<String, Type>,
    /// Exported struct declarations: name -> field list
    pub structs: HashMap<String, Vec<(String, Type)>>,
    /// Exported enum declarations: name -> variant list
    pub enums: HashMap<String, Vec<(String, Vec<Type>)>>,
}

/// First-pass scan: collect every exported name and its type signature
/// without doing full type-checking.  This is fast and used to build the
/// import context for files that depend on this one.
pub fn collect_exports(program: &Program) -> ExportMap {
    let mut map = ExportMap::default();

    for stmt in &program.statements {
        if let Stmt::Export { stmt: inner } = stmt {
            register_export(&mut map, inner);
        }
    }

    map
}

fn register_export(map: &mut ExportMap, stmt: &Stmt) {
    match stmt {
        Stmt::Fn {
            name,
            type_params,
            params,
            return_ty,
            ..
        } => {
            let param_types: Vec<Type> = params.iter().map(|(_, t)| t.clone()).collect();
            map.functions.insert(
                name.clone(),
                (type_params.clone(), param_types, return_ty.clone()),
            );
        }
        Stmt::Let {
            name,
            ty,
            initializer,
        }
        | Stmt::Var {
            name,
            ty,
            initializer,
        } => {
            if let Some(t) = ty {
                // Explicit type annotation — register as a typed variable
                map.variables.insert(name.clone(), t.clone());
            } else if let auwla_ast::Expr::Closure {
                type_params,
                params,
                return_ty,
                ..
            } = initializer
            {
                // No annotation, but initializer is a closure — register as a function
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|(_, t)| t.clone().unwrap_or(Type::Basic("unknown".to_string())))
                    .collect();
                map.functions.insert(
                    name.clone(),
                    (type_params.clone(), param_types, return_ty.clone()),
                );
            }
        }
        Stmt::StructDecl { name, fields, .. } => {
            map.structs.insert(name.clone(), fields.clone());
        }
        Stmt::EnumDecl { name, variants, .. } => {
            map.enums.insert(name.clone(), variants.clone());
        }
        // nested export (shouldn't occur but handle gracefully)
        Stmt::Export { stmt: inner } => register_export(map, inner),
        _ => {}
    }
}
