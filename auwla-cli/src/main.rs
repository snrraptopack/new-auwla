use auwla_ast::Program;
use auwla_codegen::emit_js;
use auwla_error::{Diagnostic, Level};
use auwla_lexer::lex;
use auwla_parser::parse;
use auwla_typechecker::{ExportMap, Typechecker, collect_exports};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 { &args[1] } else { "app.aw" };

    let path = Path::new(target);

    if !path.exists() {
        eprintln!("[Error] Path '{}' does not exist.", target);
        std::process::exit(1);
    }

    if path.is_file() {
        // Single-file mode — no import resolution needed
        if let Err(_) = compile_file_standalone(path, Path::new("output.js")) {
            std::process::exit(1);
        }
    } else if path.is_dir() {
        let is_module_dir = has_module_structure(path);

        if is_module_dir {
            // Multi-file project: use the full module pipeline
            println!("Compiling module project: {}", target);
            let output_dir = path.join("output");
            fs::create_dir_all(&output_dir).expect("Failed to create output directory");
            if let Err(_) = compile_directory_as_module(path, &output_dir) {
                std::process::exit(1);
            }
        } else {
            // Test-runner mode: compile independent .aw files in a flat directory
            println!("Running all tests in directory: {}", target);
            let mut passed = 0;
            let mut failed = 0;

            let output_dir = path.join("output");
            if !output_dir.exists() {
                fs::create_dir_all(&output_dir).expect("Failed to create output directory");
            }

            let mut entries: Vec<PathBuf> = fs::read_dir(path)
                .expect("Failed to read directory")
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("aw"))
                .collect();
            entries.sort();

            for file_path in &entries {
                println!("\nTesting: {}", file_path.display());
                let file_stem = file_path.file_stem().unwrap();
                let output_file_path = output_dir.join(file_stem).with_extension("js");
                if compile_file_standalone(file_path, &output_file_path).is_ok() {
                    passed += 1;
                } else {
                    failed += 1;
                }
            }

            // Also discover and compile any module subdirectories
            let subdirs: Vec<PathBuf> = fs::read_dir(path)
                .expect("Failed to read directory")
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir() && p.file_name().map(|n| n != "output").unwrap_or(false))
                .collect();

            for subdir in &subdirs {
                if has_module_structure(subdir) {
                    println!("\n--- Module project: {} ---", subdir.display());
                    let sub_output = subdir.join("output");
                    fs::create_dir_all(&sub_output).expect("Failed to create subdir output");
                    if compile_directory_as_module(subdir, &sub_output).is_ok() {
                        passed += 1;
                    } else {
                        failed += 1;
                    }
                }
            }

            println!("\n=============================");
            println!("Test Results: {} passed, {} failed", passed, failed);
            if failed > 0 {
                std::process::exit(1);
            }
        }
    }
}

