use tower_lsp::lsp_types::Position;

/// Convert a byte offset in source text to an LSP `Position` (line, column).
pub fn byte_to_position(source: &str, byte: usize) -> Position {
    let safe = byte.min(source.len());
    let prefix = &source[..safe];
    let line = prefix.lines().count().max(1) - 1;
    let col = prefix.rfind('\n').map(|i| safe - i - 1).unwrap_or(safe);
    Position::new(line as u32, col as u32)
}

/// Format an `auwla_ast::Type` as a human-readable string.
pub fn format_type(ty: &auwla_ast::Type) -> String {
    match ty {
        auwla_ast::Type::Basic(name) => name.clone(),
        auwla_ast::Type::Custom(name) => name.clone(),
        auwla_ast::Type::Array(inner) => format!("array<{}>", format_type(inner)),
        auwla_ast::Type::Optional(inner) => format!("{}?", format_type(inner)),
        auwla_ast::Type::Result { ok_type, err_type } => {
            format!("{}?{}", format_type(ok_type), format_type(err_type))
        }
        auwla_ast::Type::Generic(name, args) => {
            let parts: Vec<String> = args.iter().map(format_type).collect();
            format!("{}< {}>", name, parts.join(", "))
        }
        auwla_ast::Type::Function(params, ret) => {
            let ps: Vec<String> = params.iter().map(format_type).collect();
            format!("fn({}) -> {}", ps.join(", "), format_type(ret))
        }
        auwla_ast::Type::TypeVar(name) => name.clone(),
        auwla_ast::Type::InferenceVar(id) => format!("_{}", id),
        auwla_ast::Type::SelfType => "Self".to_string(),
    }
}

/// Format an `ExtensionMethod` as a full signature string like
/// `fn name(param: type, ...) -> return_type`.
pub fn format_method_signature(method: &auwla_ast::ExtensionMethod) -> String {
    let params_str: Vec<String> = method
        .params
        .iter()
        .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
        .collect();
    let ret_str = method
        .return_ty
        .as_ref()
        .map(|r| format!(" -> {}", format_type(r)))
        .unwrap_or_default();
    format!("fn {}({}){}", method.name, params_str.join(", "), ret_str)
}

/// Extract the word (identifier) at the given character offset in a line.
/// Returns an empty string if no word is found at the offset.
pub fn get_word_at_offset(line: &str, char_idx: usize) -> &str {
    let bytes = line.as_bytes();
    if bytes.is_empty() || char_idx > bytes.len() {
        return "";
    }
    let idx = char_idx.min(bytes.len().saturating_sub(1));

    let is_word_byte = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

    let mut start = idx;
    while start > 0 && is_word_byte(bytes[start - 1]) {
        start -= 1;
    }
    // If we're not on a word char, maybe we're one past the end
    if start <= idx && idx < bytes.len() && !is_word_byte(bytes[idx]) && start > 0 {
        start = idx;
    }

    let mut end = if idx < bytes.len() && is_word_byte(bytes[idx]) {
        idx
    } else if start < idx {
        start
    } else {
        return "";
    };

    // Expand start backwards
    while start > 0 && is_word_byte(bytes[start - 1]) {
        start -= 1;
    }
    // Expand end forwards
    while end < bytes.len() && is_word_byte(bytes[end]) {
        end += 1;
    }

    &line[start..end]
}
