use tower_lsp::lsp_types::Position;

/// Convert a byte offset in source text to an LSP `Position` (line, column).
pub fn byte_to_position(source: &str, byte: usize) -> Position {
    let safe = byte.min(source.len());

    let mut line = 0u32;
    let mut line_start = 0usize;
    for (idx, ch) in source.char_indices() {
        if idx >= safe {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = idx + 1;
        }
    }

    let utf16_col = source[line_start..safe].encode_utf16().count() as u32;
    Position::new(line, utf16_col)
}

/// Convert an LSP `Position` (UTF-16 line/character) to a byte offset.
pub fn position_to_byte_offset(source: &str, position: Position) -> usize {
    let mut current_line = 0u32;
    let mut line_start = 0usize;

    for (idx, ch) in source.char_indices() {
        if current_line == position.line {
            break;
        }
        if ch == '\n' {
            current_line += 1;
            line_start = idx + 1;
        }
    }

    if current_line < position.line {
        return source.len();
    }

    let line_end = source[line_start..]
        .find('\n')
        .map(|off| line_start + off)
        .unwrap_or(source.len());
    let line = &source[line_start..line_end];

    let mut utf16_count = 0u32;
    let mut byte_in_line = line.len();
    for (idx, ch) in line.char_indices() {
        if utf16_count >= position.character {
            byte_in_line = idx;
            break;
        }

        let w = ch.len_utf16() as u32;
        if utf16_count + w > position.character {
            byte_in_line = idx;
            break;
        }

        utf16_count += w;
        if utf16_count == position.character {
            byte_in_line = idx + ch.len_utf8();
            break;
        }
    }

    line_start + byte_in_line
}

/// Format an `auwla_ast::Type` as a human-readable string.
pub fn format_type(ty: &auwla_ast::Type) -> String {
    match ty {
        auwla_ast::Type::Basic(name) => name.clone(),
        auwla_ast::Type::Custom(name) => name.clone(),
        auwla_ast::Type::Array(inner) => format!("array<{}>", format_type(inner)),
        auwla_ast::Type::Dict(k, v) => format!("dict<{}, {}>", format_type(k), format_type(v)),
        auwla_ast::Type::Optional(inner) => format!("{}?", format_type(inner)),
        auwla_ast::Type::Result { ok_type, err_type } => {
            format!("{}?{}", format_type(ok_type), format_type(err_type))
        }
        auwla_ast::Type::Generic(name, args) => {
            let parts: Vec<String> = args.iter().map(format_type).collect();
            format!("{}< {}>", name, parts.join(", "))
        }
        auwla_ast::Type::Function(params, ret) => {
            let ps: Vec<String> = params
                .iter()
                .map(|(ty, is_vararg)| {
                    if *is_vararg {
                        format!("...{}", format_type(ty))
                    } else {
                        format_type(ty)
                    }
                })
                .collect();
            format!("fn({}) -> {}", ps.join(", "), format_type(ret))
        }
        auwla_ast::Type::TypeVar(name) => name.clone(),
        auwla_ast::Type::InferenceVar(id) => format!("_{}", id),
        auwla_ast::Type::SelfType => "Self".to_string(),
        auwla_ast::Type::Wrapper(inner) => format!("wrapper<{}>", format_type(inner)),
        auwla_ast::Type::Tuple(types) => {
            let parts: Vec<String> = types.iter().map(format_type).collect();
            format!("({})", parts.join(", "))
        }
    }
}

/// Format an `ExtensionMethod` as a full signature string like
/// `fn name(param: type, ...) -> return_type`.
pub fn format_method_signature(method: &auwla_ast::ExtensionMethod) -> String {
    let params_str: Vec<String> = method
        .params
        .iter()
        .map(|(name, ty, is_vararg)| {
            if *is_vararg {
                format!("...{}: {}", name, format_type(ty))
            } else {
                format!("{}: {}", name, format_type(ty))
            }
        })
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

/// Extract identifier at a given LSP position using UTF-16 aware cursor math.
pub fn get_word_at_position(source: &str, position: Position) -> String {
    let byte = position_to_byte_offset(source, position);

    let line_start = source[..byte.min(source.len())]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let line_end = source[byte.min(source.len())..]
        .find('\n')
        .map(|off| byte.min(source.len()) + off)
        .unwrap_or(source.len());

    let line = &source[line_start..line_end];
    let local_byte = byte.saturating_sub(line_start).min(line.len());
    get_word_at_offset(line, local_byte).to_string()
}
