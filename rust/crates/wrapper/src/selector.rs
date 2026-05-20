use chumsky::Parser;
use logos::Logos;

use self::{lexer::Token, parser::parser};

pub(crate) mod ast;
mod ast_display;
mod error;
mod lexer;
mod parser;

pub use self::{ast::Selector, error::SelectorParseError};

#[derive(Debug)]
pub struct Locator {
    pub selector: String,
}

impl Locator {
    pub fn new(selector: &str) -> Self {
        Self {
            selector: selector.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Selector, SelectorParseError> {
        let tokens: Vec<Token> = Token::lexer(&self.selector).collect::<Result<Vec<_>, _>>()?;
        Ok(parser().parse(&tokens).into_result()?)
    }
}
