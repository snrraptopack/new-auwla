use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;
use crate::utils::{format_method_signature, get_word_at_offset};

/// Implements the hover handler for the Auwla Language Server.
pub async fn handle_hover(backend: &Backend, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params
        .text_document_position_params
        .text_document
        .uri
        .to_string();
    let position = params.text_document_position_params.position;

    let content = if let Some(c) = backend.documents.get(&uri) {
        c.clone()
    } else {
        return Ok(None);
    };

    // Get the word under the cursor for extension method lookup
    let lines: Vec<&str> = content.lines().collect();
    let word = lines
        .get(position.line as usize)
        .map(|line| get_word_at_offset(line, position.character as usize))
        .unwrap_or_default();

    // Calculate byte offset
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

    // Shadow-compile the document for type info
    let lexed = auwla_lexer::lex(&content);
    let token_byte_spans: Vec<std::ops::Range<usize>> =
        lexed.iter().map(|(_, s)| s.clone()).collect();
    let tokens: Vec<_> = lexed
        .into_iter()
        .filter(|(t, _)| !matches!(t, auwla_lexer::token::Token::Error(_)))
        .map(|(t, _)| t)
        .collect();

    let ast = match auwla_parser::parse(tokens) {
        Ok(a) => a,
        _ => return Ok(None),
    };

    let mut typechecker = auwla_typechecker::Typechecker::new();
    for entry in backend.metadata.iter() {
        typechecker
            .extensions
            .insert(entry.key().clone(), entry.value().clone());
    }
    let _ = typechecker.check_program(&ast);

    // --- Node type lookup: find the tightest span containing the cursor ---
    if let Some(hover) = try_node_type_hover(&typechecker, &token_byte_spans, byte_offset) {
        return Ok(Some(hover));
    }

    // --- Fallback: look up variable/function names from scopes ---
    if !word.is_empty() {
        if let Some(hover) = try_scope_hover(&typechecker, word) {
            return Ok(Some(hover));
        }
    }

    // --- Fallback: search extension methods by word ---
    if !word.is_empty() {
        if let Some(hover) = try_extension_hover(backend, word) {
            return Ok(Some(hover));
        }
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Helper: hover from node_types (tightest span match)
// ---------------------------------------------------------------------------
fn try_node_type_hover(
    typechecker: &auwla_typechecker::Typechecker,
    token_byte_spans: &[std::ops::Range<usize>],
    byte_offset: usize,
) -> Option<Hover> {
    let mut best_type: Option<auwla_ast::Type> = None;
    let mut best_span_len = usize::MAX;

    for (tok_span, ty) in typechecker.node_types.iter() {
        let b_start = token_byte_spans
            .get(tok_span.start)
            .map(|r| r.start)
            .unwrap_or(0);
        let b_end = token_byte_spans
            .get(tok_span.end.saturating_sub(1))
            .map(|r| r.end)
            .unwrap_or(0);

        if b_start <= byte_offset && byte_offset <= b_end {
            let span_len = b_end - b_start;
            if span_len < best_span_len {
                best_span_len = span_len;
                best_type = Some(ty.clone());
            }
        }
    }

    let ty = best_type?;
    let type_key = typechecker.type_to_key(&ty);

    // Resolve type alias if applicable
    let resolved_key = typechecker
        .type_aliases
        .get(&type_key)
        .map(|rt| typechecker.type_to_key(rt));

    let mut markdown = String::new();
    if let Some(ref resolved) = resolved_key {
        markdown.push_str(&format!(
            "```auwla\ntype {} = {}\n```\n\n",
            type_key, resolved
        ));
    } else {
        markdown.push_str(&format!("```auwla\n{}\n```\n\n", type_key));
    }

    // Show struct fields
    let struct_key = resolved_key.as_deref().unwrap_or(&type_key);
    append_struct_fields(&mut markdown, typechecker, struct_key);

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown,
        }),
        range: None,
    })
}

// ---------------------------------------------------------------------------
// Helper: hover from variable/function scope lookup
// ---------------------------------------------------------------------------
fn try_scope_hover(typechecker: &auwla_typechecker::Typechecker, word: &str) -> Option<Hover> {
    // Check variables
    for scope in typechecker.scopes.iter().rev() {
        if let Some(ty) = scope.variables.get(word) {
            let type_key = typechecker.type_to_key(ty);

            let resolved_key = typechecker
                .type_aliases
                .get(&type_key)
                .map(|rt| typechecker.type_to_key(rt));

            let mut markdown = String::new();
            if let Some(ref resolved) = resolved_key {
                markdown.push_str(&format!(
                    "```auwla\nvar {}: {} // alias for {}\n```\n\n",
                    word, type_key, resolved
                ));
            } else {
                markdown.push_str(&format!("```auwla\nvar {}: {}\n```\n\n", word, type_key));
            }

            let struct_key = resolved_key.as_deref().unwrap_or(&type_key);
            append_struct_fields(&mut markdown, typechecker, struct_key);

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: markdown,
                }),
                range: None,
            });
        }
    }

    // Check functions
    for scope in typechecker.scopes.iter().rev() {
        if let Some((_, params, ret)) = scope.functions.get(word) {
            let ret_str = ret
                .as_ref()
                .map(|r| typechecker.type_to_key(r))
                .unwrap_or_else(|| "void".to_string());
            let params_str: Vec<String> =
                params.iter().map(|p| typechecker.type_to_key(p)).collect();
            let markdown = format!(
                "```auwla\nfn {}({}) -> {}\n```\n",
                word,
                params_str.join(", "),
                ret_str
            );
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: markdown,
                }),
                range: None,
            });
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Helper: hover from extension method metadata
// ---------------------------------------------------------------------------
fn try_extension_hover(backend: &Backend, word: &str) -> Option<Hover> {
    for entry in backend.metadata.iter() {
        let type_key = entry.key();
        let methods = entry.value();
        if let Some(method) = methods.iter().find(|m| m.name == word) {
            let sig = format_method_signature(method);
            let mut markdown = format!("### Extension: `{}.{}`\n\n", type_key, method.name);

            if !method.attributes.is_empty() {
                markdown.push_str("**Attributes:**\n");
                for attr in &method.attributes {
                    markdown.push_str(&format!("- `@{}({:?})`\n", attr.name, attr.args));
                }
                markdown.push('\n');
            }

            markdown.push_str(&format!("```auwla\n{}\n```\n\n", sig));

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: markdown,
                }),
                range: None,
            });
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Shared: append struct field listing to markdown
// ---------------------------------------------------------------------------
fn append_struct_fields(
    markdown: &mut String,
    typechecker: &auwla_typechecker::Typechecker,
    struct_key: &str,
) {
    if let Some(fields) = typechecker.structs.get(struct_key) {
        markdown.push_str(&format!("```auwla\nstruct {} {{\n", struct_key));
        for (fname, ftype) in fields {
            markdown.push_str(&format!(
                "  {}: {}\n",
                fname,
                typechecker.type_to_key(ftype)
            ));
        }
        markdown.push_str("}\n```\n");
    }
}
