/// Post-processing utilities for generated JavaScript output.
///
/// Centralizes the logic for prepending runtime/util imports that was
/// previously duplicated between `compile_file_standalone` and
/// `compile_directory_as_module` in the CLI.

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
        import_prefix.push_str(&format!(
            "import * as __auwla from '{}/__runtime.js';\n",
            rel_prefix
        ));
        *js = js.replace("_ext_", "__auwla._ext_");
    }

    if !import_prefix.is_empty() {
        *js = format!("{}{}", import_prefix, js);
    }

    util_needed
}
