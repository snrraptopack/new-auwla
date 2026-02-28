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
    /// let { name, age } = user;
    DestructureLet {
        bindings: Vec<String>,
        initializer: Expr,
    },
    /// var x = 5;
    Var {
        name: String,
        ty: Option<Type>,
        initializer: Expr,
    },
    /// target = 10;
    Assign { target: Expr, value: Expr },
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
    /// struct Name { field: type, ... }
    StructDecl {
        name: String,
        fields: Vec<(String, Type)>, // name, type
    },
    /// enum Name { Variant1, Variant2(type) }
    EnumDecl {
        name: String,
        variants: Vec<(String, Vec<Type>)>,
    },
    /// import { add, Vec2 } from './math';
    Import { names: Vec<String>, path: String },
    /// export fn / export let / export struct / export enum
    Export { stmt: Box<Stmt> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
