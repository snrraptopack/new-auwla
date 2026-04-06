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
pub enum ExtensionOrigin {
    Std,
    User,
    Package,
}

impl Default for ExtensionOrigin {
    fn default() -> Self {
        ExtensionOrigin::User
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionMethod {
    pub type_params: Option<Vec<String>>,
    pub name: String,
    pub is_static: bool,
    pub params: Vec<(String, Type, bool)>,
    pub return_ty: Option<Type>,
    pub attributes: Vec<Attribute>,
    #[serde(skip, default = "default_span")]
    pub span: crate::Span,
    #[serde(default)]
    pub origin: ExtensionOrigin,
}

fn default_span() -> crate::Span {
    0..0
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
    /// let (x, y) = point; or let ((a, b), (c, d)) = nested;
    TupleDestructureLet {
        pattern: crate::expr::Pattern,
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
    /// target += 10;
    CompoundAssign { target: Expr, op: crate::expr::BinaryOp, value: Expr },
    /// fn add<T>(a: T, b: T): T { ... }
    Fn {
        name: String,
        type_params: Option<Vec<String>>,
        params: Vec<(String, Type, bool)>, // name, type, is_vararg
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
    /// for binding in iterable step 2 { body }
    For {
        bindings: Vec<String>,
        iterable: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    /// struct Name<T> { field: T, ... }
    StructDecl {
        name: String,
        type_params: Option<Vec<String>>,
        fields: Vec<(String, Type)>, // name, type
        attributes: Vec<Attribute>,
    },
    /// enum Name<T> { Variant1, Variant2(T) }
    EnumDecl {
        name: String,
        type_params: Option<Vec<String>>,
        variants: Vec<(String, Vec<Type>)>,
        attributes: Vec<Attribute>,
    },
    /// import { add, Vec2 } from './math';
    Import { names: Vec<String>, path: String },
    /// export fn / export let / export struct / export enum
    Export { stmt: Box<Stmt> },
    /// extend number? { fn val_or(...) } or extend Optional<T> { ... }
    Extend {
        /// Generic type parameters declared on the extend block, e.g. `T`, `E`
        type_params: Option<Vec<String>>,
        /// The full type being extended — can be number?, T?E, array<T>, a custom name, etc.
        target_type: Type,
        methods: Vec<Method>,
    },
    /// type Name<T> = Result<T, string>;
    TypeAlias {
        name: String,
        type_params: Option<Vec<String>>,
        aliased_type: Type,
    },
    /// type Name { fn method() { ... } }
    TypeDecl {
        name: String,
        type_params: Option<Vec<String>>,
        attributes: Vec<Attribute>,
        methods: Vec<Method>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

/// Operator types that can be overloaded
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperatorType {
    Add,              // +
    Sub,              // -
    Mul,              // *
    Div,              // /
    Mod,              // %
    Range,            // ..
    RangeExclusive,   // ..<
}

impl OperatorType {
    /// Get the method name suffix for this operator (e.g., "plus" for +)
    pub fn method_suffix(&self) -> &'static str {
        match self {
            OperatorType::Add => "plus",
            OperatorType::Sub => "minus",
            OperatorType::Mul => "mul",
            OperatorType::Div => "div",
            OperatorType::Mod => "mod",
            OperatorType::Range => "range",
            OperatorType::RangeExclusive => "range_exclusive",
        }
    }
}

/// A method defined inside an `extend` block.
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub name: String,
    pub attributes: Vec<Attribute>,
    /// Parameters — `self` appears as the first param for instance methods.
    /// The typechecker injects the correct type for `self`.
    pub params: Vec<(String, Option<Type>, bool)>,
    pub return_ty: Option<Type>,
    pub body: Vec<Stmt>,
    /// true when the first param is NOT `self` (static method)
    pub is_static: bool,
    pub type_params: Option<Vec<String>>,
    pub span: crate::Span,
    /// If this is an operator overload, specifies which operator
    pub operator: Option<OperatorType>,
}
