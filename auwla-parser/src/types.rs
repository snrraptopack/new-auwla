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
                if name == "array" || name == "Array" {
                    if let Some(mut args) = args {
                        if args.len() == 1 {
                            if let Some(inner) = args.pop() {
                                Type::Array(Box::new(inner))
                            } else {
                                Type::Generic(name, args)
                            }
                        } else {
                            // Fallback or error: array expects 1 arg
                            Type::Generic(name, args)
                        }
                    } else {
                        Type::Custom(name)
                    }
                } else if name == "dict" || name == "Dict" {
                    if let Some(mut args) = args {
                        if args.len() == 2 {
                            match (args.pop(), args.pop()) {
                                (Some(v), Some(k)) => Type::Dict(Box::new(k), Box::new(v)),
                                _ => Type::Generic(name, args),
                            }
                        } else {
                            Type::Generic(name, args)
                        }
                    } else {
                        Type::Custom(name)
                    }
                } else if let Some(args) = args {
                    Type::Generic(name, args)
                } else {
                    match name.as_str() {
                        "number" | "string" | "bool" | "void" | "char" => Type::Basic(name),
                        "Self" | "self" => Type::SelfType,
                        _ => Type::Custom(name),
                    }
                }
            });

        // Tuple type: (T1, T2, ...) - must have at least 2 elements or trailing comma
        let tuple = ty
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .try_map(|types, _span| {
                // Empty tuple () is void
                if types.is_empty() {
                    Ok(Type::Basic("void".to_string()))
                } else if types.len() == 1 {
                    // Single element without trailing comma is grouped type, not tuple
                    // Parser can't distinguish, so we treat (T) as T
                    if let Some(inner) = types.into_iter().next() {
                        Ok(inner)
                    } else {
                        Ok(Type::Basic("void".to_string()))
                    }
                } else {
                    // Multiple elements = tuple
                    Ok(Type::Tuple(types))
                }
            });

        let func = ty
            .clone()
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .then_ignore(just(Token::FatArrow))
            .then(ty.clone())
            .map(|(params, ret)| {
                let params_with_vararg = params.into_iter().map(|p| (p, false)).collect();
                Type::Function(params_with_vararg, Box::new(ret))
            });

        let array_keyword = just(Token::Array)
            .then(
                ty.clone()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::Lt), just(Token::Gt)),
            )
            .map(|(_, mut args)| {
                if args.len() == 1 {
                    if let Some(inner) = args.pop() {
                        Type::Array(Box::new(inner))
                    } else {
                        Type::Generic("array".to_string(), args)
                    }
                } else {
                    Type::Generic("array".to_string(), args)
                }
            });

        let atom = func.or(tuple).or(array_keyword).or(basic_or_custom);

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
