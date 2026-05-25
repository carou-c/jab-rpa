use std::num::ParseIntError;

use chumsky::error::Rich;
use thiserror::Error;

use super::lexer::Token;

#[derive(Default, Debug, Clone, PartialEq, Error)]
pub enum LexingError {
    #[error("Invalid integer: {0}")]
    InvalidInteger(#[from] ParseIntError),
    #[default]
    #[error("Default error")]
    Other,
}

#[derive(Debug, Error)]
pub enum SelectorParseError {
    #[error("Lexing error: {0}")]
    Lexing(#[from] LexingError),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl From<Vec<Rich<'_, Token>>> for SelectorParseError {
    fn from(rich: Vec<Rich<'_, Token>>) -> Self {
        Self::Parse(
            rich.into_iter()
                .map(|r| format!("{:?}", r))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

#[cfg(test)]
mod tests;