/// Returns true if the directory contains any .aw file with import/export statements,
/// indicating it should be compiled as a multi-file module project.
fn has_module_structure(dir: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("aw") {
                if let Ok(src) = fs::read_to_string(&p) {
                    let trimmed = src.trim_start();
                    if trimmed.starts_with("import")
                        || trimmed.contains("\nimport")
                        || trimmed.starts_with("export")
                        || trimmed.contains("\nexport")
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────
// Multi-file module pipeline
// ─────────────────────────────────────────────────────────────

fn compile_directory_as_module(dir: &Path, output_dir: &Path) -> Result<(), ()> {
    // 1. Parse all .aw files
    let mut file_asts: HashMap<String, Program> = HashMap::new(); // canonical_key -> ast
    let mut file_paths: HashMap<String, PathBuf> = HashMap::new(); // canonical_key -> path
    let mut file_sources: HashMap<String, String> = HashMap::new(); // canonical_key -> source
    let mut file_token_spans: HashMap<String, Vec<std::ops::Range<usize>>> = HashMap::new();

    let entries: Vec<PathBuf> = fs::read_dir(dir)
        .expect("Failed to read dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("aw"))
        .collect();

    for file_path in &entries {
        let key = file_key(dir, file_path);
        let source = match fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[Error] Could not read '{}': {}", file_path.display(), e);
                return Err(());
            }
        };
        let (ast, token_byte_spans) = match parse_source(&source, file_path) {
            Ok(res) => res,
            Err(_) => return Err(()),
        };
        file_asts.insert(key.clone(), ast);
        file_paths.insert(key.clone(), file_path.clone());
        file_sources.insert(key.clone(), source);
        file_token_spans.insert(key, token_byte_spans);
    }

    // 2. Build import graph: key -> [keys it depends on]
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    for (key, ast) in &file_asts {
        let mut file_deps = Vec::new();
        for stmt in &ast.statements {
            if let auwla_ast::StmtKind::Import { path, .. } = &stmt.node {
                // Resolve path relative to the file's directory
                let dep_key = resolve_import_key(dir, path);
                file_deps.push(dep_key);
            }
        }
        deps.insert(key.clone(), file_deps);
    }

    // 3. Topological sort using Kahn's algorithm
    let sorted_keys = match topological_sort(&deps) {
        Ok(order) => order,
        Err(cycle) => {
            eprintln!("[Error] Circular import detected involving: {}", cycle);
            return Err(());
        }
    };

    // 4. First pass: collect exports from each file in dependency order
    let mut export_maps: HashMap<String, ExportMap> = HashMap::new();
    for key in &sorted_keys {
        if let Some(ast) = file_asts.get(key) {
            export_maps.insert(key.clone(), collect_exports(ast));
        }
    }

    // 5. Build the import context each file needs: path -> ExportMap
    //    For file F importing './math', the import context key is './math' (its raw import string)
    //    We need to map those raw strings to their ExportMaps.
    //    We resolve: for each file, for each of its imports, get the dep_key and its ExportMap.

    // 6. Second pass: typecheck + codegen each file
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut all_extensions = String::new();
    let mut util_needed = false;

    for key in &sorted_keys {
        if let (Some(ast), Some(file_path)) = (file_asts.get(key), file_paths.get(key)) {
            println!("\nCompiling: {}", file_path.display());

            // Build import context: raw_import_path -> ExportMap
            let mut import_ctx: HashMap<String, ExportMap> = HashMap::new();
            for stmt in &ast.statements {
                if let auwla_ast::StmtKind::Import { path: raw_path, .. } = &stmt.node {
                    let dep_key = resolve_import_key(dir, raw_path);
                    if let Some(map) = export_maps.get(&dep_key) {
                        import_ctx.insert(raw_path.clone(), map.clone());
                    }
                }
            }

            let mut typechecker = Typechecker::new();
            match typechecker.check_program_with_imports(ast, &import_ctx) {
                Ok(_) => {
                    println!("✓  Typechecking passed — no errors found.");
                    let (mut js_output, ext_output) = emit_js(ast, typechecker.get_extensions());
                    all_extensions.push_str(&ext_output);

                    let mut import_prefix = String::new();
                    if js_output.contains("__print(") {
                        util_needed = true;
                        import_prefix.push_str("import { __print } from './__util.js';\n");
                    }
                    if js_output.contains("__ext_") {
                        import_prefix.push_str("import * as __auwla from './__runtime.js';\n");
                        js_output = js_output.replace("__ext_", "__auwla.__ext_");
                    }
                    if !import_prefix.is_empty() {
                        js_output = format!("{}{}", import_prefix, js_output);
                    }

                    let stem = file_path.file_stem().unwrap();
                    let out_path = output_dir.join(stem).with_extension("js");
                    fs::write(&out_path, &js_output).unwrap_or_else(|e| {
                        eprintln!("[Error] Failed to write '{}': {}", out_path.display(), e);
                    });
                    println!(
                        "✓  Generated '{}' ({} bytes)",
                        out_path.display(),
                        js_output.len()
                    );
                    success_count += 1;
                }
                Err(e) => {
                    let source = file_sources.get(key).unwrap();
                    let token_byte_spans = file_token_spans.get(key).unwrap();

                    let byte_start = token_byte_spans
                        .get(e.span.start)
                        .map(|r| r.start)
                        .unwrap_or(0);
                    let byte_end = token_byte_spans
                        .get(e.span.end.saturating_sub(1))
                        .map(|r| r.end)
                        .unwrap_or(source.len());

                    Diagnostic::new(Level::Error, "Type Error", file_path.to_string_lossy())
                        .with_label(byte_start..byte_end, e.message.clone())
                        .emit(source);
                    fail_count += 1;
                }
            }
        }
    }

    if !all_extensions.is_empty() {
        let runtime_path = output_dir.join("__runtime.js");
        fs::write(&runtime_path, &all_extensions).unwrap_or_else(|e| {
            eprintln!("[Error] Failed to write '__runtime.js': {}", e);
        });
        println!(
            "✓  Generated '__runtime.js' ({} bytes)",
            all_extensions.len()
        );
    }
    if util_needed {
        let util_path = output_dir.join("__util.js");
        let contents = util_js_source();
        fs::write(&util_path, contents).unwrap_or_else(|e| {
            eprintln!("[Error] Failed to write '__util.js': {}", e);
        });
        println!("✓  Generated '__util.js' ({} bytes)", contents.len());
    }

    println!("\n=============================");
    println!("Results: {} compiled, {} failed", success_count, fail_count);
    if fail_count > 0 { Err(()) } else { Ok(()) }
}

/// Canonical key for a file relative to the project directory.
/// e.g., dir = "tests/modules", file = "tests/modules/math.aw" -> "./math"
fn file_key(_dir: &Path, file: &Path) -> String {
    let stem = file.file_stem().unwrap().to_string_lossy();
    format!("./{}", stem)
}

/// Resolve a raw import path like `'./math'` to its file key.
fn resolve_import_key(_dir: &Path, raw: &str) -> String {
    // Normalize: strip .aw extension if present
    if raw.ends_with(".aw") {
        raw[..raw.len() - 3].to_string()
    } else {
        raw.to_string()
    }
}

/// Kahn's algorithm topological sort.
/// Returns files in dependency order (dependencies first).
fn topological_sort(deps: &HashMap<String, Vec<String>>) -> Result<Vec<String>, String> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut reverse: HashMap<String, Vec<String>> = HashMap::new(); // node -> files that depend on it

    for (node, _) in deps {
        in_degree.entry(node.clone()).or_insert(0);
        reverse.entry(node.clone()).or_default();
    }
    for (node, node_deps) in deps {
        for dep in node_deps {
            *in_degree.entry(dep.clone()).or_insert(0) += 0; // ensure dep is in map
            in_degree.entry(node.clone()).or_insert(0);
            reverse.entry(dep.clone()).or_default().push(node.clone());
        }
    }

    // Compute proper in-degrees (number of deps each file has)
    let mut real_in_degree: HashMap<String, usize> = HashMap::new();
    for (node, node_deps) in deps {
        real_in_degree.entry(node.clone()).or_insert(0);
        for dep in node_deps {
            real_in_degree.entry(node.clone()).and_modify(|d| *d += 1);
            // make sure dep is in map
            real_in_degree.entry(dep.clone()).or_insert(0);
        }
    }

    let mut queue: VecDeque<String> = real_in_degree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(k, _)| k.clone())
        .collect();

    let mut sorted = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    while let Some(node) = queue.pop_front() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node.clone());
        sorted.push(node.clone());
        if let Some(dependents) = reverse.get(&node) {
            for dep in dependents {
                let entry = real_in_degree.entry(dep.clone()).or_insert(1);
                if *entry > 0 {
                    *entry -= 1;
                }
                if *entry == 0 {
                    queue.push_back(dep.clone());
                }
            }
        }
    }

    if sorted.len() < deps.len() {
        let unresolved: Vec<_> = deps
            .keys()
            .filter(|k| !visited.contains(*k))
            .cloned()
            .collect();
        return Err(unresolved.join(", "));
    }

    Ok(sorted)
}

