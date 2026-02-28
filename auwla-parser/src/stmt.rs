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

        let destructure_stmt = just(Token::Let)
            .ignore_then(
                select! { Token::Ident(name) => name }
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(bindings, initializer)| Stmt::DestructureLet {
                bindings,
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

        let generic_params = select! { Token::Ident(name) => name }
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::Lt), just(Token::Gt))
            .or_not();

        let fn_decl = just(Token::Fn)
            .ignore_then(select! { Token::Ident(name) => name })
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
                    .map(|(mut body, trailing_expr)| {
                        if let Some(e) = trailing_expr {
                            body.push(Stmt::Return(Some(e)));
                        } else if let Some(Stmt::Expr(auwla_ast::Expr::Match { .. })) = body.last()
                        {
                            if let Stmt::Expr(e) = body.pop().unwrap() {
                                body.push(Stmt::Return(Some(e)));
                            }
                        }
                        body
                    })
                    // Expression body: => expr;
                    .or(just(Token::FatArrow)
                        .ignore_then(expr.clone())
                        .then_ignore(just(Token::Semicolon))
                        .map(|e| vec![Stmt::Return(Some(e))])),
            )
            .map(
                |((((name, type_params), params), return_ty), body)| Stmt::Fn {
                    name,
                    type_params,
                    params,
                    return_ty,
                    body,
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
            .then(generic_params.clone())
            .then(
                select! { Token::Ident(name) => name }
                    .then_ignore(just(Token::Colon))
                    .then(ty.clone())
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|((name, type_params), fields)| Stmt::StructDecl {
                name,
                type_params,
                fields,
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
            .map(|((name, type_params), variants)| Stmt::EnumDecl {
                name,
                type_params,
                variants,
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
            .map(|(names, path)| Stmt::Import { names, path });

        // export fn / export let / export var / export struct / export enum
        let export_stmt = just(Token::Export)
            .ignore_then(stmt.clone())
            .map(|s| Stmt::Export { stmt: Box::new(s) });

        // extend TypeName { fn method(self, ...) => expr; }
        let extend_decl = just(Token::Extend)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(generic_params.clone())
            .then({
                let method_body = stmt
                    .clone()
                    .repeated()
                    .then(expr.clone().or_not())
                    .delimited_by(just(Token::LBrace), just(Token::RBrace));

                let method = just(Token::Fn)
                    .ignore_then(select! { Token::Ident(name) => name })
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
                                    body.push(Stmt::Return(Some(e)));
                                }
                                body
                            })
                            .or(just(Token::FatArrow)
                                .ignore_then(expr.clone())
                                .then_ignore(just(Token::Semicolon))
                                .map(|e| vec![Stmt::Return(Some(e))])),
                    )
                    .map(|((((name, type_params), params), return_ty), body)| {
                        let is_static = params.first().map(|(n, _)| n != "self").unwrap_or(true);
                        auwla_ast::stmt::Method {
                            name,
                            params,
                            return_ty,
                            body,
                            is_static,
                            type_params,
                        }
                    });

                method
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace))
            })
            .map(|((type_name, type_params), methods)| Stmt::Extend {
                type_params,
                type_name,
                methods,
            });

        // type Name = Result<string, string>;
        let type_alias = just(Token::Type)
            .ignore_then(select! { Token::Ident(name) => name })
            .then(generic_params.clone())
            .then_ignore(just(Token::Assign))
            .then(ty.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|((name, type_params), aliased_type)| Stmt::TypeAlias {
                name,
                type_params,
                aliased_type,
            });

        // Expression as statement \u2014 with semicolon for most expressions,
        // but match expressions don't need a trailing semicolon
        let match_stmt = expr.clone().try_map(|e, span| match &e {
            auwla_ast::Expr::Match { .. } => Ok(Stmt::Expr(e)),
            _ => Err(Simple::custom(span, "expected match expression")),
        });

        let expr_stmt = expr
            .clone()
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Expr);

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
