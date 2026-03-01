use crate::Spanned;
use crate::expr::Expr;
use crate::types::Type;
use serde::{Deserialize, Serialize};

pub type Stmt = Spanned<StmtKind>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionMethod {
    pub type_params: Option<Vec<String>>,
    pub name: String,
    pub is_static: bool,
    pub params: Vec<(String, Type)>,
    pub return_ty: Option<Type>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
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
    /// fn add<T>(a: T, b: T): T { ... }
    Fn {
        name: String,
        type_params: Option<Vec<String>>,
        params: Vec<(String, Type)>, // name, type
        return_ty: Option<Type>,
        body: Vec<Stmt>,
        attributes: Vec<Attribute>,
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
    /// struct Name<T> { field: T, ... }
    StructDecl {
        name: String,
        type_params: Option<Vec<String>>,
        fields: Vec<(String, Type)>, // name, type
    },
    /// enum Name<T> { Variant1, Variant2(T) }
    EnumDecl {
        name: String,
        type_params: Option<Vec<String>>,
        variants: Vec<(String, Vec<Type>)>,
    },
    /// import { add, Vec2 } from './math';
    Import { names: Vec<String>, path: String },
    /// export fn / export let / export struct / export enum
    Export { stmt: Box<Stmt> },
    /// extend<T> TypeName { fn method(self, ...) { ... } }
    Extend {
        type_params: Option<Vec<String>>,
        type_args: Option<Vec<Type>>,
        /// The type being extended — can be a built-in ("number", "string") or a custom struct name.
        type_name: String,
        methods: Vec<Method>,
    },
    /// type Name<T> = Result<T, string>;
    TypeAlias {
        name: String,
        type_params: Option<Vec<String>>,
        aliased_type: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

/// A method defined inside an `extend` block.
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub name: String,
    pub attributes: Vec<Attribute>,
    /// Parameters — `self` appears as the first param for instance methods.
    /// The typechecker injects the correct type for `self`.
    pub params: Vec<(String, Option<Type>)>,
    pub return_ty: Option<Type>,
    pub body: Vec<Stmt>,
    /// true when the first param is NOT `self` (static method)
    pub is_static: bool,
    pub type_params: Option<Vec<String>>,
    pub span: crate::Span,
}
