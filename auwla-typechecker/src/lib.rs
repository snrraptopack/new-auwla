pub mod checker;
pub mod expr;
pub mod inference;
pub mod module;
pub mod scope;
pub mod stmt;

pub use checker::Typechecker;
pub use module::{ExportMap, collect_exports};
pub use scope::Scope;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub span: auwla_ast::Span,
    pub message: String,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use auwla_ast::Type;

    use super::*;

    #[test]
    fn test_valid_some_assignment() {
        let mut checker = Typechecker::new();
        // let val: string?string = some("hello");
        let stmt = auwla_ast::Spanned::new(
            auwla_ast::StmtKind::Let {
                name: "val".to_string(),
                ty: Some(Type::Result {
                    ok_type: Box::new(Type::Basic("string".to_string())),
                    err_type: Box::new(Type::Basic("string".to_string())),
                }),
                initializer: auwla_ast::Spanned::new(
                    auwla_ast::ExprKind::Some(Box::new(auwla_ast::Spanned::new(
                        auwla_ast::ExprKind::StringLit("hello".to_string()),
                        0..0,
                    ))),
                    0..0,
                ),
            },
            0..0,
        );

        assert!(checker.check_stmt(&stmt).is_ok());
    }

    #[test]
    fn test_invalid_type_assignment() {
        let mut checker = Typechecker::new();
        // let val: number = "hello";
        let stmt = auwla_ast::Spanned::new(
            auwla_ast::StmtKind::Let {
                name: "val".to_string(),
                ty: Some(Type::Basic("number".to_string())),
                initializer: auwla_ast::Spanned::new(
                    auwla_ast::ExprKind::StringLit("hello".to_string()),
                    0..0,
                ),
            },
            0..0,
        );

        assert!(checker.check_stmt(&stmt).is_err());
    }

    #[test]
    fn test_valid_none_assignment() {
        let mut checker = Typechecker::new();
        // let val: string?string = none("error_msg");
        let stmt = auwla_ast::Spanned::new(
            auwla_ast::StmtKind::Let {
                name: "val".to_string(),
                ty: Some(Type::Result {
                    ok_type: Box::new(Type::Basic("string".to_string())),
                    err_type: Box::new(Type::Basic("string".to_string())),
                }),
                initializer: auwla_ast::Spanned::new(
                    auwla_ast::ExprKind::None(Some(Box::new(auwla_ast::Spanned::new(
                        auwla_ast::ExprKind::StringLit("error_msg".to_string()),
                        0..0,
                    )))),
                    0..0,
                ),
            },
            0..0,
        );

        assert!(checker.check_stmt(&stmt).is_ok());
    }
}
