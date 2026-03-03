use tower_lsp::lsp_types::*;

use crate::Backend;
use crate::utils::byte_to_position;

/// Lex, parse, and type-check a document, then publish diagnostics to the client.
pub async fn analyze_document(backend: &Backend, uri: &str, content: &str) {
    let mut diagnostics = Vec::new();

    let lexed = match std::panic::catch_unwind(|| auwla_lexer::lex(content)) {
        Ok(l) => l,
        Err(_) => return,
    };
    let spans: Vec<std::ops::Range<usize>> = lexed.iter().map(|(_, s)| s.clone()).collect();
    let tokens: Vec<_> = lexed.into_iter().map(|(t, _)| t).collect();

    match auwla_parser::parse(tokens) {
        Ok(ast) => {
            handle_typechecker_errors(backend, &ast, &spans, content, &mut diagnostics);
        }
        Err(errs) => {
            handle_parser_errors(&errs, &spans, content, &mut diagnostics);
        }
    }

    if let Ok(url) = url::Url::parse(uri) {
        backend
            .client
            .publish_diagnostics(url, diagnostics, None)
            .await;
    }
}

// ---------------------------------------------------------------------------
// Typechecker errors
// ---------------------------------------------------------------------------
fn handle_typechecker_errors(
    backend: &Backend,
    ast: &auwla_ast::Program,
    spans: &[std::ops::Range<usize>],
    content: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut typechecker = auwla_typechecker::Typechecker::new();
    for entry in backend.metadata.iter() {
        typechecker
            .extensions
            .insert(entry.key().clone(), entry.value().clone());
    }

    if let Err(e) = typechecker.check_program(ast) {
        let byte_start = spans.get(e.span.start).map(|r| r.start).unwrap_or(0);
        let byte_end = spans
            .get(e.span.end.saturating_sub(1))
            .map(|r| r.end)
            .unwrap_or(content.len());

        diagnostics.push(Diagnostic {
            range: Range::new(
                byte_to_position(content, byte_start),
                byte_to_position(content, byte_end),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            message: e.message,
            source: Some("auwla".to_string()),
            ..Default::default()
        });
    }
}

// ---------------------------------------------------------------------------
// Parser errors
// ---------------------------------------------------------------------------
fn handle_parser_errors(
    errs: &[chumsky::error::Simple<auwla_lexer::token::Token>],
    spans: &[std::ops::Range<usize>],
    content: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for e in errs {
        let span = e.span();
        let byte_start = spans
            .get(span.start)
            .map(|r| r.start)
            .unwrap_or_else(|| content.len());
        let byte_end = spans
            .get(span.end.saturating_sub(1))
            .map(|r| r.end)
            .unwrap_or_else(|| content.len());

        let mut message = format_parser_error(e);

        // Check for missing semicolon
        use auwla_lexer::token::Token;
        let mut is_missing_semi = false;
        for expected_token in e.expected() {
            if let Some(token) = expected_token {
                if matches!(token, Token::Semicolon) {
                    message = "Missing semicolon ';' at the end of statement".to_string();
                    is_missing_semi = true;
                    break;
                }
            }
        }

        // Pin the error span precisely
        let (b_start, b_end) = if is_missing_semi && span.start > 0 {
            // Pin to the end of the previous token
            if let Some(prev_span) = spans.get(span.start - 1) {
                (prev_span.end, prev_span.end)
            } else {
                (byte_start, byte_end)
            }
        } else {
            // Highlight only the first offensive token
            let pinned_end = spans.get(span.start).map(|r| r.end).unwrap_or(byte_start);
            (byte_start, pinned_end)
        };

        diagnostics.push(Diagnostic {
            range: Range::new(
                byte_to_position(content, b_start),
                byte_to_position(content, b_end),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            message,
            source: Some("auwla".to_string()),
            ..Default::default()
        });
    }
}

fn format_parser_error(e: &chumsky::error::Simple<auwla_lexer::token::Token>) -> String {
    match e.reason() {
        chumsky::error::SimpleReason::Unclosed { delimiter, .. } => {
            format!("Unclosed delimiter '{}'", delimiter)
        }
        chumsky::error::SimpleReason::Unexpected => {
            let expected_str = if e.expected().len() == 0 {
                "something else".to_string()
            } else {
                let mut strings: Vec<String> = e
                    .expected()
                    .map(|ex| match ex {
                        Some(token) => token.to_string(),
                        None => "end of input".to_string(),
                    })
                    .collect();
                strings.sort();
                strings.dedup();
                strings.join(", ")
            };

            if let Some(found) = e.found() {
                format!("Unexpected token '{}', expected {}", found, expected_str)
            } else {
                format!("Unexpected end of input, expected {}", expected_str)
            }
        }
        chumsky::error::SimpleReason::Custom(msg) => msg.to_string(),
    }
}
