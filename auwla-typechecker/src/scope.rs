use auwla_ast::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutability {
    /// Declared with `let` — cannot be reassigned.
    Immutable,
    /// Declared with `var` — can be reassigned.
    Mutable,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub variables: HashMap<String, Type>,
    pub mutability: HashMap<String, Mutability>,
    pub functions: HashMap<String, (Vec<Type>, Option<Type>)>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            mutability: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}
