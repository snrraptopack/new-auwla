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
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("match")]
    Match,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("struct")]
    Struct,

    // Identifiers
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLit(String),

    // String interpolation tokens (emitted by post-processing in lex())
    /// Start of an interpolated string
    InterpStart,
    /// A literal text fragment inside an interpolated string
    StringFragment(String),
    /// End of an interpolated string
    InterpEnd,

    #[regex("[0-9]+([.][0-9]+)?", |lex| lex.slice().to_string())]
    NumberLit(String),

    #[regex("'[^']'", |lex| lex.slice().chars().nth(1).unwrap())]
    CharLit(char),

    // Symbols & Punctuation
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token("?")]
    QuestionMark,

    // Operators — longer tokens first so logos picks them
    #[token("..<")]
    DotDotLt,
    #[token("..")]
    DotDot,
    #[token(".")]
    Dot,
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
    // Logical operators
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,
    #[token("=>")]
    FatArrow,
}
