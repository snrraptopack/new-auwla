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

        let param = select! { Token::Ident(name) => name }
            .then_ignore(just(Token::Colon))
            .then(ty.clone());

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
                        } else if let Some(stmt) = body.last() {
                            if let StmtKind::Expr(ref e) = stmt.node {
                                if let auwla_ast::ExprKind::Match { .. } = e.node {
                                    let last = body.pop().unwrap();
                                    if let StmtKind::Expr(e) = last.node {
                                        body.push(auwla_ast::Spanned::new(
                                            StmtKind::Return(Some(e)),
                                            last.span,
                                        ));
                                    }
                                }
                            }
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

        let for_stmt = just(Token::For)
            .ignore_then(select! { Token::Ident(name) => name })
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .then(
                stmt.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|((binding, iterable), body), span| {
                auwla_ast::Spanned::new(
                    auwla_ast::StmtKind::For {
                        binding,
                        iterable,
                        body,
                    },
                    span,
                )
            });

        // Assignment: `target = value;`
        let assign_stmt = expr
            .clone()
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|(target, value), span| {
                auwla_ast::Spanned::new(auwla_ast::StmtKind::Assign { target, value }, span)
            });

        let struct_decl = just(Token::Struct)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(generic_params.clone())
            .then(
                select! { Token::Ident(name) => name }
                    .then_ignore(just(Token::Colon))
                    .then(ty.clone())
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|((name, type_params), fields), span| {
                auwla_ast::Spanned::new(
                    StmtKind::StructDecl {
                        name,
                        type_params,
                        fields,
                    },
                    span,
                )
            });

        // enum Name { Variant1, Variant2(type) }
        let enum_decl = just(Token::Enum)
            .ignore_then(select! { Token::Ident(name) => name })
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
            .map_with_span(|((name, type_params), variants), span| {
                auwla_ast::Spanned::new(
                    StmtKind::EnumDecl {
                        name,
                        type_params,
                        variants,
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

        // extend TypeName { fn method(self, ...) => expr; }
        let extend_type_args = ty
            .clone()
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::Lt), just(Token::Gt))
            .or_not();

        let extend_decl = just(Token::Extend)
            .ignore_then(
                select! { Token::Ident(name) => name }
                    .or(just(Token::Array).to("array".to_string())),
            )
            .then(extend_type_args)
            .then({
                let method_body = stmt
                    .clone()
                    .repeated()
                    .then(expr.clone().or_not())
                    .delimited_by(just(Token::LBrace), just(Token::RBrace));

                let static_kw = select! { Token::Ident(name) if name == "static" => name }
                    .or_not()
                    .map(|kw| kw.is_some());

                let method = attributes
                    .clone()
                    .then(static_kw)
                    .then_ignore(just(Token::Fn))
                    .then(select! { Token::Ident(name) => name })
                    .then(generic_params.clone())
                    .then(
                        select! { Token::Ident(name) => name }
                            .then(just(Token::Colon).ignore_then(ty.clone()).or_not())
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
                                        ((Vec<auwla_ast::Attribute>, bool), String),
                                        Option<Vec<String>>,
                                    ),
                                    Vec<(String, Option<auwla_ast::Type>)>,
                                ),
                                Option<auwla_ast::Type>,
                            ),
                            Vec<auwla_ast::Stmt>,
                        ),
                         span: std::ops::Range<usize>| {
                            let (
                                ((((attributes_and_static, name), type_params), params), return_ty),
                                body,
                            ) = args;
                            let (attributes, is_static) = attributes_and_static;
                            auwla_ast::Method {
                                name,
                                attributes,
                                params,
                                return_ty,
                                body,
                                is_static,
                                type_params,
                                span,
                            }
                        },
                    );

                method
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace))
            })
            .map_with_span(|((type_name, type_args_raw), methods), span| {
                let (type_params, type_args) = if let Some(args) = type_args_raw {
                    let is_type_params = args.iter().all(|t| {
                        matches!(t, auwla_ast::Type::Custom(name) if name.len() == 1 && name.chars().all(|c| c.is_ascii_uppercase()))
                    });
                    if is_type_params {
                        let params = args
                            .into_iter()
                            .map(|t| match t {
                                auwla_ast::Type::Custom(name) => name,
                                _ => String::new(),
                            })
                            .collect();
                        (Some(params), None)
                    } else {
                        (None, Some(args))
                    }
                } else {
                    (None, None)
                };
                auwla_ast::Spanned::new(
                    StmtKind::Extend {
                        type_params,
                        type_args,
                        type_name,
                        methods,
                    },
                    span,
                )
            });

        // type Name = Result<string, string>;
        let type_alias = just(Token::Type)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(generic_params.clone())
            .then_ignore(just(Token::Assign))
            .then(ty.clone())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|((name, type_params), aliased_type), span| {
                auwla_ast::Spanned::new(
                    StmtKind::TypeAlias {
                        name,
                        type_params,
                        aliased_type,
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
            type_alias,
            match_stmt,
            expr_stmt,
        ))
    })
}
