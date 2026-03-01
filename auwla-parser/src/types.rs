use auwla_ast::Type;
use auwla_lexer::token::Token;
use chumsky::prelude::*;

pub fn type_parser() -> impl Parser<Token, Type, Error = Simple<Token>> + Clone {
    recursive(|ty| {
        let basic_or_custom = select! { Token::Ident(name) => name }
            .then(
                ty.clone()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::Lt), just(Token::Gt))
                    .or_not(),
            )
            .map(|(name, args)| {
                if name == "array" {
                    if let Some(mut args) = args {
                        if args.len() == 1 {
                            Type::Array(Box::new(args.pop().unwrap()))
                        } else {
                            // Fallback or error: array expects 1 arg
                            Type::Generic(name, args)
                        }
                    } else {
                        Type::Custom(name)
                    }
                } else if let Some(args) = args {
                    Type::Generic(name, args)
                } else {
                    match name.as_str() {
                        "number" | "string" | "bool" | "void" => Type::Basic(name),
                        _ => Type::Custom(name),
                    }
                }
            });

        let func = ty
            .clone()
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .then_ignore(just(Token::FatArrow))
            .then(ty.clone())
            .map(|(params, ret)| Type::Function(params, Box::new(ret)));

        let array_keyword = just(Token::Array)
            .then(
                ty.clone()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::Lt), just(Token::Gt)),
            )
            .map(|(_, mut args)| {
                if args.len() == 1 {
                    Type::Array(Box::new(args.pop().unwrap()))
                } else {
                    Type::Generic("array".to_string(), args)
                }
            });

        let atom = func.or(array_keyword).or(basic_or_custom);

        // base_type with optional array brackets (support nested: number[][])
        let base = atom
            .then(
                just(Token::LBracket)
                    .ignore_then(just(Token::RBracket))
                    .repeated(),
            )
            .foldl(|ty, _| Type::Array(Box::new(ty)));

        // optional or result type: base? or base?err
        let result = base
            .clone()
            .then(just(Token::QuestionMark).ignore_then(base.clone().or_not()))
            .map(|(ok, err_opt)| match err_opt {
                Some(err) => Type::Result {
                    ok_type: Box::new(ok),
                    err_type: Box::new(err),
                },
                None => Type::Optional(Box::new(ok)),
            });

        result.or(base)
    })
}
