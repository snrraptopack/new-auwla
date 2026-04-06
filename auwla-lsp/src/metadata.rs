use auwla_ast::{ExtensionOrigin, Program, StmtKind};
use auwla_lexer::lex;
use auwla_parser::parse_recovery;
use dashmap::DashMap;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct MetadataCache {
    /// The global view of all extensions: type_name -> Vec<ExtensionMethod>
    pub global_metadata: Arc<DashMap<String, Vec<auwla_ast::ExtensionMethod>>>,

    /// file path -> list of (type_name, [methods from this file])
    /// This allows us to easily remove a file's old contributions when it changes.
    pub file_contributions: DashMap<PathBuf, Vec<(String, Vec<auwla_ast::ExtensionMethod>)>>,
}

impl MetadataCache {
    fn genericize_type(ty: &auwla_ast::Type, type_params: &[String]) -> auwla_ast::Type {
        use auwla_ast::Type;

        match ty {
            Type::Custom(name) if type_params.contains(name) => Type::TypeVar(name.clone()),
            Type::Array(inner) => Type::Array(Box::new(Self::genericize_type(inner, type_params))),
            Type::Dict(k, v) => Type::Dict(
                Box::new(Self::genericize_type(k, type_params)),
                Box::new(Self::genericize_type(v, type_params)),
            ),
            Type::Optional(inner) => {
                Type::Optional(Box::new(Self::genericize_type(inner, type_params)))
            }
            Type::Result { ok_type, err_type } => Type::Result {
                ok_type: Box::new(Self::genericize_type(ok_type, type_params)),
                err_type: Box::new(Self::genericize_type(err_type, type_params)),
            },
            Type::Generic(name, args) => Type::Generic(
                name.clone(),
                args.iter()
                    .map(|a| Self::genericize_type(a, type_params))
                    .collect(),
            ),
            Type::Function(params, ret) => Type::Function(
                params
                    .iter()
                    .map(|(p, is_v)| (Self::genericize_type(p, type_params), *is_v))
                    .collect(),
                Box::new(Self::genericize_type(ret, type_params)),
            ),
            Type::Tuple(types) => Type::Tuple(
                types
                    .iter()
                    .map(|t| Self::genericize_type(t, type_params))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }

    fn substitute_type_var(
        ty: &auwla_ast::Type,
        var_name: &str,
        replacement: &auwla_ast::Type,
    ) -> auwla_ast::Type {
        use auwla_ast::Type;

        match ty {
            Type::Custom(name) if name == var_name => replacement.clone(),
            Type::TypeVar(name) if name == var_name => replacement.clone(),
            Type::Array(inner) => {
                Type::Array(Box::new(Self::substitute_type_var(inner, var_name, replacement)))
            }
            Type::Dict(k, v) => Type::Dict(
                Box::new(Self::substitute_type_var(k, var_name, replacement)),
                Box::new(Self::substitute_type_var(v, var_name, replacement)),
            ),
            Type::Optional(inner) => Type::Optional(Box::new(Self::substitute_type_var(
                inner,
                var_name,
                replacement,
            ))),
            Type::Result { ok_type, err_type } => Type::Result {
                ok_type: Box::new(Self::substitute_type_var(ok_type, var_name, replacement)),
                err_type: Box::new(Self::substitute_type_var(err_type, var_name, replacement)),
            },
            Type::Generic(name, args) => Type::Generic(
                name.clone(),
                args.iter()
                    .map(|a| Self::substitute_type_var(a, var_name, replacement))
                    .collect(),
            ),
            Type::Function(params, ret) => Type::Function(
                params
                    .iter()
                    .map(|(p, is_v)| (Self::substitute_type_var(p, var_name, replacement), *is_v))
                    .collect(),
                Box::new(Self::substitute_type_var(ret, var_name, replacement)),
            ),
            Type::Tuple(types) => Type::Tuple(
                types
                    .iter()
                    .map(|t| Self::substitute_type_var(t, var_name, replacement))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }

    fn resolve_self_type(ty: &auwla_ast::Type, self_ty: &auwla_ast::Type) -> auwla_ast::Type {
        Self::substitute_type_var(ty, "Self", self_ty)
    }

    pub fn new(global_metadata: Arc<DashMap<String, Vec<auwla_ast::ExtensionMethod>>>) -> Self {
        Self {
            global_metadata,
            file_contributions: DashMap::new(),
        }
    }

    /// Recursively scan a workspace folder for all `.aw` files and build the cache.
    pub fn scan_workspace(&self, root: &Path) {
        let mut extensions_by_file = HashMap::new();

        for entry in WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                // Skip std folder and output folders
                let path = e.path();
                if path.components().any(|c| {
                    c.as_os_str() == "std" 
                    || c.as_os_str() == "output" 
                    || c.as_os_str() == "test_output"
                    || c.as_os_str() == "target"
                }) {
                    return false;
                }
                path.extension().map_or(false, |ext| ext == "aw")
            })
        {
            let path: PathBuf = entry.path().to_path_buf();
            if let Ok(content) = fs::read_to_string(&path) {
                let module_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let file_exts = Self::extract_extensions_from_text(&content, &module_name);
                extensions_by_file.insert(path, file_exts);
            }
        }

        self.apply_all(extensions_by_file);
    }

    /// Parse a single file's text and extract `extend` blocks.
    pub fn update_from_content(&self, file_path: &Path, content: &str) {
        let module_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let new_exts = Self::extract_extensions_from_text(content, &module_name);

        self.update_file(file_path.to_path_buf(), new_exts);
    }

    fn extract_extensions_from_text(
        text: &str,
        module_name: &str,
    ) -> HashMap<String, Vec<auwla_ast::ExtensionMethod>> {
        let tokens: Vec<_> = lex(text).into_iter().map(|(t, _)| t).collect();
        let (ast_opt, _) = parse_recovery(tokens);

        if let Some(ast) = ast_opt {
            Self::extract_extensions_from_ast(&ast, module_name)
        } else {
            HashMap::new()
        }
    }

    fn extract_extensions_from_ast(
        ast: &Program,
        _module_name: &str,
    ) -> HashMap<String, Vec<auwla_ast::ExtensionMethod>> {
        let mut extracted = HashMap::new();

        for stmt in &ast.statements {
            if let StmtKind::Extend {
                type_params,
                target_type,
                methods,
            } = &stmt.node
            {
                let base_tps = type_params.clone().unwrap_or_default();
                let self_type = Self::genericize_type(target_type, &base_tps);
                let type_key = target_type.base_key();
                for method in methods {
                    let mut all_tps = base_tps.clone();
                    if let Some(method_tps) = &method.type_params {
                        all_tps.extend(method_tps.clone());
                    }

                    // Convert AST Method to ExtensionMethod manually
                    let params = method
                        .params
                        .iter()
                        .map(|(n, t, is_v)| {
                            let resolved_ty = if n == "self" {
                                if let Some(explicit_ty) = t {
                                    let generic_ty = Self::genericize_type(explicit_ty, &all_tps);
                                    Self::resolve_self_type(&generic_ty, &self_type)
                                } else {
                                    self_type.clone()
                                }
                            } else {
                                let ty = t
                                    .clone()
                                    .unwrap_or(auwla_ast::Type::InferenceVar(0));
                                let generic_ty = Self::genericize_type(&ty, &all_tps);
                                Self::resolve_self_type(&generic_ty, &self_type)
                            };

                            (
                                n.clone(),
                                resolved_ty,
                                *is_v,
                            )
                        })
                        .collect();

                    let return_ty = method.return_ty.as_ref().map(|ret| {
                        let generic_ret = Self::genericize_type(ret, &all_tps);
                        Self::resolve_self_type(&generic_ret, &self_type)
                    });

                    let combined_tps = if all_tps.is_empty() {
                        None
                    } else {
                        Some(all_tps)
                    };

                    let ext_method = auwla_ast::ExtensionMethod {
                        type_params: combined_tps,
                        name: method.name.clone(),
                        is_static: method.is_static,
                        params,
                        return_ty,
                        attributes: method.attributes.clone(),
                        span: method.span.clone(),
                        origin: ExtensionOrigin::User,
                    };

                    extracted
                        .entry(type_key.clone())
                        .or_insert_with(Vec::new)
                        .push(ext_method);
                }
            }
        }

        extracted
    }

    /// Replace the metadata contributions for a specific file.
    fn update_file(
        &self,
        file_path: PathBuf,
        new_contributions: HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
    ) {
        // 1. Remove old contributions for this file from global_metadata
        if let Some((_, old_contributions)) = self.file_contributions.remove(&file_path) {
            for (type_name, old_methods) in old_contributions {
                if let Some(mut global_entry) = self.global_metadata.get_mut(&type_name) {
                    global_entry.retain(|m| !old_methods.iter().any(|om| om.name == m.name));
                }
            }
        }

        // 2. Add new contributions to global_metadata
        let mut new_saved_contributions = Vec::new();
        for (type_name, methods) in new_contributions {
            let mut global_entry = self
                .global_metadata
                .entry(type_name.clone())
                .or_insert_with(Vec::new);

            for method in &methods {
                // Determine if we should replace or append. We just append for now,
                // or optionally remove an existing one with the same name if we want shadowing.
                global_entry.retain(|m| m.name != method.name); // Simple overwrite if same name
                global_entry.push(method.clone());
            }

            new_saved_contributions.push((type_name, methods));
        }

        // 3. Save new tracking info for this file
        self.file_contributions
            .insert(file_path, new_saved_contributions);
    }

    fn apply_all(
        &self,
        all_files: HashMap<PathBuf, HashMap<String, Vec<auwla_ast::ExtensionMethod>>>,
    ) {
        for (path, contributions) in all_files {
            self.update_file(path, contributions);
        }
    }
}
