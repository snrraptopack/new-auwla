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
    #[token("enum")]
    Enum,
    #[token("import")]
    Import,
    #[token("export")]
    Export,
    #[token("from")]
    From,
    #[token("extend")]
    Extend,
    #[token("type")]
    Type,
    #[token("array")]
    Array,

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
    #[token("::")]
    DoubleColon,
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
    #[token("|")]
    Pipe,
    #[token("!")]
    Not,
    #[token("=>")]
    FatArrow,
    #[token("@")]
    At,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Let => write!(f, "let"),
            Token::Var => write!(f, "var"),
            Token::Fn => write!(f, "fn"),
            Token::Return => write!(f, "return"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Some => write!(f, "some"),
            Token::None => write!(f, "none"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Match => write!(f, "match"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Struct => write!(f, "struct"),
            Token::Enum => write!(f, "enum"),
            Token::Import => write!(f, "import"),
            Token::Export => write!(f, "export"),
            Token::From => write!(f, "from"),
            Token::Extend => write!(f, "extend"),
            Token::Type => write!(f, "type"),
            Token::Array => write!(f, "array"),
            Token::Ident(name) => write!(f, "{}", name),
            Token::StringLit(s) => write!(f, "\"{}\"", s),
            Token::InterpStart => write!(f, "interp_start"),
            Token::StringFragment(s) => write!(f, "{}", s),
            Token::InterpEnd => write!(f, "interp_end"),
            Token::NumberLit(n) => write!(f, "{}", n),
            Token::CharLit(c) => write!(f, "'{}'", c),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::DoubleColon => write!(f, "::"),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::QuestionMark => write!(f, "?"),
            Token::DotDotLt => write!(f, "..<"),
            Token::DotDot => write!(f, ".."),
            Token::Dot => write!(f, "."),
            Token::Assign => write!(f, "="),
            Token::Eq => write!(f, "=="),
            Token::Neq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Lte => write!(f, "<="),
            Token::Gte => write!(f, ">="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Pipe => write!(f, "|"),
            Token::Not => write!(f, "!"),
            Token::FatArrow => write!(f, "=>"),
            Token::At => write!(f, "@"),
        }
    }
}
