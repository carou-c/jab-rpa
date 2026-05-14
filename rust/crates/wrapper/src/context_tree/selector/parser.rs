use chumsky::prelude::*;

use super::ast::Selector;
use super::lexer::Token;

pub fn parser<'src>() -> impl Parser<'src, &'src [Token], Selector, extra::Err<Simple<'src, Token>>>
{
    end().map(|_| Selector {
        alternatives: vec![],
    })
}
