use auwla_ast::Type;
use auwla_lexer::token::Token;
use chumsky::prelude::*;

pub fn type_parser() -> impl Parser<Token, Type, Error = Simple<Token>> + Clone {
    recursive(|_ty| {
        let basic = select! { Token::Ident(name) => Type::Basic(name) };

        // base_type is either a basic type or basic[] (array)
        let base = basic
            .clone()
            .then(
                just(Token::LBracket)
                    .ignore_then(just(Token::RBracket))
                    .or_not(),
            )
            .map(|(ty, brackets)| {
                if brackets.is_some() {
                    Type::Array(Box::new(ty))
                } else {
                    ty
                }
            });

        // result type: base?base
        let result = base
            .clone()
            .then_ignore(just(Token::QuestionMark))
            .then(base.clone())
            .map(|(ok, err)| Type::Result {
                ok_type: Box::new(ok),
                err_type: Box::new(err),
            });

        result.or(base)
    })
}
