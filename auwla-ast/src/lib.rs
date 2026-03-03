pub mod expr;
pub mod stmt;
pub mod types;

pub use expr::{BinaryOp, Expr, ExprKind, MatchArm, Pattern, PatternKind, UnaryOp};
pub use stmt::{Attribute, ExtensionMethod, ExtensionOrigin, Method, Program, Stmt, StmtKind};
pub use types::Type;

pub type Span = std::ops::Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}
