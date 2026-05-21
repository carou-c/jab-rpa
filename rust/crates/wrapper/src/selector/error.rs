use std::num::ParseIntError;

use chumsky::error::Simple;
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

impl From<Vec<Simple<'_, Token>>> for SelectorParseError {
    fn from(simple: Vec<Simple<'_, Token>>) -> Self {
        Self::Parse(
            simple
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

#[cfg(test)]
mod tests;
