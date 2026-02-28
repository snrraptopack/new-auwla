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

            let struct_init = select! { Token::Ident(name) => name }
                .then(
                    select! { Token::Ident(field) => field }
                        .then_ignore(just(Token::Colon))
                        .then(expr.clone())
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .delimited_by(just(Token::LBrace), just(Token::RBrace)),
                )
                .map(|(name, fields)| Expr::StructInit { name, fields });

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

            let ident_call_struct = struct_init.or(ident_or_call);

            let num = select! { Token::NumberLit(n) => Expr::NumberLit(n.parse().unwrap()) };
            let str_lit = select! { Token::StringLit(s) => Expr::StringLit(s) };
            let char_lit = select! { Token::CharLit(c) => Expr::CharLit(c) };

            // Array literal: [expr, expr, ...]
            let array_lit = expr
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .map(Expr::Array);

            // String interpolation: InterpStart (StringFragment | expr)* InterpEnd
            let interp_part =
                select! { Token::StringFragment(s) => Expr::StringLit(s) }.or(expr.clone());
            let interp = just(Token::InterpStart)
                .ignore_then(interp_part.repeated())
                .then_ignore(just(Token::InterpEnd))
                .map(Expr::Interpolation);

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
                .or(interp)
                .or(ident_call_struct.clone())
                .or(num)
                .or(str_lit)
                .or(char_lit)
                .or(array_lit)
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

            #[derive(Clone)]
            enum PostOp {
                Index(Expr),
                Try(Option<Expr>),
                Prop(String),
            }

            // Postfix: expr[index], expr?(error_expr), and expr.property
            let index_postfix = expr
                .clone()
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .map(PostOp::Index);

            let try_postfix = just(Token::QuestionMark)
                .then(
                    expr.clone()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .or_not(),
                )
                .map(|(_, err)| PostOp::Try(err));

            let prop_postfix = just(Token::Dot)
                .ignore_then(select! { Token::Ident(prop) => prop })
                .map(PostOp::Prop);

            let postfix = unary
                .then(index_postfix.or(try_postfix).or(prop_postfix).repeated())
                .map(|(base, ops): (Expr, Vec<PostOp>)| {
                    ops.into_iter().fold(base, |acc, op| match op {
                        PostOp::Index(idx) => Expr::Index {
                            expr: Box::new(acc),
                            index: Box::new(idx),
                        },
                        PostOp::Try(err) => Expr::Try {
                            expr: Box::new(acc),
                            error_expr: err.map(Box::new),
                        },
                        PostOp::Prop(prop) => Expr::PropertyAccess {
                            expr: Box::new(acc),
                            property: prop,
                        },
                    })
                })
                .boxed();

            // product: * /
            let product = postfix
                .clone()
                .then(
                    choice((
                        just(Token::Star).to(BinaryOp::Mul),
                        just(Token::Slash).to(BinaryOp::Div),
                    ))
                    .then(postfix)
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

            // range: expr..expr (inclusive) or expr..<expr (exclusive)
            let range = cmp
                .clone()
                .then(
                    just(Token::DotDot)
                        .to(true) // inclusive
                        .or(just(Token::DotDotLt).to(false)) // exclusive
                        .then(cmp.clone())
                        .or_not(),
                )
                .map(|(lhs, rhs)| {
                    if let Some((inclusive, end)) = rhs {
                        Expr::Range {
                            start: Box::new(lhs),
                            end: Box::new(end),
                            inclusive,
                        }
                    } else {
                        lhs
                    }
                });

            // logical: && ||
            let logical = range
                .clone()
                .then(
                    choice((
                        just(Token::And).to(BinaryOp::And),
                        just(Token::Or).to(BinaryOp::Or),
                    ))
                    .then(range)
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
