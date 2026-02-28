use auwla_ast::{BinaryOp, Expr};
use auwla_lexer::token::Token;
use chumsky::prelude::*;

pub fn expr_parser() -> impl Parser<Token, Expr, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let ident = select! { Token::Ident(s) => Expr::Identifier(s) };
        let num = select! { Token::NumberLit(n) => Expr::NumberLit(n.parse().unwrap()) };
        let str_lit = select! { Token::StringLit(s) => Expr::StringLit(s) };

        let some_expr = just(Token::Some)
            .ignore_then(
                expr.clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .map(|inner| Expr::Some(Box::new(inner)));

        let none_expr = just(Token::None)
            .ignore_then(
                expr.clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .map(|inner| Expr::None(Box::new(inner)));

        let atom = ident
            .or(num)
            .or(str_lit)
            .or(some_expr)
            .or(none_expr)
            .or(expr.delimited_by(just(Token::LParen), just(Token::RParen)));

        // Very basic simple expression (we will add full precedence later)
        let product = atom
            .clone()
            .then(
                choice((
                    just(Token::Star).to(BinaryOp::Mul),
                    just(Token::Slash).to(BinaryOp::Div),
                ))
                .then(atom)
                .repeated(),
            )
            .map(|(lhs, rhs_list)| {
                rhs_list
                    .into_iter()
                    .fold(lhs, |acc, (op, rhs)| Expr::Binary {
                        op,
                        left: Box::new(acc),
                        right: Box::new(rhs),
                    })
            });

        let sum = product
            .clone()
            .then(
                choice((
                    just(Token::Plus).to(BinaryOp::Add),
                    just(Token::Minus).to(BinaryOp::Sub),
                ))
                .then(product)
                .repeated(),
            )
            .map(|(lhs, rhs_list)| {
                rhs_list
                    .into_iter()
                    .fold(lhs, |acc, (op, rhs)| Expr::Binary {
                        op,
                        left: Box::new(acc),
                        right: Box::new(rhs),
                    })
            });

        let cmp = sum
            .clone()
            .then(
                choice((
                    just(Token::Eq).to(BinaryOp::Eq),
                    just(Token::Neq).to(BinaryOp::Neq),
                    just(Token::Lt).to(BinaryOp::Lt),
                    just(Token::Gt).to(BinaryOp::Gt),
                    just(Token::Lte).to(BinaryOp::Lte),
                    just(Token::Gte).to(BinaryOp::Gte),
                ))
                .then(sum)
                .repeated(),
            )
            .map(|(lhs, rhs_list)| {
                rhs_list
                    .into_iter()
                    .fold(lhs, |acc, (op, rhs)| Expr::Binary {
                        op,
                        left: Box::new(acc),
                        right: Box::new(rhs),
                    })
            });

        cmp
    })
}
