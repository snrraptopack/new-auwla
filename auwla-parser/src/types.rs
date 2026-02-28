use auwla_ast::Type;
use auwla_lexer::token::Token;
use chumsky::prelude::*;

pub fn type_parser() -> impl Parser<Token, Type, Error = Simple<Token>> + Clone {
    recursive(|_ty| {
        let basic = select! { Token::Ident(name) => Type::Basic(name) };

        let result = basic
            .clone()
            .then_ignore(just(Token::QuestionMark))
            .then(basic.clone())
            .map(|(ok, err)| Type::Result {
                ok_type: Box::new(ok),
                err_type: Box::new(err),
            });

        result.or(basic)
    })
}
