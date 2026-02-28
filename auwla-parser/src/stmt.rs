use crate::expr::expr_parser;
use crate::types::type_parser;
use auwla_ast::Stmt;
use auwla_lexer::token::Token;
use chumsky::prelude::*;

pub fn stmt_parser() -> impl Parser<Token, Stmt, Error = Simple<Token>> + Clone {
    let expr = expr_parser();
    let ty = type_parser();

    recursive(move |stmt| {
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
            .map(|expr| Stmt::Return(expr));

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
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|(((name, params), return_ty), body)| Stmt::Fn {
                name,
                params,
                return_ty,
                body,
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

        // Assignment: `name = value;`  (must come before expr_stmt since both start with ident)
        let assign_stmt = select! { Token::Ident(name) => name }
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(name, value)| Stmt::Assign { name, value });

        let expr_stmt = expr
            .clone()
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Expr);

        let_stmt
            .or(var_stmt)
            .or(return_stmt)
            .or(fn_decl)
            .or(if_stmt)
            .or(assign_stmt) // before expr_stmt so `a = x;` isn't consumed as an expression
            .or(expr_stmt)
    })
}
