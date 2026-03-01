use crate::Spanned;

pub type Pattern = Spanned<PatternKind>;

/// A match arm — either a single expression or a block with a final expression.
/// For block arms the last expression (no semicolon) is the yielded value.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternKind {
    /// E.g. "admin", 42, true
    Literal(Expr),
    /// E.g. Banned(reason)
    Variant { name: String, bindings: Vec<String> },
    /// E.g. _
    Wildcard,
    /// E.g. "click" | "tap"
    Or(Vec<Pattern>),
    /// E.g. 1..5 or 'a'..<'z'
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    /// A catch-all variable binding, e.g. `count` or `other`
    Variable(String),
    /// Destructured Object format, e.g User { role: "admin", name } or { age, name }
    Struct(Option<String>, Vec<(String, Option<Pattern>)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    /// Optional preliminary statements (only in block-style arms)
    pub stmts: Vec<crate::stmt::Stmt>,
    /// The yielded value expression (None means yields Void)
    pub result: Option<Box<Expr>>,
}

pub type Expr = Spanned<ExprKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
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
    /// none(error_value) or none() for Optionals
    None(Option<Box<Expr>>),
    /// A function call (e.g., add::<number>(1, 2))
    Call {
        name: String,
        type_args: Option<Vec<crate::types::Type>>,
        args: Vec<Expr>,
    },
    /// A unary operation (e.g., !flag, -x)
    Unary { op: UnaryOp, expr: Box<Expr> },
    /// match expr { variant(bind) => arm, ... }
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
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
    /// Name::<T> { field: expr, ... }
    StructInit {
        name: String,
        type_args: Option<Vec<crate::types::Type>>,
        fields: Vec<(String, Expr)>,
    },
    /// EnumName::<T>::VariantName(args)
    EnumInit {
        enum_name: String,
        type_args: Option<Vec<crate::types::Type>>,
        variant_name: String,
        args: Vec<Expr>,
    },
    /// expr.property
    PropertyAccess { expr: Box<Expr>, property: String },
    /// expr.method::<T>(args) — may be an extension call or a closure field call
    MethodCall {
        expr: Box<Expr>,
        method: String,
        type_args: Option<Vec<crate::types::Type>>,
        args: Vec<Expr>,
    },
    /// <T>(x: T) => x * 2
    Closure {
        type_params: Option<Vec<String>>,
        params: Vec<(String, Option<crate::types::Type>)>,
        return_ty: Option<crate::types::Type>,
        body: Box<Expr>,
    },
    /// { stmt1; stmt2; expr }
    Block(Vec<crate::stmt::Stmt>, Option<Box<Expr>>),
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