// ─────────────────────────────────────────────────────────────
// Single-file helpers (unchanged from before)
// ─────────────────────────────────────────────────────────────

fn compile_file_standalone(path: &Path, output_file: &Path) -> Result<(), ()> {
    let source = match fs::read_to_string(path) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("[Error] Could not read '{}': {}", path.display(), e);
            return Err(());
        }
    };

    let (ast, token_byte_spans) = parse_source(&source, path)?;

    if std::env::var("AUWLA_DEBUG").is_ok() {
        println!("--- Parsed AST ---\n{:#?}\n---\n", ast);
    }

    let mut typechecker = Typechecker::new();
    match typechecker.check_program(&ast) {
        Ok(_) => {
            println!("✓  Typechecking passed — no errors found.");
            let (mut js_output, ext_output) = emit_js(&ast, typechecker.get_extensions());

            if !ext_output.is_empty() {
                let out_dir = output_file.parent().unwrap_or(Path::new("."));
                let runtime_path = out_dir.join("__runtime.js");
                fs::write(&runtime_path, &ext_output).unwrap_or_else(|e| {
                    eprintln!("[Error] Failed to write '__runtime.js': {}", e);
                });
                println!("✓  Generated '__runtime.js' ({} bytes)", ext_output.len());
            }

            let mut import_prefix = String::new();
            if js_output.contains("__print(") {
                let out_dir = output_file.parent().unwrap_or(Path::new("."));
                let util_path = out_dir.join("__util.js");
                let contents = util_js_source();
                fs::write(&util_path, contents).unwrap_or_else(|e| {
                    eprintln!("[Error] Failed to write '__util.js': {}", e);
                });
                println!("✓  Generated '__util.js' ({} bytes)", contents.len());
                import_prefix.push_str("import { __print } from './__util.js';\n");
            }
            if js_output.contains("__ext_") {
                import_prefix.push_str("import * as __auwla from './__runtime.js';\n");
                js_output = js_output.replace("__ext_", "__auwla.__ext_");
            }
            if !import_prefix.is_empty() {
                js_output = format!("{}{}", import_prefix, js_output);
            }

            fs::write(output_file, &js_output).unwrap_or_else(|e| {
                eprintln!("[Error] Failed to write '{}': {}", output_file.display(), e);
            });
            println!(
                "✓  Generated '{}' ({} bytes)",
                output_file.display(),
                js_output.len()
            );
            Ok(())
        }
        Err(e) => {
            let byte_start = token_byte_spans
                .get(e.span.start)
                .map(|r| r.start)
                .unwrap_or(0);
            let byte_end = token_byte_spans
                .get(e.span.end.saturating_sub(1))
                .map(|r| r.end)
                .unwrap_or(source.len());

            Diagnostic::new(Level::Error, "Type Error", path.to_string_lossy())
                .with_label(byte_start..byte_end, e.message)
                .emit(&source);
            Err(())
        }
    }
}

