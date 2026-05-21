use crate::selector::error::*;
use std::num::ParseIntError;

#[test]
fn test_lexing_error_default() {
    let err: LexingError = Default::default();
    assert_eq!(err, LexingError::Other);
    assert_eq!(err.to_string(), "Default error");
}

#[test]
fn test_lexing_error_invalid_integer_from_parse_int() {
    let parse_err = "not_a_number".parse::<i32>().unwrap_err();
    let err = LexingError::InvalidInteger(parse_err);
    assert!(err.to_string().contains("Invalid integer"));
}

#[test]
fn test_lexing_error_from_parse_int_error() {
    let parse_err: ParseIntError = "abc".parse::<i32>().unwrap_err();
    let err: LexingError = parse_err.into();
    assert!(matches!(err, LexingError::InvalidInteger(_)));
}

#[test]
fn test_selector_parse_error_lexing_variant() {
    let lex_err = LexingError::Other;
    let err: SelectorParseError = lex_err.into();
    assert!(matches!(err, SelectorParseError::Lexing(_)));
    assert_eq!(err.to_string(), "Lexing error: Default error");
}

#[test]
fn test_selector_parse_error_parse_variant() {
    let err = SelectorParseError::Parse("unexpected token".to_string());
    assert_eq!(err.to_string(), "Parse error: unexpected token");
}

#[test]
fn test_selector_parse_error_from_chumsky_errors() {
    let simple_errors: Vec<chumsky::error::Simple<'_, crate::selector::lexer::Token>> = vec![];
    let err: SelectorParseError = simple_errors.into();
    assert!(matches!(err, SelectorParseError::Parse(_)));
}
