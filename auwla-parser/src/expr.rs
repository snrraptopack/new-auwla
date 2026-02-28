use auwla_ast::{BinaryOp, Expr, MatchArm, Stmt, UnaryOp};
use auwla_lexer::token::Token;
use chumsky::prelude::*;

/// Build simple expression parser (no match arm support).
/// Used by tests and simple contexts.
pub fn expr_parser() -> impl Parser<Token, Expr, Error = Simple<Token>> + Clone {
    expr_parser_inner(None)
}

/// Internal: builds expression parser. If a boxed stmt parser is provided,
/// match-expression with block arms is supported.
fn expr_parser_inner(
    maybe_stmt: Option<BoxedParser<'static, Token, Stmt, Simple<Token>>>,
) -> impl Parser<Token, Expr, Error = Simple<Token>> + Clone {
    recursive(
        move |expr: chumsky::recursive::Recursive<'_, Token, Expr, Simple<Token>>| {
            let bool_lit = just(Token::True)
                .to(Expr::BoolLit(true))
                .or(just(Token::False).to(Expr::BoolLit(false)));

            let ident_or_call = select! { Token::Ident(s) => s }
                .then(
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .or_not(),
                )
                .map(|(name, args)| {
                    if let Some(args) = args {
                        Expr::Call { name, args }
                    } else {
                        Expr::Identifier(name)
                    }
                });

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

            // Build match expression parser conditionally
            let base_atom = bool_lit
                .or(some_expr)
                .or(none_expr)
                .or(ident_or_call)
                .or(num)
                .or(str_lit)
                .or(expr
                    .clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)));

            let atom: Box<dyn Parser<Token, Expr, Error = Simple<Token>> + '_> =
                if let Some(ref stmt) = maybe_stmt {
                    let arm_binding = |kw: Token| {
                        just(kw).ignore_then(
                            select! { Token::Ident(n) => n }
                                .delimited_by(just(Token::LParen), just(Token::RParen)),
                        )
                    };

                    let block_arm_body = stmt
                        .clone()
                        .repeated()
                        .then(expr.clone().or_not())
                        .delimited_by(just(Token::LBrace), just(Token::RBrace));

                    let arm_rhs = just(Token::FatArrow).ignore_then(
                        block_arm_body
                            .map(|(stmts, result)| (stmts, result))
                            .or(expr.clone().map(|e| (vec![], Some(e)))),
                    );

                    let some_arm_parser = arm_binding(Token::Some).then(arm_rhs.clone()).map(
                        |(binding, (stmts, result))| MatchArm {
                            binding,
                            stmts,
                            result: result.map(Box::new),
                        },
                    );

                    let none_arm_parser =
                        arm_binding(Token::None)
                            .then(arm_rhs)
                            .map(|(binding, (stmts, result))| MatchArm {
                                binding,
                                stmts,
                                result: result.map(Box::new),
                            });

                    let match_expr = just(Token::Match)
                        .ignore_then(expr.clone())
                        .then(
                            just(Token::LBrace)
                                .ignore_then(
                                    some_arm_parser.clone().then(none_arm_parser.clone()).or(
                                        none_arm_parser.then(some_arm_parser).map(|(n, s)| (s, n)),
                                    ),
                                )
                                .then_ignore(just(Token::RBrace)),
                        )
                        .map(|(e, (some_arm, none_arm))| Expr::Match {
                            expr: Box::new(e),
                            some_arm,
                            none_arm,
                        });

                    Box::new(match_expr.or(base_atom))
                } else {
                    Box::new(base_atom)
                };

            // Unary: !expr or -expr
            let unary = just(Token::Not)
                .to(UnaryOp::Not)
                .or(just(Token::Minus).to(UnaryOp::Neg))
                .or_not()
                .then(atom)
                .map(|(op, e)| {
                    if let Some(op) = op {
                        Expr::Unary {
                            op,
                            expr: Box::new(e),
                        }
                    } else {
                        e
                    }
                })
                .boxed();

            // product: * /
            let product = unary
                .clone()
                .then(
                    choice((
                        just(Token::Star).to(BinaryOp::Mul),
                        just(Token::Slash).to(BinaryOp::Div),
                    ))
                    .then(unary)
                    .repeated(),
                )
                .map(|(lhs, rhs_list): (Expr, Vec<(BinaryOp, Expr)>)| {
                    rhs_list
                        .into_iter()
                        .fold(lhs, |acc, (op, rhs)| Expr::Binary {
                            op,
                            left: Box::new(acc),
                            right: Box::new(rhs),
                        })
                });

            // sum: + -
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
                .map(|(lhs, rhs_list): (Expr, Vec<(BinaryOp, Expr)>)| {
                    rhs_list
                        .into_iter()
                        .fold(lhs, |acc, (op, rhs)| Expr::Binary {
                            op,
                            left: Box::new(acc),
                            right: Box::new(rhs),
                        })
                });

            // cmp: == != < > <= >=
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
                .map(|(lhs, rhs_list): (Expr, Vec<(BinaryOp, Expr)>)| {
                    rhs_list
                        .into_iter()
                        .fold(lhs, |acc, (op, rhs)| Expr::Binary {
                            op,
                            left: Box::new(acc),
                            right: Box::new(rhs),
                        })
                });

            // logical: && ||
            let logical = cmp
                .clone()
                .then(
                    choice((
                        just(Token::And).to(BinaryOp::And),
                        just(Token::Or).to(BinaryOp::Or),
                    ))
                    .then(cmp)
                    .repeated(),
                )
                .map(|(lhs, rhs_list): (Expr, Vec<(BinaryOp, Expr)>)| {
                    rhs_list
                        .into_iter()
                        .fold(lhs, |acc, (op, rhs)| Expr::Binary {
                            op,
                            left: Box::new(acc),
                            right: Box::new(rhs),
                        })
                });

            logical
        },
    )
}

type BoxedParser<'a, I, O, E> = chumsky::BoxedParser<'a, I, O, E>;

/// Build expression parser with stmt support for match arms.
pub fn expr_parser_with_stmt(
    stmt: impl Parser<Token, Stmt, Error = Simple<Token>> + Clone + 'static,
) -> impl Parser<Token, Expr, Error = Simple<Token>> + Clone {
    expr_parser_inner(Some(stmt.boxed()))
}
