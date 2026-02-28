use auwla_ast::Program;
use auwla_lexer::lex;
use auwla_parser::parse;
use auwla_typechecker::Typechecker;
use std::fs;

fn main() {
    let filename = "app.aw";

    println!("Reading and compiling {}...\n", filename);

    let source = match fs::read_to_string(filename) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("[Error] Could not read '{}': {}", filename, e);
            eprintln!("Tip: Create an `app.aw` file in your project root.");
            return;
        }
    };

    let lexed = lex(&source);
    // keep a mapping from token index → byte offset for error reporting
    let spans: Vec<std::ops::Range<usize>> = lexed.iter().map(|(_, s)| s.clone()).collect();
    let tokens: Vec<_> = lexed.into_iter().map(|(t, _)| t).collect();

    let ast: Program = match parse(tokens) {
        Ok(program) => program,
        Err(errs) => {
            eprintln!("╔═══════════════════════════════════╗");
            eprintln!("║       Auwla  ─  Syntax Error       ║");
            eprintln!("╚═══════════════════════════════════╝");
            for e in errs {
                // chumsky Simple<Token> gives us a span over token indices
                let span = e.span();
                // Try to map the token span back to a byte range in source
                let byte_start = spans.get(span.start).map(|r| r.start).unwrap_or(0);
                let byte_end = spans
                    .get(span.end.saturating_sub(1))
                    .map(|r| r.end)
                    .unwrap_or(source.len());

                // Compute line/col from byte offset
                let (line, col) = byte_to_line_col(&source, byte_start);
                let snippet = source[byte_start..byte_end.min(source.len())].trim();

                eprintln!(
                    "→  Line {}, Col {}: Unexpected token '{}'.",
                    line, col, snippet
                );

                let expected: Vec<_> = e.expected().filter_map(|t| t.as_ref()).cloned().collect();

                use auwla_lexer::token::Token;
                if expected.contains(&Token::Semicolon) {
                    // Walk back one token to find where the incomplete statement ended
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
            return;
        }
    };

    // Only show AST in debug mode (set AUWLA_DEBUG=1 to enable)
    if std::env::var("AUWLA_DEBUG").is_ok() {
        println!("--- Parsed AST ---\n{:#?}\n---\n", ast);
    }

    let mut typechecker = Typechecker::new();
    match typechecker.check_program(&ast) {
        Ok(_) => println!("✓  Typechecking passed — no errors found."),
        Err(e) => {
            eprintln!("╔═══════════════════════════════════╗");
            eprintln!("║       Auwla  ─  Type Error         ║");
            eprintln!("╚═══════════════════════════════════╝");
            eprintln!("→  {}", e);
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
