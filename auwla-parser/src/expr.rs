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

            // EnumInit: EnumName::VariantName(arg1, arg2)
            let enum_init = select! { Token::Ident(name) => name }
                .then_ignore(just(Token::DoubleColon))
                .then(select! { Token::Ident(vname) => vname })
                .then(
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .or_not()
                        .map(|o| o.unwrap_or_default()),
                )
                .map(|((enum_name, variant_name), args)| Expr::EnumInit {
                    enum_name,
                    variant_name,
                    args,
                });

            // Build match expression parser conditionally
            let base_atom = bool_lit
                .clone()
                .or(some_expr)
                .or(none_expr)
                .or(interp)
                .or(enum_init)
                .or(ident_call_struct.clone())
                .or(num.clone())
                .or(str_lit.clone())
                .or(char_lit.clone())
                .or(array_lit)
                .or(expr
                    .clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)));

            let atom: Box<dyn Parser<Token, Expr, Error = Simple<Token>> + '_> = if let Some(
                ref stmt,
            ) = maybe_stmt
            {
                // parse variant name and optional bindings inside parens
                let lit_expr = choice((
                    bool_lit.clone(),
                    num.clone(),
                    str_lit.clone(),
                    char_lit.clone(),
                ));

                let range_or_lit = lit_expr
                    .then(
                        just(Token::DotDot)
                            .to(true)
                            .or(just(Token::DotDotLt).to(false))
                            .then(choice((num.clone(), char_lit.clone())))
                            .or_not(),
                    )
                    .map(|(lhs, rhs)| {
                        if let Some((inclusive, end)) = rhs {
                            auwla_ast::Pattern::Range {
                                start: Box::new(lhs),
                                end: Box::new(end),
                                inclusive,
                            }
                        } else {
                            auwla_ast::Pattern::Literal(lhs)
                        }
                    });

                let base_pattern = recursive(|pattern| {
                    choice((
                            select! { Token::Ident(n) if n == "_" => auwla_ast::Pattern::Wildcard },
                            range_or_lit.clone(),
                            // Struct pattern Parser: User { role: "admin", name } or { role: "admin" }
                            select! { Token::Ident(n) if n.chars().next().map_or(false, |c| c.is_uppercase()) => n }
                                .or_not()
                                .then(
                                    select! { Token::Ident(f) => f }
                                        .then(just(Token::Colon).ignore_then(pattern.clone()).or_not())
                                        .separated_by(just(Token::Comma))
                                        .delimited_by(just(Token::LBrace), just(Token::RBrace))
                                )
                                .map(|(name, fields)| auwla_ast::Pattern::Struct(name, fields)),
                            // Variant and Variable Pattern Parser
                            select! { Token::Ident(n) if n != "_" => n }
                                .or(just(Token::Some).to("some".to_string()))
                                .or(just(Token::None).to("none".to_string()))
                                .then(
                                    select! { Token::Ident(n) => n }
                                        .separated_by(just(Token::Comma))
                                        .delimited_by(just(Token::LParen), just(Token::RParen))
                                        .or_not(),
                                )
                                .map(|(name, opt_bindings)| {
                                    if name == "some" || name == "none" {
                                        auwla_ast::Pattern::Variant {
                                            name,
                                            bindings: opt_bindings.unwrap_or_default(),
                                        }
                                    } else if let Some(bindings) = opt_bindings {
                                        auwla_ast::Pattern::Variant { name, bindings }
                                    } else if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                                        auwla_ast::Pattern::Variant {
                                            name,
                                            bindings: vec![],
                                        }
                                    } else {
                                        auwla_ast::Pattern::Variable(name)
                                    }
                                }),
                        ))
                });

                let arm_pattern = base_pattern
                    .clone()
                    .separated_by(just(Token::Pipe))
                    .at_least(1)
                    .map(|mut patterns| {
                        if patterns.len() == 1 {
                            patterns.pop().unwrap()
                        } else {
                            auwla_ast::Pattern::Or(patterns)
                        }
                    });

                let block_arm_body = stmt
                    .clone()
                    .repeated()
                    .then(expr.clone().or_not())
                    .delimited_by(just(Token::LBrace), just(Token::RBrace));

                let arm_rhs = just(Token::FatArrow).ignore_then(
                    block_arm_body
                        .map(|(mut stmts, mut result)| {
                            if result.is_none() {
                                if let Some(auwla_ast::Stmt::Expr(auwla_ast::Expr::Match {
                                    ..
                                })) = stmts.last()
                                {
                                    if let auwla_ast::Stmt::Expr(e) = stmts.pop().unwrap() {
                                        result = Some(e);
                                    }
                                }
                            }
                            (stmts, result)
                        })
                        .or(expr.clone().map(|e| (vec![], Some(e)))),
                );

                let arm_guard = just(Token::If).ignore_then(expr.clone()).or_not();

                let arm_parser = arm_pattern.then(arm_guard).then(arm_rhs).map(
                    |((pattern, guard), (stmts, result)): (
                        _,
                        (Vec<auwla_ast::Stmt>, Option<Expr>),
                    )| {
                        MatchArm {
                            pattern,
                            guard: guard.map(Box::new),
                            stmts,
                            result: result.map(Box::new),
                        }
                    },
                );

                let match_expr = just(Token::Match)
                    .ignore_then(expr.clone())
                    .then(
                        arm_parser
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .delimited_by(just(Token::LBrace), just(Token::RBrace)),
                    )
                    .map(|(e, arms)| Expr::Match {
                        expr: Box::new(e),
                        arms,
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
