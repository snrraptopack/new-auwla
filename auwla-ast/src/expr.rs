#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal string (e.g., "hello")
    StringLit(String),
    /// A literal number (e.g., 42, 3.14)
    NumberLit(f64),
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
}
