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
    use auwla_ast::{BinaryOp, Type};
    use auwla_lexer::lex;
    use auwla_lexer::token::Token;

    use crate::parse;

    #[test]
    fn test_parse_let() {
        let source = "let x = 5;";
        let tokens: Vec<Token> = lex(source).into_iter().map(|(t, _)| t).collect();
        let ast = parse(tokens).expect("Failed to parse");

        assert_eq!(ast.statements.len(), 1);
        if let auwla_ast::StmtKind::Let {
            name,
            ty,
            initializer,
        } = &ast.statements[0].node
        {
            assert_eq!(name, "x");
            assert_eq!(ty, &None);
            if let auwla_ast::ExprKind::NumberLit(val) = initializer.node {
                assert_eq!(val, 5.0);
            } else {
                panic!("Expected NumberLit");
            }
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
        if let auwla_ast::StmtKind::Let {
            name,
            ty,
            initializer,
        } = &ast.statements[0].node
        {
            assert_eq!(name, "msg");
            if let Some(Type::Result { ok_type, err_type }) = ty {
                assert_eq!(&**ok_type, &Type::Basic("string".to_string()));
                assert_eq!(&**err_type, &Type::Basic("string".to_string()));
            } else {
                panic!("Expected Result type");
            }
            if let auwla_ast::ExprKind::Some(inner) = &initializer.node {
                if let auwla_ast::ExprKind::StringLit(s) = &inner.node {
                    assert_eq!(s, "error");
                } else {
                    panic!("Expected StringLit inner");
                }
            } else {
                panic!("Expected Some expr");
            }
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
        if let auwla_ast::StmtKind::Var {
            name,
            ty,
            initializer,
        } = &ast.statements[0].node
        {
            assert_eq!(name, "y");
            assert_eq!(ty, &None);
            if let auwla_ast::ExprKind::Binary { op, left, right } = &initializer.node {
                assert_eq!(op, &BinaryOp::Add);
                if let auwla_ast::ExprKind::NumberLit(l) = left.node {
                    assert_eq!(l, 10.0);
                }
                if let auwla_ast::ExprKind::NumberLit(r) = right.node {
                    assert_eq!(r, 20.0);
                }
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
        if let auwla_ast::StmtKind::Fn {
            name,
            params,
            return_ty,
            body,
            ..
        } = &ast.statements[0].node
        {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].0, "a");
            assert_eq!(params[0].1, Type::Basic("number".to_string()));

            assert_eq!(return_ty, &Some(Type::Basic("number".to_string())));
            assert_eq!(body.len(), 1);
            if let auwla_ast::StmtKind::Return(Some(e)) = &body[0].node {
                if let auwla_ast::ExprKind::Binary { op, .. } = &e.node {
                    assert_eq!(op, &BinaryOp::Add);
                } else {
                    panic!("Expected Return binary expression");
                }
            } else {
                panic!("Expected Return statement");
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
        if let auwla_ast::StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } = &ast.statements[0].node
        {
            if let auwla_ast::ExprKind::Binary { op, .. } = &condition.node {
                assert_eq!(op, &BinaryOp::Gt);
            } else {
                panic!("Expected > condition");
            }

            assert_eq!(then_branch.len(), 1);
            if let auwla_ast::StmtKind::Return(Some(e)) = &then_branch[0].node {
                if let auwla_ast::ExprKind::Some(_) = e.node {
                    // Return Some
                } else {
                    panic!("Expected return some(...)");
                }
            } else {
                panic!("Expected Return statement");
            }

            let els = else_branch.as_ref().expect("Expected else branch");
            assert_eq!(els.len(), 1);
            if let auwla_ast::StmtKind::Return(Some(e)) = &els[0].node {
                if let auwla_ast::ExprKind::None(_) = e.node {
                    // Return None
                } else {
                    panic!("Expected return none(...)");
                }
            } else {
                panic!("Expected Return statement");
            }
        } else {
            panic!("Expected If statement");
        }
    }
}
