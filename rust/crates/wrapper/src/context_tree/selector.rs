mod ast_display;
mod ast;
mod error;
mod matcher;
mod lexer;
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

impl<'a> ContextTree {
    pub fn get_nodes(
        &'a self,
        locator: &Locator,
        relative_to: Option<&'a ContextNode>,
    ) -> Result<Vec<&'a ContextNode>, GetNodesError> {
        let tokens: Vec<Token> = Token::lexer(&locator.selector).collect::<Result<Vec<_>, _>>()?;

        let parsed = parser().parse(&tokens).into_result()?;

        if relative_to.is_none() {
            for alt in &parsed.alternatives {
                if alt.leading_combinator.is_some() {
                    return Err(GetNodesError::NoRelativeContext(
                        "leading combinator requires relative_to".to_string(),
                    ));
                }
            }
        }

        Ok(select_nodes(self, &parsed, relative_to))
    }
}
