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
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "aw"))
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
                type_name, methods, ..
            } = &stmt.node
            {
                for method in methods {
                    // Convert AST Method to ExtensionMethod manually
                    let params = method
                        .params
                        .iter()
                        .map(|(n, t)| {
                            (
                                n.clone(),
                                t.clone().unwrap_or(auwla_ast::Type::InferenceVar(0)),
                            )
                        })
                        .collect();

                    let ext_method = auwla_ast::ExtensionMethod {
                        type_params: method.type_params.clone(),
                        name: method.name.clone(),
                        is_static: method.is_static,
                        params,
                        return_ty: method.return_ty.clone(),
                        attributes: method.attributes.clone(),
                        span: method.span.clone(),
                        origin: ExtensionOrigin::User,
                    };

                    extracted
                        .entry(type_name.clone())
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
