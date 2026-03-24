use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// A basic type like `string`, `number`, `bool`
    Basic(String),
    /// The custom error type pattern: `expected_type?error_type`
    /// Example: `string?string` means expect `Basic("string")` or error `Basic("string")`
    Result {
        ok_type: Box<Type>,
        err_type: Box<Type>,
    },
    /// Homogeneous array type: `number[]`, `string[]`
    Array(Box<Type>),
    /// User-defined struct type: `User`, `Point`
    Custom(String),
    /// Dictionary mapping: `dict<K, V>`
    Dict(Box<Type>, Box<Type>),
    /// Function type: `(string, ...number) => bool`
    Function(Vec<(Type, bool)>, Box<Type>),
    /// Optional type: `string?`, `number?`
    Optional(Box<Type>),
    /// A generic type instantiation: `Result<T, string>`
    Generic(String, Vec<Type>),
    /// A raw generic type variable: `T`
    TypeVar(String),
    /// An internal unification variable used during type inference
    InferenceVar(usize),
    /// The `Self` type — resolves to the enclosing type name during typechecking
    SelfType,
    /// Internal: A polymorphic wrapper (either Optional or Result) specifically for some()
    Wrapper(Box<Type>),
    /// Tuple type: (number, string, bool)
    Tuple(Vec<Type>),
}

impl Type {
    /// Returns the core type constructor name for grouping extensions.
    /// This prevents string-matching hacks like "array<T>" vs "array<number>".
    pub fn base_key(&self) -> String {
        match self {
            Type::Basic(name) | Type::Custom(name) | Type::TypeVar(name) | Type::Generic(name, _) => name.clone(),
            Type::Array(_) => "array".to_string(),
            Type::Dict(_, _) => "dict".to_string(),
            Type::Optional(_) => "optional".to_string(),
            Type::Result { .. } => "result".to_string(),
            Type::Function(_, _) => "fn".to_string(),
            Type::InferenceVar(id) => format!("_{}", id),
            Type::SelfType => "Self".to_string(),
            Type::Wrapper(inner) => format!("wrapper<{}>", inner.base_key()),
            Type::Tuple(_) => "tuple".to_string(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Basic(name) => write!(f, "{}", name),
            Type::Custom(name) => write!(f, "{}", name),
            Type::TypeVar(name) => write!(f, "{}", name),
            Type::InferenceVar(id) => write!(f, "?T{}", id),
            Type::Array(inner) => write!(f, "{}[]", inner),
            Type::Dict(k, v) => write!(f, "dict<{}, {}>", k, v),
            Type::Optional(inner) => write!(f, "{}?", inner),
            Type::Result { ok_type, err_type } => write!(f, "{}?{}", ok_type, err_type),
            Type::Function(params, ret) => {
                write!(f, "(")?;
                for (i, (p, is_vararg)) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if *is_vararg {
                        write!(f, "...")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") => {}", ret)
            }
            Type::Generic(name, args) => {
                write!(f, "{}<", name)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ">")
            }
            Type::SelfType => write!(f, "Self"),
            Type::Wrapper(inner) => write!(f, "wrapper<{}>", inner),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
        }
    }
}
