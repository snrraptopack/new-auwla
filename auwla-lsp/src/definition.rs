use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;
use crate::utils::{byte_to_position, get_word_at_offset};

/// Implements go-to-definition for the Auwla Language Server.
pub async fn handle_definition(
    backend: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
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

    // Get the word under the cursor
    let lines: Vec<&str> = content.lines().collect();
    let word = lines
        .get(position.line as usize)
        .map(|line| get_word_at_offset(line, position.character as usize))
        .unwrap_or_default();

    if word.is_empty() {
        return Ok(None);
    }

    // Shadow-compile to populate the definitions map
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

    // Look up the word in the definitions map
    if let Some(def_span) = typechecker.definitions.get(word) {
        let byte_start = token_byte_spans
            .get(def_span.start)
            .map(|r| r.start)
            .unwrap_or(0);
        let byte_end = token_byte_spans
            .get(def_span.end.saturating_sub(1))
            .map(|r| r.end)
            .unwrap_or(byte_start);

        let start_pos = byte_to_position(&content, byte_start);
        let end_pos = byte_to_position(&content, byte_end);

        let target_uri = params.text_document_position_params.text_document.uri;

        return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
            target_uri,
            Range::new(start_pos, end_pos),
        ))));
    }

    Ok(None)
}
