pub mod token;

use logos::Logos;
use token::Token;

/// Lexes the given source code string into a vector of Tokens and their corresponding byte spans.
/// If an unrecognizable token is found, it currently just skips it (Logos behavior when returning `Result`).
/// For simplicity, we filter out errors in this basic implementation or map them appropriately.
pub fn lex(source: &str) -> Vec<(Token, std::ops::Range<usize>)> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(source);

    while let Some(res) = lexer.next() {
        match res {
            Ok(token) => tokens.push((token, lexer.span())),
            Err(_) => {
                // Ignore or collect errors here
                // For a robust compiler, we'd emit diagnostics.
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
}
