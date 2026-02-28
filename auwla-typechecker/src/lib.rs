pub mod checker;
pub mod scope;

pub use checker::Typechecker;
pub use scope::Scope;

#[cfg(test)]
mod tests {
    use auwla_ast::{Expr, Stmt, Type};

    use super::*;

    #[test]
    fn test_valid_some_assignment() {
        let mut checker = Typechecker::new();
        // let val: string?string = some("hello");
        let stmt = Stmt::Let {
            name: "val".to_string(),
            ty: Some(Type::Result {
                ok_type: Box::new(Type::Basic("string".to_string())),
                err_type: Box::new(Type::Basic("string".to_string())),
            }),
            initializer: Expr::Some(Box::new(Expr::StringLit("hello".to_string()))),
        };

        assert!(checker.check_stmt(&stmt).is_ok());
    }

    #[test]
    fn test_invalid_type_assignment() {
        let mut checker = Typechecker::new();
        // let val: number = "hello";
        let stmt = Stmt::Let {
            name: "val".to_string(),
            ty: Some(Type::Basic("number".to_string())),
            initializer: Expr::StringLit("hello".to_string()),
        };

        assert!(checker.check_stmt(&stmt).is_err());
    }

    #[test]
    fn test_valid_none_assignment() {
        let mut checker = Typechecker::new();
        // let val: string?string = none("error_msg");
        let stmt = Stmt::Let {
            name: "val".to_string(),
            ty: Some(Type::Result {
                ok_type: Box::new(Type::Basic("string".to_string())),
                err_type: Box::new(Type::Basic("string".to_string())),
            }),
            initializer: Expr::None(Box::new(Expr::StringLit("error_msg".to_string()))),
        };

        assert!(checker.check_stmt(&stmt).is_ok());
    }
}
