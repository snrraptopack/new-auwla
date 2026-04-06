use crate::expr::expr_parser_with_stmt;
use crate::types::type_parser;
use auwla_ast::{Stmt, StmtKind};
use auwla_lexer::token::Token;
use chumsky::prelude::*;

pub fn stmt_parser() -> impl Parser<Token, Stmt, Error = Simple<Token>> + Clone {
    let ty = type_parser();

    recursive(move |stmt| {
        // Build expression parser WITH stmt support (for match arms)
        let expr = expr_parser_with_stmt(stmt.clone());

        let attribute = just(Token::At)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(
                select! { Token::StringLit(s) => s }
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .or_not()
                    .map(|args| args.unwrap_or_default()),
            )
            .map(|(name, args)| auwla_ast::Attribute { name, args });

        let attributes = attribute.repeated();

        let let_stmt = just(Token::Let)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|((name, ty), initializer), span| {
                auwla_ast::Spanned::new(
                    StmtKind::Let {
                        name,
                        ty,
                        initializer,
                    },
                    span,
                )
            });

        let destructure_stmt = just(Token::Let)
            .ignore_then(
                select! { Token::Ident(name) => name }
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|(bindings, initializer), span| {
                auwla_ast::Spanned::new(
                    StmtKind::DestructureLet {
                        bindings,
                        initializer,
                    },
                    span,
                )
            });

        let tuple_destructure_stmt = just(Token::Let)
            .ignore_then(
                // Use a simple recursive pattern parser for tuple destructuring
                recursive(|pattern| {
                    choice((
                        // Tuple pattern: (x, y) or ((a, b), c)
                        pattern
                            .clone()
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .delimited_by(just(Token::LParen), just(Token::RParen))
                            .map_with_span(|patterns, span| {
                                if patterns.len() == 1 {
                                    patterns.into_iter().next().unwrap()
                                } else {
                                    auwla_ast::Spanned::new(auwla_ast::PatternKind::Tuple(patterns), span)
                                }
                            }),
                        // Variable pattern: x, y, z
                        select! { Token::Ident(name) => name }
                            .map_with_span(|name, span| {
                                auwla_ast::Spanned::new(auwla_ast::PatternKind::Variable(name), span)
                            }),
                    ))
                })
            )
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|(pattern, initializer), span| {
                auwla_ast::Spanned::new(
                    StmtKind::TupleDestructureLet {
                        pattern,
                        initializer,
                    },
                    span,
                )
            });

        let var_stmt = just(Token::Var)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|((name, ty), initializer), span| {
                auwla_ast::Spanned::new(
                    StmtKind::Var {
                        name,
                        ty,
                        initializer,
                    },
                    span,
                )
            });

        let return_stmt = just(Token::Return)
            .ignore_then(expr.clone().or_not())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|inner, span| auwla_ast::Spanned::new(StmtKind::Return(inner), span));

        let param = just(Token::Ellipsis)
            .or_not()
            .then(select! { Token::Ident(name) => name })
            .then_ignore(just(Token::Colon))
            .then(ty.clone())
            .map(|((ellipsis, name), ty)| (name, ty, ellipsis.is_some()));

        let generic_params = select! { Token::Ident(name) => name }
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::Lt), just(Token::Gt))
            .or_not();

        let fn_decl = attributes
            .clone()
            .then_ignore(just(Token::Fn))
            .then(select! { Token::Ident(name) => name })
            .then(generic_params.clone())
            .then(
                param
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
            .then(
                // Block body: { stmts... [trailing_expr] }
                stmt.clone()
                    .repeated()
                    .then(expr.clone().or_not())
                    .delimited_by(just(Token::LBrace), just(Token::RBrace))
                    .map_with_span(|(mut body, trailing_expr), _span| {
                        if let Some(e) = trailing_expr {
                            let e_span = e.span.clone();
                            body.push(auwla_ast::Spanned::new(StmtKind::Return(Some(e)), e_span));
                        }
                        body
                    })
                    // Expression body: => expr;
                    .or(just(Token::FatArrow)
                        .ignore_then(expr.clone())
                        .then_ignore(just(Token::Semicolon))
                        .map_with_span(|e, span| {
                            vec![auwla_ast::Spanned::new(StmtKind::Return(Some(e)), span)]
                        })),
            )
            .map_with_span(
                |(((((attributes, name), type_params), params), return_ty), body), span| {
                    auwla_ast::Spanned::new(
                        StmtKind::Fn {
                            name,
                            type_params,
                            params,
                            return_ty,
                            body,
                            attributes,
                        },
                        span,
                    )
                },
            );

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
            .map_with_span(|((condition, then_branch), else_branch), span| {
                auwla_ast::Spanned::new(
                    auwla_ast::StmtKind::If {
                        condition,
                        then_branch,
                        else_branch,
                    },
                    span,
                )
            });

        let while_stmt = just(Token::While)
            .ignore_then(expr.clone())
            .then(
                stmt.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|(condition, body), span| {
                auwla_ast::Spanned::new(auwla_ast::StmtKind::While { condition, body }, span)
            });

        let for_binding = select! { Token::Ident(name) => vec![name] }
            .or(select! { Token::Ident(name) => name }
                .separated_by(just(Token::Comma))
                .delimited_by(just(Token::LParen), just(Token::RParen)));

        let for_stmt = just(Token::For)
            .ignore_then(for_binding)
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .then(just(Token::Step).ignore_then(expr.clone()).or_not())
            .then(
                stmt.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|(((bindings, iterable), step), body), span| {
                auwla_ast::Spanned::new(
                    auwla_ast::StmtKind::For {
                        bindings,
                        iterable,
                        step,
                        body,
                    },
                    span,
                )
            });

        // Assignment: `target = value;` or `target += value;`
        let assign_stmt = expr
            .clone()
            .then(
                just(Token::Assign)
                    .to(None)
                    .or(just(Token::PlusEq).to(Some(auwla_ast::BinaryOp::Add)))
                    .or(just(Token::MinusEq).to(Some(auwla_ast::BinaryOp::Sub)))
                    .or(just(Token::StarEq).to(Some(auwla_ast::BinaryOp::Mul)))
                    .or(just(Token::SlashEq).to(Some(auwla_ast::BinaryOp::Div)))
                    .or(just(Token::PercentEq).to(Some(auwla_ast::BinaryOp::Mod))),
            )
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|((target, op_opt), value), span| {
                if let Some(op) = op_opt {
                    auwla_ast::Spanned::new(auwla_ast::StmtKind::CompoundAssign { target, op, value }, span)
                } else {
                    auwla_ast::Spanned::new(auwla_ast::StmtKind::Assign { target, value }, span)
                }
            });

        let struct_decl = attributes
            .clone()
            .then_ignore(just(Token::Struct))
            .then(select! { Token::Ident(name) => name })
            .then(generic_params.clone())
            .then(
                select! { Token::Ident(name) => name }
                    .then_ignore(just(Token::Colon))
                    .then(ty.clone())
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|(((attributes, name), type_params), fields), span| {
                auwla_ast::Spanned::new(
                    StmtKind::StructDecl {
                        name,
                        type_params,
                        fields,
                        attributes,
                    },
                    span,
                )
            });

        // enum Name { Variant1, Variant2(type) }
        let enum_decl = attributes
            .clone()
            .then_ignore(just(Token::Enum))
            .then(select! { Token::Ident(name) => name })
            .then(generic_params.clone())
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
            .map_with_span(|(((attributes, name), type_params), variants), span| {
                auwla_ast::Spanned::new(
                    StmtKind::EnumDecl {
                        name,
                        type_params,
                        variants,
                        attributes,
                    },
                    span,
                )
            });

        // import { a, b } from './math';
        let import_stmt = just(Token::Import)
            .ignore_then(
                select! { Token::Ident(name) => name }
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .then_ignore(just(Token::From))
            .then(select! { Token::StringLit(path) => path })
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|(names, path), span| {
                auwla_ast::Spanned::new(StmtKind::Import { names, path }, span)
            });

        // export fn / export let / export var / export struct / export enum
        let export_stmt = just(Token::Export)
            .ignore_then(stmt.clone())
            .map_with_span(|s, span| {
                auwla_ast::Spanned::new(StmtKind::Export { stmt: Box::new(s) }, span)
            });

        // method_parser logic extracted
        let method_body = stmt
            .clone()
            .repeated()
            .then(expr.clone().or_not())
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        let static_kw = select! { Token::Ident(name) if name == "static" => name }
            .or_not()
            .map(|kw| kw.is_some());

        // Parse operator symbol: +, -, *, /, %, .., ..<
        let operator_symbol = choice((
            just(Token::Plus).to(auwla_ast::OperatorType::Add),
            just(Token::Minus).to(auwla_ast::OperatorType::Sub),
            just(Token::Star).to(auwla_ast::OperatorType::Mul),
            just(Token::Slash).to(auwla_ast::OperatorType::Div),
            just(Token::Percent).to(auwla_ast::OperatorType::Mod),
            just(Token::DotDotLt).to(auwla_ast::OperatorType::RangeExclusive),
            just(Token::DotDot).to(auwla_ast::OperatorType::Range),
        ));

        let method =
            attributes
                .clone()
                .then(static_kw)
                .then(
                    // Either "operator SYMBOL" or "fn name"
                    just(Token::Operator)
                        .ignore_then(operator_symbol)
                        .map(|op| (Some(op), format!("op_{}", op.method_suffix())))
                        .or(just(Token::Fn)
                            .ignore_then(select! { Token::Ident(name) => name })
                            .map(|name| (None, name)))
                )
                .then(generic_params.clone())
                .then(
                    just(Token::Self_)
                        .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
                        .map(|(_, t)| ("self".to_string(), t, false))
                        .or(just(Token::Ellipsis)
                            .or_not()
                            .then(select! { Token::Ident(name) => name })
                            .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
                            .map(|((ellipsis, n), t)| (n, t, ellipsis.is_some())))
                        .separated_by(just(Token::Comma))
                        .delimited_by(just(Token::LParen), just(Token::RParen)),
                )
                .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
                .then(
                    method_body
                        .map(|(mut body, trailing)| {
                            if let Some(e) = trailing {
                                let span = e.span.clone();
                                body.push(auwla_ast::Spanned::new(
                                    auwla_ast::StmtKind::Return(Some(e)),
                                    span,
                                ));
                            }
                            body
                        })
                        .or(just(Token::FatArrow)
                            .ignore_then(expr.clone())
                            .then_ignore(just(Token::Semicolon))
                            .map(|e| {
                                let span = e.span.clone();
                                vec![auwla_ast::Spanned::new(
                                    auwla_ast::StmtKind::Return(Some(e)),
                                    span,
                                )]
                            }))
                        .or(just(Token::Semicolon).to(Vec::new())),
                )
                .map_with_span(
                    |args: (
                        (
                            (
                                (
                                    ((Vec<auwla_ast::Attribute>, bool), (Option<auwla_ast::OperatorType>, String)),
                                    Option<Vec<String>>,
                                ),
                                Vec<(String, Option<auwla_ast::Type>, bool)>,
                            ),
                            Option<auwla_ast::Type>,
                        ),
                        Vec<auwla_ast::Stmt>,
                    ),
                     span: std::ops::Range<usize>| {
                        let (
                            ((((attributes_and_static, operator_and_name), type_params), params), return_ty),
                            body,
                        ) = args;
                        let (attributes, is_static) = attributes_and_static;
                        let (operator, name) = operator_and_name;
                        auwla_ast::Method {
                            name,
                            attributes,
                            params,
                            return_ty,
                            body,
                            is_static,
                            type_params,
                            span,
                            operator,
                        }
                    },
                );

        // type Name = Result<string, string>;
        // OR
        // type Name { fn method() { ... } }
        let type_decl = attributes
            .clone()
            .then_ignore(just(Token::Type))
            .then(select! { Token::Ident(name) => name })
            .then(generic_params.clone())
            .then(
                just(Token::Assign)
                    .ignore_then(ty.clone())
                    .then_ignore(just(Token::Semicolon))
                    .map(|aliased| (Some(aliased), Vec::new()))
                    .or(method
                        .clone()
                        .repeated()
                        .delimited_by(just(Token::LBrace), just(Token::RBrace))
                        .map(|methods| (None, methods))),
            )
            .map_with_span(
                |(((attributes, name), type_params), (aliased, methods)), span| {
                    if let Some(aliased_type) = aliased {
                        auwla_ast::Spanned::new(
                            StmtKind::TypeAlias {
                                name,
                                type_params,
                                aliased_type,
                            },
                            span,
                        )
                    } else {
                        auwla_ast::Spanned::new(
                            StmtKind::TypeDecl {
                                name,
                                type_params,
                                attributes,
                                methods,
                            },
                            span,
                        )
                    }
                },
            );

        // extend number? { ... }  |  extend Optional<T> { ... }  |  extend T?E { ... }
        // The target must be a full type expression; single-uppercase-letter bare names
        // within it (T, E, K, V …) are treated as generic type-params for the block.

        // Helper: collect all single-uppercase-letter Custom names from a type tree
        fn collect_type_vars(ty: &auwla_ast::Type, out: &mut Vec<String>) {
            match ty {
                auwla_ast::Type::Custom(name)
                    if name.len() <= 2
                        && name.chars().all(|c| c.is_ascii_uppercase()) =>
                {
                    if !out.contains(name) {
                        out.push(name.clone());
                    }
                }
                auwla_ast::Type::Optional(inner) => collect_type_vars(inner, out),
                auwla_ast::Type::Result { ok_type, err_type } => {
                    collect_type_vars(ok_type, out);
                    collect_type_vars(err_type, out);
                }
                auwla_ast::Type::Array(inner) => collect_type_vars(inner, out),
                auwla_ast::Type::Dict(k, v) => {
                    collect_type_vars(k, out);
                    collect_type_vars(v, out);
                }
                auwla_ast::Type::Generic(_, args) => {
                    for a in args {
                        collect_type_vars(a, out);
                    }
                }
                auwla_ast::Type::Function(params, ret) => {
                    for (p, _) in params {
                        collect_type_vars(p, out);
                    }
                    collect_type_vars(ret, out);
                }
                _ => {}
            }
        }

        // Helper: replace Custom(name) with TypeVar(name) when name is in type_params
        fn parameterise(ty: auwla_ast::Type, tps: &[String]) -> auwla_ast::Type {
            match ty {
                auwla_ast::Type::Custom(ref name) if tps.contains(name) => {
                    auwla_ast::Type::TypeVar(name.clone())
                }
                auwla_ast::Type::Optional(inner) => {
                    auwla_ast::Type::Optional(Box::new(parameterise(*inner, tps)))
                }
                auwla_ast::Type::Result { ok_type, err_type } => auwla_ast::Type::Result {
                    ok_type: Box::new(parameterise(*ok_type, tps)),
                    err_type: Box::new(parameterise(*err_type, tps)),
                },
                auwla_ast::Type::Array(inner) => {
                    auwla_ast::Type::Array(Box::new(parameterise(*inner, tps)))
                }
                auwla_ast::Type::Dict(k, v) => auwla_ast::Type::Dict(
                    Box::new(parameterise(*k, tps)),
                    Box::new(parameterise(*v, tps)),
                ),
                auwla_ast::Type::Generic(name, args) => auwla_ast::Type::Generic(
                    name,
                    args.into_iter().map(|a| parameterise(a, tps)).collect(),
                ),
                other => other,
            }
        }

        let extend_decl = attributes
            .clone()
            .then_ignore(just(Token::Extend))
            .then(ty.clone()) // ← full type: number?, T?E, Optional<T>, …
            .then(
                #[allow(clippy::redundant_clone)]
                method
                    .clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|((_attributes, raw_target), methods), span| {
                // Collect single-uppercase-letter customs → type params
                let mut tps: Vec<String> = Vec::new();
                collect_type_vars(&raw_target, &mut tps);

                let (type_params, target_type) = if tps.is_empty() {
                    (None, raw_target)
                } else {
                    let parameterised = parameterise(raw_target, &tps);
                    (Some(tps), parameterised)
                };

                auwla_ast::Spanned::new(
                    StmtKind::Extend {
                        type_params,
                        target_type,
                        methods,
                    },
                    span,
                )
            });

        // Expression as statement — with semicolon for most expressions,
        // but match expressions don't need a trailing semicolon
        let match_stmt = expr.clone().try_map(|e, span| {
            if let auwla_ast::ExprKind::Match { .. } = e.node {
                Ok(auwla_ast::Spanned::new(StmtKind::Expr(e), span))
            } else {
                Err(Simple::custom(span, "expected match expression"))
            }
        });

        let expr_stmt = expr
            .clone()
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|inner, span| auwla_ast::Spanned::new(StmtKind::Expr(inner), span));

        choice((
            import_stmt,
            export_stmt,
            extend_decl,
            let_stmt,
            tuple_destructure_stmt,
            destructure_stmt,
            var_stmt,
            return_stmt,
            if_stmt,
            while_stmt,
            for_stmt,
            struct_decl,
            enum_decl,
            fn_decl,
            assign_stmt,
            type_decl,
            match_stmt,
            expr_stmt,
        ))
    })
}
