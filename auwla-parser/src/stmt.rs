use crate::expr::expr_parser_with_stmt;
use crate::types::type_parser;
use auwla_ast::Stmt;
use auwla_lexer::token::Token;
use chumsky::prelude::*;

pub fn stmt_parser() -> impl Parser<Token, Stmt, Error = Simple<Token>> + Clone {
    let ty = type_parser();

    recursive(move |stmt| {
        // Build expression parser WITH stmt support (for match arms)
        let expr = expr_parser_with_stmt(stmt.clone());

        let let_stmt = just(Token::Let)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|((name, ty), initializer)| Stmt::Let {
                name,
                ty,
                initializer,
            });

        let var_stmt = just(Token::Var)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|((name, ty), initializer)| Stmt::Var {
                name,
                ty,
                initializer,
            });

        let return_stmt = just(Token::Return)
            .ignore_then(expr.clone().or_not())
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Return);

        let param = select! { Token::Ident(name) => name }
            .then_ignore(just(Token::Colon))
            .then(ty.clone());

        let fn_decl = just(Token::Fn)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(
                param
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
            .then(
                stmt.clone()
                    .repeated()
                    .then(expr.clone().or_not())
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|(((name, params), return_ty), (mut body, trailing_expr))| {
                // Desugar trailing expression into Stmt::Return
                if let Some(e) = trailing_expr {
                    body.push(Stmt::Return(Some(e)));
                } else if let Some(Stmt::Expr(auwla_ast::Expr::Match { .. })) = body.last() {
                    if let Stmt::Expr(e) = body.pop().unwrap() {
                        body.push(Stmt::Return(Some(e)));
                    }
                }
                Stmt::Fn {
                    name,
                    params,
                    return_ty,
                    body,
                }
            });

        let if_stmt = just(Token::If)
            .ignore_then(expr.clone())
            .then(
                stmt.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .then(
                just(Token::Else)
                    .ignore_then(
                        stmt.clone()
                            .repeated()
                            .delimited_by(just(Token::LBrace), just(Token::RBrace)),
                    )
                    .or_not(),
            )
            .map(|((condition, then_branch), else_branch)| Stmt::If {
                condition,
                then_branch,
                else_branch,
            });

        let while_stmt = just(Token::While)
            .ignore_then(expr.clone())
            .then(
                stmt.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|(condition, body)| Stmt::While { condition, body });

        let for_stmt = just(Token::For)
            .ignore_then(select! { Token::Ident(name) => name })
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .then(
                stmt.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|((binding, iterable), body)| Stmt::For {
                binding,
                iterable,
                body,
            });

        // Assignment: `target = value;`
        let assign_stmt = expr
            .clone()
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(target, value)| Stmt::Assign { target, value });

        let struct_decl = just(Token::Struct)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(
                select! { Token::Ident(name) => name }
                    .then_ignore(just(Token::Colon))
                    .then(ty.clone())
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|(name, fields)| Stmt::StructDecl { name, fields });

        // enum Name { Variant1, Variant2(type) }
        let enum_decl = just(Token::Enum)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(
                select! { Token::Ident(variant_name) => variant_name }
                    .then(
                        ty.clone()
                            .separated_by(just(Token::Comma))
                            .delimited_by(just(Token::LParen), just(Token::RParen))
                            .or_not()
                            .map(|t| t.unwrap_or_default()),
                    )
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|(name, variants)| Stmt::EnumDecl { name, variants });

        // Expression as statement — with semicolon for most expressions,
        // but match expressions don't need a trailing semicolon
        let match_stmt = expr.clone().try_map(|e, span| match &e {
            auwla_ast::Expr::Match { .. } => Ok(Stmt::Expr(e)),
            _ => Err(Simple::custom(span, "expected match expression")),
        });

        let expr_stmt = expr
            .clone()
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Expr);

        let_stmt
            .or(var_stmt)
            .or(return_stmt)
            .or(fn_decl)
            .or(if_stmt)
            .or(while_stmt)
            .or(for_stmt)
            .or(assign_stmt)
            .or(struct_decl)
            .or(enum_decl)
            .or(match_stmt)
            .or(expr_stmt)
    })
}
