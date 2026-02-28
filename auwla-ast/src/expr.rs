/// A match arm — either a single expression or a block with a final expression.
/// For block arms the last expression (no semicolon) is the yielded value.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    /// The bound name for the inner value (e.g. `val` in `some(val) => ...`)
    pub binding: String,
    /// Optional preliminary statements (only in block-style arms)
    pub stmts: Vec<crate::stmt::Stmt>,
    /// The yielded value expression (None means yields Void)
    pub result: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Void / Unit value
    Void,
    /// A literal string (e.g., "hello")
    StringLit(String),
    /// A literal number (e.g., 42, 3.14)
    NumberLit(f64),
    /// A boolean literal (true or false)
    BoolLit(bool),
    /// A character literal (e.g., 'a')
    CharLit(char),
    /// A variable usage (e.g., my_var)
    Identifier(String),
    /// A binary operation (e.g., a + b)
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// some(value)
    Some(Box<Expr>),
    /// none(error_value)
    None(Box<Expr>),
    /// A function call (e.g., add(1, 2))
    Call { name: String, args: Vec<Expr> },
    /// A unary operation (e.g., !flag, -x)
    Unary { op: UnaryOp, expr: Box<Expr> },
    /// match expr { some(val) => arm, none(err) => arm }
    Match {
        expr: Box<Expr>,
        some_arm: MatchArm,
        none_arm: MatchArm,
    },
    /// Array literal: [1, 2, 3]
    Array(Vec<Expr>),
    /// Array indexing: arr[0]
    Index { expr: Box<Expr>, index: Box<Expr> },
    /// Range: 1..10 (inclusive) or 1..<10 (exclusive), also 'a'..'z'
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    /// String interpolation: "Hello {name}!" → vec![StringLit("Hello "), Identifier("name"), StringLit("!")]
    Interpolation(Vec<Expr>),
    /// Try operator: expr?("error") — unwraps some, or returns none(error) from enclosing fn
    Try {
        expr: Box<Expr>,
        error_expr: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not, // !
    Neg, // -
}
