use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
#[logos(skip r"[ \t\n\f]+")] // whitespace
#[logos(skip(r"//[^\n]*", allow_greedy = true))] // single-line comments
pub enum Token {
    // Keywords
    #[token("let")]
    Let,
    #[token("var")]
    Var,
    #[token("fn")]
    Fn,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("some")]
    Some,
    #[token("none")]
    None,

    // Identifiers
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLit(String),

    #[regex("[0-9]+([.][0-9]+)?", |lex| lex.slice().to_string())]
    NumberLit(String),

    // Symbols & Punctuation
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token("?")]
    QuestionMark,

    // Operators
    #[token("=")]
    Assign,
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Lte,
    #[token(">=")]
    Gte,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
}
