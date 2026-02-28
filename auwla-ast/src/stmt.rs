use crate::expr::Expr;
use crate::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// let x: string?string = "hello";
    Let {
        name: String,
        ty: Option<Type>,
        initializer: Expr,
    },
    /// var x = 5;
    Var {
        name: String,
        ty: Option<Type>,
        initializer: Expr,
    },
    /// x = 10;
    Assign { name: String, value: Expr },
    /// fn add(a: number, b: number): number { ... }
    Fn {
        name: String,
        params: Vec<(String, Type)>, // name, type
        return_ty: Option<Type>,
        body: Vec<Stmt>,
    },
    /// return expr;
    Return(Option<Expr>),
    /// if count > 0 { ... } else { ... }
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    /// An expression evaluated for its side effects (or final value)
    Expr(Expr),
    /// while condition { body }
    While { condition: Expr, body: Vec<Stmt> },
    /// for binding in iterable { body }
    For {
        binding: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
