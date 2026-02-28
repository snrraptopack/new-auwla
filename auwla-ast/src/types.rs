use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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
    /// Function type: `(string, number) => bool`
    Function(Vec<Type>, Box<Type>),
    /// Optional type: `string?`, `number?`
    Optional(Box<Type>),
    /// A generic type instantiation: `Result<T, string>`
    Generic(String, Vec<Type>),
    /// A raw generic type variable: `T`
    TypeVar(String),
    /// An internal unification variable used during type inference
    InferenceVar(usize),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Basic(name) => write!(f, "{}", name),
            Type::Custom(name) => write!(f, "{}", name),
            Type::TypeVar(name) => write!(f, "{}", name),
            Type::InferenceVar(id) => write!(f, "?T{}", id),
            Type::Array(inner) => write!(f, "{}[]", inner),
            Type::Optional(inner) => write!(f, "{}?", inner),
            Type::Result { ok_type, err_type } => write!(f, "{}?{}", ok_type, err_type),
            Type::Function(params, ret) => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
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
        }
    }
}
