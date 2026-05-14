mod ast;
mod ast_display;
mod error;
mod lexer;
mod matcher;
mod parser;

use chumsky::Parser;
use logos::Logos;

use crate::context_tree::selector::{lexer::Token, matcher::select_nodes, parser::parser};
use crate::context_tree::{ContextNode, ContextTree};

pub use crate::context_tree::selector::error::GetNodesError;

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
}

impl<'a> ContextTree {
    pub fn get_nodes(
        &'a self,
        locator: &Locator,
        relative_to: Option<&'a ContextNode>,
    ) -> Result<Vec<&'a ContextNode>, GetNodesError> {
        let tokens: Vec<Token> = Token::lexer(&locator.selector).collect::<Result<Vec<_>, _>>()?;

        let parsed = parser().parse(&tokens).into_result()?;

        Ok(select_nodes(self, &parsed, relative_to))
    }
}
