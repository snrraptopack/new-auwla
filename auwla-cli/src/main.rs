use auwla_ast::Program;
use auwla_codegen::emit_js;
use auwla_lexer::lex;
use auwla_parser::parse;
use auwla_typechecker::Typechecker;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 { &args[1] } else { "app.aw" };

    let path = Path::new(target);

    if !path.exists() {
        eprintln!("[Error] Path '{}' does not exist.", target);
        std::process::exit(1);
    }

    if path.is_file() {
        if let Err(_) = compile_file(path, Path::new("output.js")) {
            std::process::exit(1);
        }
    } else if path.is_dir() {
        println!("Running all tests in directory: {}", target);
        let mut passed = 0;
        let mut failed = 0;

        let output_dir = path.join("output");
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir).expect("Failed to create output directory");
        }

        let entries = match fs::read_dir(path) {
            Ok(iter) => iter,
            Err(e) => {
                eprintln!("[Error] Failed to read directory '{}': {}", target, e);
                std::process::exit(1);
            }
        };

        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("aw") {
                println!("\nTesting: {}", file_path.display());

                let file_stem = file_path.file_stem().unwrap();
                let output_file_path = output_dir.join(file_stem).with_extension("js");

                if compile_file(&file_path, &output_file_path).is_ok() {
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

fn compile_file(path: &Path, output_file: &Path) -> Result<(), ()> {
    let source = match fs::read_to_string(path) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("[Error] Could not read '{}': {}", path.display(), e);
            return Err(());
        }
    };

    let lexed = lex(&source);
    let spans: Vec<std::ops::Range<usize>> = lexed.iter().map(|(_, s)| s.clone()).collect();
    let tokens: Vec<_> = lexed.into_iter().map(|(t, _)| t).collect();

    let ast: Program = match parse(tokens) {
        Ok(program) => program,
        Err(errs) => {
            eprintln!("╔═══════════════════════════════════╗");
            eprintln!("║       Auwla  ─  Syntax Error       ║");
            eprintln!("╚═══════════════════════════════════╝");
            for e in errs {
                let span = e.span();
                let byte_start = spans.get(span.start).map(|r| r.start).unwrap_or(0);
                let byte_end = spans
                    .get(span.end.saturating_sub(1))
                    .map(|r| r.end)
                    .unwrap_or(source.len());

                let (line, col) = byte_to_line_col(&source, byte_start);
                let snippet = source[byte_start..byte_end.min(source.len())].trim();

                eprintln!(
                    "→  Line {}, Col {}: Unexpected token '{}'.",
                    line, col, snippet
                );

                let expected: Vec<_> = e.expected().filter_map(|t| t.as_ref()).cloned().collect();

                use auwla_lexer::token::Token;
                if expected.contains(&Token::Semicolon) {
                    let prev_byte = spans
                        .get(span.start.saturating_sub(1))
                        .map(|r| r.end.saturating_sub(1))
                        .unwrap_or(0);
                    let (prev_line, _) = byte_to_line_col(&source, prev_byte);
                    let prev_line_src = source
                        .lines()
                        .nth(prev_line.saturating_sub(1))
                        .unwrap_or("")
                        .trim();
                    eprintln!(
                        "   Hint: Did you forget a semicolon `;` at the end of line {}?",
                        prev_line
                    );
                    eprintln!("         `{}`", prev_line_src);
                } else if !expected.is_empty() {
                    let names: Vec<_> = expected.iter().map(|t| format!("{:?}", t)).collect();
                    eprintln!("   Expected one of: {}", names.join(", "));
                }
            }
            return Err(());
        }
    };

    if std::env::var("AUWLA_DEBUG").is_ok() {
        println!("--- Parsed AST ---\n{:#?}\n---\n", ast);
    }

    let mut typechecker = Typechecker::new();
    match typechecker.check_program(&ast) {
        Ok(_) => {
            println!("✓  Typechecking passed — no errors found.");

            let js_output = emit_js(&ast);
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
            eprintln!("╔═══════════════════════════════════╗");
            eprintln!("║       Auwla  ─  Type Error         ║");
            eprintln!("╚═══════════════════════════════════╝");
            eprintln!("→  {}", e);
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
