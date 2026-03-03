use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;
use crate::utils::format_method_signature;

/// Implements the completion handler for the Auwla Language Server.
pub async fn handle_completion(
    backend: &Backend,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri.to_string();
    let position = params.text_document_position.position;
    let mut items = Vec::new();

    let content = if let Some(c) = backend.documents.get(&uri) {
        c.clone()
    } else {
        return Ok(None);
    };

    // Calculate byte offset from (line, character) using raw bytes
    let byte_offset = calculate_byte_offset(&content, position);

    // Search backwards from cursor for a dot, but ONLY on the current line
    let dot_idx = find_dot_before_cursor(&content, byte_offset);

    if let Some(di) = dot_idx {
        handle_dot_completion(backend, &content, di, &mut items);
        if items.is_empty() {
            return Ok(Some(global_completions(backend)));
        }
    } else {
        handle_general_completion(backend, &content, &mut items);
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.dedup_by(|a, b| a.label == b.label);
    Ok(Some(CompletionResponse::Array(items)))
}

/// Build a fallback completion list from all extension methods across all types.
pub fn global_completions(backend: &Backend) -> CompletionResponse {
    let mut items = Vec::new();
    for entry in backend.metadata.iter() {
        let type_key = entry.key();
        for method in entry.value() {
            let sig = format_method_signature(method);
            items.push(CompletionItem {
                label: method.name.clone(),
                detail: Some(format!("{} ({})", sig, type_key)),
                kind: Some(CompletionItemKind::METHOD),
                ..Default::default()
            });
        }
    }
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.dedup_by(|a, b| a.label == b.label);
    CompletionResponse::Array(items)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn calculate_byte_offset(content: &str, position: Position) -> usize {
    let mut byte_offset = 0usize;
    let mut current_line = 0u32;
    for (i, byte) in content.as_bytes().iter().enumerate() {
        if current_line == position.line {
            byte_offset = i + position.character as usize;
            break;
        }
        if *byte == b'\n' {
            current_line += 1;
        }
    }
    if current_line < position.line {
        byte_offset = content.len();
    }
    byte_offset
}

fn find_dot_before_cursor(content: &str, byte_offset: usize) -> Option<usize> {
    let search_start = byte_offset.saturating_sub(1);
    if search_start >= content.len() {
        return None;
    }
    for i in (0..=search_start).rev() {
        let b = content.as_bytes()[i];
        if b == b'.' {
            return Some(i);
        }
        if b == b'\n' || b == b'\r' {
            break;
        }
    }
    None
}

/// Dot completion: resolve the type before the dot and show fields + methods.
fn handle_dot_completion(
    backend: &Backend,
    content: &str,
    dot_idx: usize,
    items: &mut Vec<CompletionItem>,
) {
    // Replace the dot with a space so the expression before it parses cleanly
    let mut shadow = String::with_capacity(content.len());
    shadow.push_str(&content[..dot_idx]);
    shadow.push(' ');
    shadow.push_str(&content[dot_idx + 1..]);

    let lexed = match std::panic::catch_unwind(|| auwla_lexer::lex(&shadow)) {
        Ok(l) => l,
        Err(_) => return,
    };
    let token_byte_spans: Vec<std::ops::Range<usize>> =
        lexed.iter().map(|(_, s)| s.clone()).collect();
    let tokens: Vec<_> = lexed.into_iter().map(|(t, _)| t).collect();

    if let Ok(ast) = auwla_parser::parse(tokens) {
        let mut typechecker = auwla_typechecker::Typechecker::new();
        for entry in backend.metadata.iter() {
            typechecker
                .extensions
                .insert(entry.key().clone(), entry.value().clone());
        }
        let _ = typechecker.check_program(&ast);

        // Find the expression whose byte span ends closest to (but before) the dot
        let mut best_fit: Option<auwla_ast::Type> = None;
        let mut best_byte_end = 0usize;

        for (tok_span, ty) in typechecker.node_types.iter() {
            let byte_end = token_byte_spans
                .get(tok_span.end.saturating_sub(1))
                .map(|r| r.end)
                .unwrap_or(0);
            if byte_end <= dot_idx && byte_end > best_byte_end {
                best_byte_end = byte_end;
                best_fit = Some(ty.clone());
            }
        }

        if let Some(ref ty) = best_fit {
            let type_key = typechecker.type_to_key(ty);

            // Add struct fields
            if let Some(fields) = typechecker.structs.get(&type_key) {
                for (field_name, field_type) in fields {
                    items.push(CompletionItem {
                        label: field_name.clone(),
                        detail: Some(format!(
                            "{}: {}",
                            field_name,
                            typechecker.type_to_key(field_type)
                        )),
                        kind: Some(CompletionItemKind::FIELD),
                        ..Default::default()
                    });
                }
            }

            // Add extension methods for this type
            if let Some(methods) = backend.metadata.get(&type_key) {
                for method in methods.value() {
                    let sig = format_method_signature(method);
                    items.push(CompletionItem {
                        label: method.name.clone(),
                        detail: Some(format!("extension for {}", type_key)),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("```auwla\n{}\n```", sig),
                        })),
                        kind: Some(CompletionItemKind::METHOD),
                        ..Default::default()
                    });
                }
            }
        }
    }
}

