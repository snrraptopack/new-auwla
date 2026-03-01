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
    /// Exported extension methods: type_key -> [ExtensionMethod]
    pub extensions: HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
}

/// First-pass scan: collect every exported name and its type signature
/// without doing full type-checking.  This is fast and used to build the
/// import context for files that depend on this one.
pub fn collect_exports(program: &Program, file_path: &str) -> ExportMap {
    let mut map = ExportMap::default();

    for stmt in &program.statements {
        match &stmt.node {
            auwla_ast::StmtKind::Export { stmt: inner } => {
                register_export(&mut map, inner, file_path);
            }
            auwla_ast::StmtKind::Extend { .. } => {
                // Extensions are globally visible in Auwla if the file is part of the project
                register_export(&mut map, stmt, file_path);
            }
            _ => {}
        }
    }

    map
}

fn register_export(map: &mut ExportMap, stmt: &Stmt, file_path: &str) {
    match &stmt.node {
        auwla_ast::StmtKind::Fn {
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
        auwla_ast::StmtKind::Let {
            name,
            ty,
            initializer,
        }
        | auwla_ast::StmtKind::Var {
            name,
            ty,
            initializer,
        } => {
            if let Some(t) = ty {
                // Explicit type annotation — register as a typed variable
                map.variables.insert(name.clone(), t.clone());
            } else if let auwla_ast::ExprKind::Closure {
                type_params,
                params,
                return_ty,
                ..
            } = &initializer.node
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
        auwla_ast::StmtKind::StructDecl { name, fields, .. } => {
            map.structs.insert(name.clone(), fields.clone());
        }
        auwla_ast::StmtKind::EnumDecl { name, variants, .. } => {
            map.enums.insert(name.clone(), variants.clone());
        }
        // nested export (shouldn't occur but handle gracefully)
        auwla_ast::StmtKind::Export { stmt: inner } => register_export(map, inner, file_path),
        auwla_ast::StmtKind::Extend {
            type_name,
            type_params,
            type_args,
            methods,
        } => {
            // In Auwla, extensions are currently public/global by default if the file is imported.
            // (Similar to how we bundle all extensions into __runtime.js)
            // We'll collect them into the ExportMap so the typechecker can see them.
            let type_key = extend_key_simple(type_name, type_args);
            let mut method_sigs = Vec::new();

            let mut base_tps = Vec::new();
            if let Some(tps) = type_params.as_ref() {
                base_tps.extend(tps.clone());
            }

            // We need a way to determine the 'self' type for the signature
            let self_type = if let Some(args) = type_args {
                if type_name == "array" {
                    if let Some(first) = args.first() {
                        Type::Array(Box::new(first.clone()))
                    } else {
                        Type::Array(Box::new(Type::Basic("unknown".to_string())))
                    }
                } else {
                    Type::Generic(type_name.clone(), args.clone())
                }
            } else if type_name == "array" {
                if let Some(tps) = type_params {
                    if let Some(tp) = tps.first() {
                        Type::Array(Box::new(Type::TypeVar(tp.clone())))
                    } else {
                        Type::Array(Box::new(Type::Basic("unknown".to_string())))
                    }
                } else {
                    Type::Array(Box::new(Type::Basic("unknown".to_string())))
                }
            } else {
                match type_name.as_str() {
                    "number" | "string" | "boolean" | "bool" => Type::Basic(type_name.clone()),
                    _ => Type::Custom(type_name.clone()),
                }
            };

            for method in methods {
                let mut method_tps = base_tps.clone();
                if let Some(mtps) = method.type_params.as_ref() {
                    method_tps.extend(mtps.clone());
                }

                let full_params: Vec<(String, Type)> = method
                    .params
                    .iter()
                    .map(|(n, ty_opt)| {
                        if n == "self" {
                            (n.clone(), self_type.clone())
                        } else {
                            (
                                n.clone(),
                                ty_opt.clone().unwrap_or(Type::Basic("unknown".to_string())),
                            )
                        }
                    })
                    .collect();

                method_sigs.push(auwla_ast::ExtensionMethod {
                    type_params: if method_tps.is_empty() {
                        None
                    } else {
                        Some(method_tps)
                    },
                    name: method.name.clone(),
                    is_static: method.is_static,
                    params: full_params,
                    return_ty: method.return_ty.clone(),
                    attributes: method.attributes.clone(),
                    file: file_path.to_string(),
                    span: method.span.clone(),
                });
            }
            map.extensions
                .entry(type_key)
                .or_default()
                .extend(method_sigs);
        }
        _ => {}
    }
}

fn extend_key_simple(type_name: &str, type_args: &Option<Vec<Type>>) -> String {
    if let Some(args) = type_args {
        let parts: Vec<String> = args.iter().map(|a| format!("{}", a)).collect();
        format!("{}<{}>", type_name, parts.join(", "))
    } else {
        type_name.to_string()
    }
}
