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
}
