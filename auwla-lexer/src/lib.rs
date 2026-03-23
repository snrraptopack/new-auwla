pub mod token;

use logos::Logos;
use token::Token;

/// Lexes the given source code string into a vector of Tokens and their corresponding byte spans.
/// String literals containing `{expr}` are post-processed into interpolation tokens.
pub fn lex(source: &str) -> Vec<(Token, std::ops::Range<usize>)> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(source);

    while let Some(res) = lexer.next() {
        match res {
            Ok(token) => {
                let span = lexer.span();
                // Post-process string literals for interpolation
                if let Token::StringLit(ref s) = token {
                    if s.contains('{') {
                        // Split into interpolation tokens
                        let interp_tokens = expand_interpolation(s, &span, source);
                        tokens.extend(interp_tokens);
                        continue;
                    }
                }
                tokens.push((token, span));
            }
            Err(_) => {
                // Emit an Error token for the unrecognized character so the
                // parser can report it instead of working on an incomplete stream.
                let span = lexer.span();
                let bad = source[span.clone()].to_string();
                tokens.push((Token::Error(bad), span));
            }
        }
    }
    tokens
}

/// Expands a string like `Hello {name}! You are {age} years old.`
/// into: InterpStart, StringFragment("Hello "), <name tokens>, StringFragment("! You are "),
///       <age tokens>, StringFragment(" years old."), InterpEnd
fn expand_interpolation(
    s: &str,
    span: &std::ops::Range<usize>,
    _source: &str,
) -> Vec<(Token, std::ops::Range<usize>)> {
    let mut result = Vec::new();
    result.push((Token::InterpStart, span.clone()));

    let mut i = 0;
    let bytes = s.as_bytes();
    let mut frag_start = 0;

    let s_offset = span.start + 1; // skip leading quote

    while i < bytes.len() {
        if bytes[i] == b'{' {
            // Emit the text before this `{` as a fragment
            let frag = &s[frag_start..i];
            if !frag.is_empty() {
                let frag_span = (s_offset + frag_start)..(s_offset + i);
                result.push((Token::StringFragment(frag.to_string()), frag_span));
            }
            // Find the matching `}`
            let expr_start = i + 1;
            let mut depth = 1;
            let mut j = expr_start;
            let mut in_string = false;
            let mut in_char = false;

            while j < bytes.len() && depth > 0 {
                if in_string {
                    if bytes[j] == b'\\' {
                        j += 1;
                    } else if bytes[j] == b'"' {
                        in_string = false;
                    }
                } else if in_char {
                    if bytes[j] == b'\\' {
                        j += 1;
                    } else if bytes[j] == b'\'' {
                        in_char = false;
                    }
                } else {
                    match bytes[j] {
                        b'"' => in_string = true,
                        b'\'' => in_char = true,
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                }
                
                if depth > 0 {
                    j += 1;
                }
            }
            // j points to the closing `}`
            let expr_src = &s[expr_start..j];
            // Lex the expression inside braces
            let inner_tokens = lex_inner_expr(expr_src, s_offset + expr_start);
            result.extend(inner_tokens);
            i = j + 1;
            frag_start = i;
        } else {
            i += 1;
        }
    }

    // Emit trailing text (guard against out-of-bounds from unclosed braces)
    if frag_start < s.len() {
        let frag = &s[frag_start..];
        if !frag.is_empty() {
            let frag_span = (s_offset + frag_start)..(s_offset + s.len());
            result.push((Token::StringFragment(frag.to_string()), frag_span));
        }
    }

    result.push((Token::InterpEnd, span.clone()));
    result
}

/// Lex an expression string from inside `{...}` interpolation braces
fn lex_inner_expr(
    expr_src: &str,
    offset: usize,
) -> Vec<(Token, std::ops::Range<usize>)> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(expr_src);
    while let Some(res) = lexer.next() {
        let span = lexer.span();
        let adj_span = (offset + span.start)..(offset + span.end);
        match res {
            Ok(token) => tokens.push((token, adj_span)),
            Err(_) => {
                let bad = expr_src[span.clone()].to_string();
                tokens.push((Token::Error(bad), adj_span));
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lexing() {
        let source = r#"
            let name = "Auwla";
            var count = 10;
            if count > 5 {
                return name;
            }
        "#;

        let tokens: Vec<Token> = lex(source).into_iter().map(|(t, _)| t).collect();

        use Token::*;
        assert_eq!(
            tokens,
            vec![
                Let,
                Ident("name".to_string()),
                Assign,
                StringLit("Auwla".to_string()),
                Semicolon,
                Var,
                Ident("count".to_string()),
                Assign,
                NumberLit("10".to_string()),
                Semicolon,
                If,
                Ident("count".to_string()),
                Gt,
                NumberLit("5".to_string()),
                LBrace,
                Return,
                Ident("name".to_string()),
                Semicolon,
                RBrace
            ]
        );
    }

    #[test]
    fn test_interpolation_lexing() {
        let source = r#""Hello {name}!""#;
        let tokens: Vec<Token> = lex(source).into_iter().map(|(t, _)| t).collect();

        use Token::*;
        assert_eq!(
            tokens,
            vec![
                InterpStart,
                StringFragment("Hello ".to_string()),
                Ident("name".to_string()),
                StringFragment("!".to_string()),
                InterpEnd,
            ]
        );
    }
}
