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
                .map_with_span(|_, span| {
                    auwla_ast::Spanned::new(auwla_ast::ExprKind::BoolLit(true), span)
                })
                .or(just(Token::False).map_with_span(|_, span| {
                    auwla_ast::Spanned::new(auwla_ast::ExprKind::BoolLit(false), span)
                }));

            let generic_args = just(Token::DoubleColon)
                .ignore_then(
                    crate::types::type_parser()
                        .separated_by(just(Token::Comma))
                        .delimited_by(just(Token::Lt), just(Token::Gt)),
                )
                .or_not();

            let struct_init = select! { Token::Ident(name) => name }
                .then(generic_args.clone())
                .then(
                    select! { Token::Ident(field) => field }
                        .then_ignore(just(Token::Colon))
                        .then(expr.clone())
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .delimited_by(just(Token::LBrace), just(Token::RBrace)),
                )
                .map_with_span(|((name, type_args), fields), span| {
                    auwla_ast::Spanned::new(
                        auwla_ast::ExprKind::StructInit {
                            name,
                            type_args,
                            fields,
                        },
                        span,
                    )
                });

            let ident_or_call = select! { Token::Ident(s) => s }
                .then(generic_args.clone())
                .then(
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .or_not(),
                )
                .map_with_span(|((name, type_args), args), span| {
                    if let Some(args) = args {
                        auwla_ast::Spanned::new(
                            auwla_ast::ExprKind::Call {
                                name,
                                type_args,
                                args,
                            },
                            span,
                        )
                    } else {
                        auwla_ast::Spanned::new(auwla_ast::ExprKind::Identifier(name), span)
                    }
                });

            let ident_call_struct = struct_init.or(ident_or_call);

            let num = select! { Token::NumberLit(n) => n }.map_with_span(|n, span| {
                auwla_ast::Spanned::new(auwla_ast::ExprKind::NumberLit(n.parse().unwrap()), span)
            });
            let str_lit = select! { Token::StringLit(s) => s }.map_with_span(|s, span| {
                auwla_ast::Spanned::new(auwla_ast::ExprKind::StringLit(s), span)
            });
            let char_lit = select! { Token::CharLit(c) => c }.map_with_span(|c, span| {
                auwla_ast::Spanned::new(auwla_ast::ExprKind::CharLit(c), span)
            });

            // Array literal: [expr, expr, ...]
            let array_lit = expr
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .map_with_span(|inner, span| {
                    auwla_ast::Spanned::new(auwla_ast::ExprKind::Array(inner), span)
                });

            // String interpolation: InterpStart (StringFragment | expr)* InterpEnd
            let interp_part = select! { Token::StringFragment(s) => s }
                .map_with_span(|s, span| {
                    auwla_ast::Spanned::new(auwla_ast::ExprKind::StringLit(s), span)
                })
                .or(expr.clone());
            let interp = just(Token::InterpStart)
                .ignore_then(interp_part.repeated())
                .then_ignore(just(Token::InterpEnd))
                .map_with_span(|inner, span| {
                    auwla_ast::Spanned::new(auwla_ast::ExprKind::Interpolation(inner), span)
                });

            let some_expr = just(Token::Some)
                .ignore_then(
                    expr.clone()
                        .delimited_by(just(Token::LParen), just(Token::RParen)),
                )
                .map_with_span(|inner, span| {
                    auwla_ast::Spanned::new(auwla_ast::ExprKind::Some(Box::new(inner)), span)
                });

            let none_expr = just(Token::None)
                .ignore_then(
                    expr.clone()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .or_not(),
                )
                .map_with_span(|inner, span| {
                    auwla_ast::Spanned::new(auwla_ast::ExprKind::None(inner.map(Box::new)), span)
                });

            // StaticMethodCall: TypeName::<T>::method(arg1, arg2)
            let static_method_call = select! { Token::Ident(name) => name }
                .or(just(Token::Array).to("array".to_string()))
                .then(generic_args.clone())
                .then_ignore(just(Token::DoubleColon))
                .then(select! { Token::Ident(method) => method })
                .then(
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .delimited_by(just(Token::LParen), just(Token::RParen)),
                )
                .map_with_span(|(((type_name, type_args), method), args), span| {
                    auwla_ast::Spanned::new(
                        auwla_ast::ExprKind::StaticMethodCall {
                            type_name,
                            type_args,
                            method,
                            args,
                        },
                        span,
                    )
                });

            // EnumInit: EnumName::<T>::VariantName(arg1, arg2) (subset of static method call if parens present)
            let enum_init = select! { Token::Ident(name) => name }
                .then(generic_args.clone())
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
                .map_with_span(|(((enum_name, type_args), variant_name), args), span| {
                    auwla_ast::Spanned::new(
                        auwla_ast::ExprKind::EnumInit {
                            enum_name,
                            type_args,
                            variant_name,
                            args,
                        },
                        span,
                    )
                });

            let closure_params = select! { Token::Ident(name) => name }
                .then(
                    just(Token::Colon)
                        .ignore_then(crate::types::type_parser())
                        .or_not(),
                )
                .separated_by(just(Token::Comma))
                .delimited_by(just(Token::LParen), just(Token::RParen));

            let generic_params = select! { Token::Ident(name) => name }
                .separated_by(just(Token::Comma))
                .delimited_by(just(Token::Lt), just(Token::Gt))
                .or_not();

            let closure = generic_params
                .then(closure_params)
                .then(
                    just(Token::Colon)
                        .ignore_then(crate::types::type_parser())
                        .or_not(),
                )
                .then_ignore(just(Token::FatArrow))
                .then(expr.clone())
                .map_with_span(|(((type_params, params), return_ty), body), span| {
                    auwla_ast::Spanned::new(
                        auwla_ast::ExprKind::Closure {
                            type_params,
                            params,
                            return_ty,
                            body: Box::new(body),
                        },
                        span,
                    )
                });

            let block = if let Some(ref s_parser) = maybe_stmt {
                s_parser
                    .clone()
                    .repeated()
                    .then(expr.clone().or_not())
                    .delimited_by(just(Token::LBrace), just(Token::RBrace))
                    .map_with_span(|(stmts, result), span| {
                        auwla_ast::Spanned::new(
                            auwla_ast::ExprKind::Block(stmts, result.map(Box::new)),
                            span,
                        )
                    })
                    .boxed()
            } else {
                just(Token::LBrace)
                    .ignore_then(just(Token::RBrace))
                    .map_with_span(|_, span| {
                        auwla_ast::Spanned::new(auwla_ast::ExprKind::Block(vec![], None), span)
                    })
                    .boxed()
            };

            // Build match expression parser conditionally
            let base_atom = closure
                .or(bool_lit.clone())
                .or(some_expr)
                .or(none_expr)
                .or(interp)
                .or(enum_init)
                .or(static_method_call)
                .or(ident_call_struct.clone())
                .or(num.clone())
                .or(str_lit.clone())
                .or(char_lit.clone())
                .or(array_lit)
                .or(block.clone())
                .or(expr
                    .clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)));

            let atom: Box<dyn Parser<Token, Expr, Error = Simple<Token>> + '_> = if let Some(
                ref _stmt,
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
                    .map_with_span(|(lhs, rhs), span| {
                        if let Some((inclusive, end)) = rhs {
                            auwla_ast::Pattern::new(
                                auwla_ast::PatternKind::Range {
                                    start: Box::new(lhs),
                                    end: Box::new(end),
                                    inclusive,
                                },
                                span,
                            )
                        } else {
                            auwla_ast::Pattern::new(auwla_ast::PatternKind::Literal(lhs), span)
                        }
                    });

                let base_pattern = recursive(|pattern| {
                    choice((
                        select! { Token::Ident(n) if n == "_" => n }
                            .map_with_span(|_, span| auwla_ast::Pattern::new(auwla_ast::PatternKind::Wildcard, span)),
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
                                .map_with_span(|(name, fields), span| {
                                    auwla_ast::Pattern::new(auwla_ast::PatternKind::Struct(name, fields), span)
                                }),
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
                                .map_with_span(|(name, opt_bindings), span| {
                                    if name == "some" || name == "none" {
                                        auwla_ast::Pattern::new(
                                            auwla_ast::PatternKind::Variant {
                                                name,
                                                bindings: opt_bindings.unwrap_or_default(),
                                            },
                                            span,
                                        )
                                    } else if let Some(bindings) = opt_bindings {
                                        auwla_ast::Pattern::new(auwla_ast::PatternKind::Variant { name, bindings }, span)
                                    } else if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                                        auwla_ast::Pattern::new(
                                            auwla_ast::PatternKind::Variant {
                                                name,
                                                bindings: vec![],
                                            },
                                            span,
                                        )
                                    } else {
                                        auwla_ast::Pattern::new(auwla_ast::PatternKind::Variable(name), span)
                                    }
                                }),
                        ))
                });

                let arm_pattern = base_pattern
                    .clone()
                    .separated_by(just(Token::Pipe))
                    .at_least(1)
                    .map_with_span(|mut patterns, span| {
                        if patterns.len() == 1 {
                            patterns.pop().unwrap()
                        } else {
                            auwla_ast::Pattern::new(auwla_ast::PatternKind::Or(patterns), span)
                        }
                    });

                let arm_rhs = just(Token::FatArrow).ignore_then(
                    block
                        .clone()
                        .map(|e| match e.node {
                            auwla_ast::ExprKind::Block(stmts, res) => (stmts, res.map(|b| *b)),
                            _ => unreachable!(),
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
                    .map_with_span(|(e, arms), span| {
                        auwla_ast::Spanned::new(
                            auwla_ast::ExprKind::Match {
                                expr: Box::new(e),
                                arms,
                            },
                            span,
                        )
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
                .map_with_span(|(op, e), span| {
                    if let Some(op) = op {
                        auwla_ast::Spanned::new(
                            auwla_ast::ExprKind::Unary {
                                op,
                                expr: Box::new(e),
                            },
                            span,
                        )
                    } else {
                        e
                    }
                })
                .boxed();

            #[derive(Clone)]
            enum PostOp {
                Index(Expr, std::ops::Range<usize>),
                Try(Option<Expr>, std::ops::Range<usize>),
                /// Dot followed by ident — resolved to Method (with args) or Prop (no args)
                Method(
                    String,
                    Option<Vec<auwla_ast::Type>>,
                    Vec<Expr>,
                    std::ops::Range<usize>,
                ),
                Prop(String, std::ops::Range<usize>),
            }

            // Postfix: expr[index], expr?(error_expr), expr.method(args), expr.property
            let index_postfix = expr
                .clone()
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .map_with_span(|inner, span| PostOp::Index(inner, span));

            let try_postfix = just(Token::QuestionMark)
                .then(
                    expr.clone()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .or_not(),
                )
                .map_with_span(|(_, err), span| PostOp::Try(err, span));

            // Dot followed by ident then optional call args
            let dot_postfix = just(Token::Dot)
                .ignore_then(select! { Token::Ident(name) => name })
                .then(generic_args.clone())
                .then(
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .or_not(),
                )
                .map_with_span(|((name, type_args), args_opt), span| match args_opt {
                    Some(args) => PostOp::Method(name, type_args, args, span),
                    None => PostOp::Prop(name, span),
                });

            let postfix = unary
                .then(index_postfix.or(try_postfix).or(dot_postfix).repeated())
                .map(|(base, ops): (Expr, Vec<PostOp>)| {
                    ops.into_iter().fold(base, |acc, op| {
                        let start = acc.span.start;
                        match op {
                            PostOp::Index(idx, span) => auwla_ast::Spanned::new(
                                auwla_ast::ExprKind::Index {
                                    expr: Box::new(acc),
                                    index: Box::new(idx),
                                },
                                start..span.end,
                            ),
                            PostOp::Try(err, span) => auwla_ast::Spanned::new(
                                auwla_ast::ExprKind::Try {
                                    expr: Box::new(acc),
                                    error_expr: err.map(Box::new),
                                },
                                start..span.end,
                            ),
                            PostOp::Method(method, type_args, args, span) => {
                                auwla_ast::Spanned::new(
                                    auwla_ast::ExprKind::MethodCall {
                                        expr: Box::new(acc),
                                        method,
                                        type_args,
                                        args,
                                    },
                                    start..span.end,
                                )
                            }
                            PostOp::Prop(prop, span) => auwla_ast::Spanned::new(
                                auwla_ast::ExprKind::PropertyAccess {
                                    expr: Box::new(acc),
                                    property: prop,
                                },
                                start..span.end,
                            ),
                        }
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
                    rhs_list.into_iter().fold(lhs, |acc, (op, rhs)| {
                        let start = acc.span.start;
                        let end = rhs.span.end;
                        auwla_ast::Spanned::new(
                            auwla_ast::ExprKind::Binary {
                                op,
                                left: Box::new(acc),
                                right: Box::new(rhs),
                            },
                            start..end,
                        )
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
                    rhs_list.into_iter().fold(lhs, |acc, (op, rhs)| {
                        let start = acc.span.start;
                        let end = rhs.span.end;
                        auwla_ast::Spanned::new(
                            auwla_ast::ExprKind::Binary {
                                op,
                                left: Box::new(acc),
                                right: Box::new(rhs),
                            },
                            start..end,
                        )
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
                    rhs_list.into_iter().fold(lhs, |acc, (op, rhs)| {
                        let start = acc.span.start;
                        let end = rhs.span.end;
                        auwla_ast::Spanned::new(
                            auwla_ast::ExprKind::Binary {
                                op,
                                left: Box::new(acc),
                                right: Box::new(rhs),
                            },
                            start..end,
                        )
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
                        let start = lhs.span.start;
                        let end_span = end.span.end;
                        auwla_ast::Spanned::new(
                            auwla_ast::ExprKind::Range {
                                start: Box::new(lhs),
                                end: Box::new(end),
                                inclusive,
                            },
                            start..end_span,
                        )
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
                    rhs_list.into_iter().fold(lhs, |acc, (op, rhs)| {
                        let start = acc.span.start;
                        let end = rhs.span.end;
                        auwla_ast::Spanned::new(
                            auwla_ast::ExprKind::Binary {
                                op,
                                left: Box::new(acc),
                                right: Box::new(rhs),
                            },
                            start..end,
                        )
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
