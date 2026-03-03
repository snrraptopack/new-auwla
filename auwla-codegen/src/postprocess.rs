/// Post-processing utilities for generated JavaScript output.
///
/// Centralizes the logic for prepending runtime/util imports that was
/// previously duplicated between `compile_file_standalone` and
/// `compile_directory_as_module` in the CLI.

/// Known std module type prefixes — maps type prefix in `_ext_` names to module name.
const STD_TYPE_MODULES: &[(&str, &str)] = &[
    ("string", "string"),
    ("array", "array"),
    ("number", "number"),
    ("Math", "math"),
];

/// Scan the generated JS for runtime dependencies (`__print`, `__range`,
/// `_ext_` calls) and prepend the appropriate import statements.
///
/// Returns `true` if `__util.js` is needed (i.e., `__print` or `__range` used).
pub fn add_runtime_imports(js: &mut String, rel_prefix: &str) -> bool {
    let mut import_prefix = String::new();
    let mut util_needed = false;
    let mut util_imports = Vec::new();

    if js.contains("__print(") {
        util_needed = true;
        util_imports.push("__print");
    }
    if js.contains("__range(") {
        util_needed = true;
        util_imports.push("__range");
    }
    if !util_imports.is_empty() {
        import_prefix.push_str(&format!(
            "import {{ {} }} from '{}/__util.js';\n",
            util_imports.join(", "),
            rel_prefix
        ));
    }

    if js.contains("_ext_") {
        // Determine which std modules are needed based on _ext_ prefixes
        let mut needed_std_modules = std::collections::HashSet::new();
        let mut needs_user_ext = false;

        // Scan for _ext_ references and classify them
        for line in js.lines() {
            let mut search_from = 0;
            while let Some(pos) = line[search_from..].find("_ext_") {
                let abs_pos = search_from + pos;
                let after = &line[abs_pos + 5..]; // skip "_ext_"
                let type_prefix = after.split("__").next().unwrap_or("");

                let mut found_std = false;
                for (prefix, module) in STD_TYPE_MODULES {
                    if type_prefix == *prefix || type_prefix.starts_with(prefix) {
                        needed_std_modules.insert(*module);
                        found_std = true;
                        break;
                    }
                }
                if !found_std && !type_prefix.is_empty() {
                    needs_user_ext = true;
                }

                search_from = abs_pos + 5;
            }
        }

        // Import std modules with namespace aliases
        for module in &needed_std_modules {
            import_prefix.push_str(&format!(
                "import * as __std_{} from '{}/std/{}.js';\n",
                module, rel_prefix, module
            ));
        }

        // Import user extensions if needed
        if needs_user_ext {
            import_prefix.push_str(&format!(
                "import * as __user from '{}/__user_ext.js';\n",
                rel_prefix
            ));
        }

        // Fallback: if we have _ext_ but couldn't classify
        if needed_std_modules.is_empty() && !needs_user_ext {
            import_prefix.push_str(&format!(
                "import * as __user from '{}/__user_ext.js';\n",
                rel_prefix
            ));
            needs_user_ext = true;
        }

        // Rewrite _ext_ calls to use the correct namespace.
        // We process std prefixes first (longest match first for safety),
        // then anything remaining is user-defined.

        // Sort std prefixes by length descending to match longest first
        let mut std_prefixes: Vec<(&str, &str)> = STD_TYPE_MODULES.to_vec();
        std_prefixes.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (prefix, module) in &std_prefixes {
            let search = format!("_ext_{}__", prefix);
            let replacement = format!("__std_{}._ext_{}__", module, prefix);
            *js = js.replace(&search, &replacement);
        }

        // Rewrite remaining _ext_ references as user extensions
        if needs_user_ext {
            let mut result = String::new();
            let mut remaining = js.as_str();
            while let Some(pos) = remaining.find("_ext_") {
                let before = &remaining[..pos];
                // Check if already rewritten (preceded by '.')
                if before.ends_with('.') {
                    result.push_str(&remaining[..pos + 5]);
                    remaining = &remaining[pos + 5..];
                } else {
                    result.push_str(before);
                    result.push_str("__user._ext_");
                    remaining = &remaining[pos + 5..];
                }
            }
            result.push_str(remaining);
            *js = result;
        }
    }

    if !import_prefix.is_empty() {
        *js = format!("{}{}", import_prefix, js);
    }

    util_needed
}
