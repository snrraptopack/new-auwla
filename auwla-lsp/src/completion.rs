use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use std::collections::HashMap;

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
    } else {
        handle_general_completion(backend, &content, &mut items);
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.dedup_by(|a, b| a.label == b.label);
    Ok(Some(CompletionResponse::Array(items)))
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

    let lexed = auwla_lexer::lex(&shadow);
    let token_byte_spans: Vec<std::ops::Range<usize>> =
        lexed.iter().map(|(_, s)| s.clone()).collect();
    let tokens: Vec<_> = lexed
        .into_iter()
        .filter(|(t, _)| !matches!(t, auwla_lexer::token::Token::Error(_)))
        .map(|(t, _)| t)
        .collect();

    if let Ok(ast) = auwla_parser::parse(tokens) {
        let mut typechecker = create_typechecker_with_metadata(backend);
        run_lenient_typecheck(&mut typechecker, &ast);

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
            let base_key = ty.base_key();

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

            push_methods_from_registry(&typechecker.extensions, &type_key, items);
            push_methods_for_type_key(backend, &type_key, items);
            if base_key != type_key {
                push_methods_from_registry(&typechecker.extensions, &base_key, items);
                push_methods_for_type_key(backend, &base_key, items);
            }
            return;
        }

        if let Some(type_key) = infer_type_key_before_dot(content, dot_idx) {
            push_methods_for_type_key(backend, &type_key, items);
            return;
        }

        if let Some((exact_key, base_key)) = infer_variable_type_keys_before_dot(backend, content, dot_idx) {
            push_methods_for_type_key(backend, &exact_key, items);
            if base_key != exact_key {
                push_methods_for_type_key(backend, &base_key, items);
            }
        }
        return;
    }

    if let Some(type_key) = infer_type_key_before_dot(content, dot_idx) {
        push_methods_for_type_key(backend, &type_key, items);
        return;
    }

    if let Some((exact_key, base_key)) = infer_variable_type_keys_before_dot(backend, content, dot_idx) {
        push_methods_for_type_key(backend, &exact_key, items);
        if base_key != exact_key {
            push_methods_for_type_key(backend, &base_key, items);
        }
    }
}

fn push_methods_for_type_key(backend: &Backend, type_key: &str, items: &mut Vec<CompletionItem>) {
    if let Some(methods) = backend.metadata.get(type_key) {
        for method in methods.value() {
            let sig = format_method_signature(method);
            items.push(CompletionItem {
                label: method.name.clone(),
                detail: Some(sig.clone()),
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

fn push_methods_from_registry(
    registry: &HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
    type_key: &str,
    items: &mut Vec<CompletionItem>,
) {
    if let Some(methods) = registry.get(type_key) {
        for method in methods {
            let sig = format_method_signature(method);
            items.push(CompletionItem {
                label: method.name.clone(),
                detail: Some(sig.clone()),
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

fn infer_type_key_before_dot(content: &str, dot_idx: usize) -> Option<String> {
    if dot_idx > content.len() {
        return None;
    }

    let line_start = content[..dot_idx]
        .rfind(|c| c == '\n' || c == '\r')
        .map(|i| i + 1)
        .unwrap_or(0);

    let receiver_src = content[line_start..dot_idx].trim_end();
    if receiver_src.is_empty() {
        return None;
    }

    let receiver_tokens: Vec<_> = auwla_lexer::lex(receiver_src)
        .into_iter()
        .map(|(t, _)| t)
        .filter(|t| !matches!(t, auwla_lexer::token::Token::Error(_)))
        .collect();

    match receiver_tokens.last() {
        Some(auwla_lexer::token::Token::StringLit(_)) => Some("string".to_string()),
        Some(auwla_lexer::token::Token::NumberLit(_)) => Some("number".to_string()),
        Some(auwla_lexer::token::Token::True) | Some(auwla_lexer::token::Token::False) => {
            Some("bool".to_string())
        }
        Some(auwla_lexer::token::Token::CharLit(_)) => Some("char".to_string()),
        Some(auwla_lexer::token::Token::Ident(name)) => {
            let lowered = name.to_ascii_lowercase();
            match lowered.as_str() {
                "string" | "number" | "bool" | "char" | "array" | "dict" | "optional"
                | "result" => Some(lowered),
                _ => None,
            }
        }
        _ => None,
    }
}

fn infer_variable_type_keys_before_dot(
    backend: &Backend,
    content: &str,
    dot_idx: usize,
) -> Option<(String, String)> {
    if dot_idx > content.len() {
        return None;
    }

    let line_start = content[..dot_idx]
        .rfind(|c| c == '\n' || c == '\r')
        .map(|i| i + 1)
        .unwrap_or(0);
    let receiver_src = content[line_start..dot_idx].trim_end();
    if receiver_src.is_empty() {
        return None;
    }

    let receiver_tokens: Vec<_> = auwla_lexer::lex(receiver_src)
        .into_iter()
        .map(|(t, _)| t)
        .filter(|t| !matches!(t, auwla_lexer::token::Token::Error(_)))
        .collect();

    let ident = match receiver_tokens.last() {
        Some(auwla_lexer::token::Token::Ident(name)) => name.clone(),
        _ => return None,
    };

    let mut prefix = content[..dot_idx].to_string();
    if !prefix.trim_end().ends_with(';') {
        prefix.push(';');
    }

    let tokens: Vec<_> = auwla_lexer::lex(&prefix)
        .into_iter()
        .map(|(t, _)| t)
        .filter(|t| !matches!(t, auwla_lexer::token::Token::Error(_)))
        .collect();

    let (ast_opt, _) = auwla_parser::parse_recovery(tokens);
    let ast = ast_opt?;

    let mut typechecker = create_typechecker_with_metadata(backend);
    run_lenient_typecheck(&mut typechecker, &ast);

    for scope in typechecker.scopes.iter().rev() {
        if let Some(ty) = scope.variables.get(&ident) {
            return Some((typechecker.type_to_key(ty), ty.base_key()));
        }
    }

    None
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
    let lexed = auwla_lexer::lex(content);
    let tokens: Vec<_> = lexed
        .into_iter()
        .filter(|(t, _)| !matches!(t, auwla_lexer::token::Token::Error(_)))
        .map(|(t, _)| t)
        .collect();

    if let Ok(ast) = auwla_parser::parse(tokens) {
        let mut typechecker = create_typechecker_with_metadata(backend);
        run_lenient_typecheck(&mut typechecker, &ast);

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
                            .map(|(ty, is_vararg)| {
                                if *is_vararg {
                                    format!("...{}", typechecker.type_to_key(ty))
                                } else {
                                    typechecker.type_to_key(ty)
                                }
                            })
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

fn create_typechecker_with_metadata(backend: &Backend) -> auwla_typechecker::Typechecker {
    let mut typechecker = auwla_typechecker::Typechecker::new();
    for entry in backend.metadata.iter() {
        typechecker
            .extensions
            .insert(entry.key().clone(), entry.value().clone());
    }
    typechecker
}

fn run_lenient_typecheck(
    typechecker: &mut auwla_typechecker::Typechecker,
    ast: &auwla_ast::Program,
) {
    for stmt in &ast.statements {
        let _ = typechecker.check_stmt(stmt);
    }
}
