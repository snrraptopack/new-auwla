pub mod expr;
pub mod stmt;
pub mod types;

use auwla_ast::Program;
use auwla_lexer::token::Token;
use chumsky::prelude::*;
use stmt::stmt_parser;

pub fn parser() -> impl Parser<Token, Program, Error = Simple<Token>> {
    stmt_parser()
        .repeated()
        .map(|statements| Program { statements })
        .then_ignore(end())
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, Vec<Simple<Token>>> {
    parser().parse(tokens)
}

#[cfg(test)]
mod tests {
    use auwla_ast::{BinaryOp, Expr, Stmt, Type};
    use auwla_lexer::lex;
    use auwla_lexer::token::Token;

    use crate::parse;

    #[test]
    fn test_parse_let() {
        let source = "let x = 5;";
        let tokens: Vec<Token> = lex(source).into_iter().map(|(t, _)| t).collect();
        let ast = parse(tokens).expect("Failed to parse");

        assert_eq!(ast.statements.len(), 1);
        if let Stmt::Let {
            name,
            ty,
            initializer,
        } = &ast.statements[0]
        {
            assert_eq!(name, "x");
            assert_eq!(ty, &None);
            assert_eq!(initializer, &Expr::NumberLit(5.0));
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_let_with_type() {
        let source = "let msg: string?string = some(\"error\");";
        let tokens: Vec<Token> = lex(source).into_iter().map(|(t, _)| t).collect();
        let ast = parse(tokens).expect("Failed to parse");

        assert_eq!(ast.statements.len(), 1);
        if let Stmt::Let {
            name,
            ty,
            initializer,
        } = &ast.statements[0]
        {
            assert_eq!(name, "msg");
            if let Some(Type::Result { ok_type, err_type }) = ty {
                assert_eq!(&**ok_type, &Type::Basic("string".to_string()));
                assert_eq!(&**err_type, &Type::Basic("string".to_string()));
            } else {
                panic!("Expected Result type");
            }
            assert_eq!(
                initializer,
                &Expr::Some(Box::new(Expr::StringLit("error".to_string())))
            );
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_var_math() {
        let source = "var y = 10 + 20;";
        let tokens: Vec<Token> = lex(source).into_iter().map(|(t, _)| t).collect();
        let ast = parse(tokens).expect("Failed to parse");

        assert_eq!(ast.statements.len(), 1);
        if let Stmt::Var {
            name,
            ty,
            initializer,
        } = &ast.statements[0]
        {
            assert_eq!(name, "y");
            assert_eq!(ty, &None);
            if let Expr::Binary { op, left, right } = initializer {
                assert_eq!(op, &BinaryOp::Add);
                assert_eq!(&**left, &Expr::NumberLit(10.0));
                assert_eq!(&**right, &Expr::NumberLit(20.0));
            } else {
                panic!("Expected Binary Expr");
            }
        } else {
            panic!("Expected Var statement");
        }
    }

    #[test]
    fn test_parse_fn() {
        let source = r#"
            fn add(a: number, b: number): number {
                return a + b;
            }
        "#;
        let tokens: Vec<Token> = lex(source).into_iter().map(|(t, _)| t).collect();
        let ast = parse(tokens).expect("Failed to parse");

        assert_eq!(ast.statements.len(), 1);
        if let Stmt::Fn {
            name,
            params,
            return_ty,
            body,
            ..
        } = &ast.statements[0]
        {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].0, "a");
            assert_eq!(params[0].1, Type::Basic("number".to_string()));

            assert_eq!(return_ty, &Some(Type::Basic("number".to_string())));
            assert_eq!(body.len(), 1);
            if let Stmt::Return(Some(Expr::Binary { op, .. })) = &body[0] {
                assert_eq!(op, &BinaryOp::Add);
            } else {
                panic!("Expected Return binary expression");
            }
        } else {
            panic!("Expected Fn statement");
        }
    }

    #[test]
    fn test_parse_if_else() {
        let source = r#"
            if count > 10 {
                return some("Too many");
            } else {
                return none("Okay");
            }
        "#;
        let tokens: Vec<Token> = lex(source).into_iter().map(|(t, _)| t).collect();
        let ast = parse(tokens).expect("Failed to parse");

        assert_eq!(ast.statements.len(), 1);
        if let Stmt::If {
            condition,
            then_branch,
            else_branch,
        } = &ast.statements[0]
        {
            if let Expr::Binary { op, .. } = condition {
                assert_eq!(op, &BinaryOp::Gt);
            } else {
                panic!("Expected > condition");
            }

            assert_eq!(then_branch.len(), 1);
            if let Stmt::Return(Some(Expr::Some(_))) = &then_branch[0] {
                // Return Some
            } else {
                panic!("Expected return some(...)");
            }

            let els = else_branch.as_ref().expect("Expected else branch");
            assert_eq!(els.len(), 1);
            if let Stmt::Return(Some(Expr::None(_))) = &els[0] {
                // Return None
            } else {
                panic!("Expected return none(...)");
            }
        } else {
            panic!("Expected If statement");
        }
    }
}