fn util_js_source() -> &'static str {
    "export function __print(...args) {\n  const format = (val, top = false) => {\n    if (val && typeof val === 'object' && 'ok' in val) {\n      if (val.ok) return `some(${format(val.value)})`;\n      if ('value' in val) return `none(${format(val.value)})`;\n      return 'none';\n    }\n    if (Array.isArray(val)) return `[${val.map(v => format(v)).join(', ')}]`;\n    if (typeof val === 'string' && !top) return `\"${val}\"`;\n    if (typeof val === 'object' && val !== null) {\n      const props = Object.entries(val).map(([k, v]) => `${k}: ${format(v)}`).join(', ');\n      return `{ ${props} }`;\n    }\n    return val;\n  };\n  console.log(...args.map(a => format(a, true)));\n}\n"
}

fn parse_source(source: &str, path: &Path) -> Result<(Program, Vec<std::ops::Range<usize>>), ()> {
    let lexed = lex(source);
    let spans: Vec<std::ops::Range<usize>> = lexed.iter().map(|(_, s)| s.clone()).collect();
    let tokens: Vec<_> = lexed.into_iter().map(|(t, _)| t).collect();

    match parse(tokens) {
        Ok(program) => Ok((program, spans)),
        Err(errs) => {
            for e in errs {
                let span = e.span();
                let byte_start = spans.get(span.start).map(|r| r.start).unwrap_or(0);
                let byte_end = spans
                    .get(span.end.saturating_sub(1))
                    .map(|r| r.end)
                    .unwrap_or(source.len());

                let mut diag =
                    Diagnostic::new(Level::Error, "Syntax Error", path.to_string_lossy());

                let expected: Vec<_> = e.expected().filter_map(|t| t.as_ref()).cloned().collect();

                let message = if let Some(found) = e.found() {
                    format!("Unexpected token '{}'", found)
                } else {
                    "Unexpected end of input".to_string()
                };

                use auwla_lexer::token::Token;
                if expected.contains(&Token::Semicolon) {
                    let prev_idx = span.start.saturating_sub(1);
                    if let Some(prev_span) = spans.get(prev_idx) {
                        diag = diag.with_label(
                            prev_span.end..prev_span.end,
                            "Expected ';' after this token",
                        );

                        let (prev_line, _) = byte_to_line_col(source, prev_span.start);
                        diag = diag.with_help(format!(
                            "Did you forget a semicolon ';' at the end of line {}?",
                            prev_line
                        ));
                    }
                }

                diag = diag.with_label(byte_start..byte_end, message);

                if !expected.is_empty() && !expected.contains(&Token::Semicolon) {
                    let names: Vec<_> = expected.iter().map(|t| format!("{}", t)).collect();
                    diag = diag.with_help(format!("Expected one of: {}", names.join(", ")));
                }

                diag.emit(&source);
            }
            Err(())
        }
    }
}

/// Converts a byte offset in `source` to a (line, col) pair (both 1-indexed).
fn byte_to_line_col(source: &str, byte: usize) -> (usize, usize) {
    let safe = byte.min(source.len());
    let prefix = &source[..safe];
    let line = prefix.lines().count().max(1);
    let col = prefix.rfind('\n').map(|i| safe - i).unwrap_or(safe + 1);
    (line, col)
}