/// General (no-dot) completion: keywords, variables, functions, struct/enum names.
fn handle_general_completion(backend: &Backend, content: &str, items: &mut Vec<CompletionItem>) {
    // Keywords
    let keywords = [
        "let", "var", "fn", "return", "if", "else", "match", "while", "for", "in", "struct",
        "enum", "import", "export", "from", "extend", "type", "array", "true", "false", "some",
        "none", "print", "break", "continue",
    ];
    for kw in &keywords {
        items.push(CompletionItem {
            label: kw.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }

    // Shadow-compile to get variables, functions, structs, enums in scope
    let lexed = match std::panic::catch_unwind(|| auwla_lexer::lex(content)) {
        Ok(l) => l,
        Err(_) => return,
    };
    let tokens: Vec<_> = lexed.into_iter().map(|(t, _)| t).collect();

    if let Ok(ast) = auwla_parser::parse(tokens) {
        let mut typechecker = auwla_typechecker::Typechecker::new();
        for entry in backend.metadata.iter() {
            typechecker
                .extensions
                .insert(entry.key().clone(), entry.value().clone());
        }
        let _ = typechecker.check_program(&ast);

        for scope in &typechecker.scopes {
            for (name, ty) in &scope.variables {
                items.push(CompletionItem {
                    label: name.clone(),
                    detail: Some(typechecker.type_to_key(ty)),
                    kind: Some(CompletionItemKind::VARIABLE),
                    ..Default::default()
                });
            }
            for (name, (_, params, ret)) in &scope.functions {
                let ret_str = ret
                    .as_ref()
                    .map(|r| typechecker.type_to_key(r))
                    .unwrap_or_else(|| "void".to_string());
                items.push(CompletionItem {
                    label: name.clone(),
                    detail: Some(format!(
                        "fn({}) -> {}",
                        params
                            .iter()
                            .map(|p| typechecker.type_to_key(p))
                            .collect::<Vec<_>>()
                            .join(", "),
                        ret_str
                    )),
                    kind: Some(CompletionItemKind::FUNCTION),
                    ..Default::default()
                });
            }
        }

        for name in typechecker.structs.keys() {
            items.push(CompletionItem {
                label: name.clone(),
                detail: Some("struct".to_string()),
                kind: Some(CompletionItemKind::STRUCT),
                ..Default::default()
            });
        }
        for name in typechecker.enums.keys() {
            items.push(CompletionItem {
                label: name.clone(),
                detail: Some("enum".to_string()),
                kind: Some(CompletionItemKind::ENUM),
                ..Default::default()
            });
        }
    }
}
