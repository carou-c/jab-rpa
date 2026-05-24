use super::*;
use crate::selector::error::LexingError;

fn lex(input: &str) -> Result<Vec<Token>, LexingError> {
    Token::lexer(input).collect()
}

fn assert_tokens(input: &str, expected: &[Token]) {
    let tokens: Vec<Token> = lex(input).unwrap();
    assert_eq!(tokens, expected);
}

#[test]
fn test_comma() {
    assert_tokens(",", &[Token::Comma]);
}

#[test]
fn test_plus() {
    assert_tokens("+", &[Token::Plus]);
}

#[test]
fn test_tilde() {
    assert_tokens("~", &[Token::Tilde]);
}

#[test]
fn test_colon() {
    assert_tokens(":", &[Token::Colon]);
}

#[test]
fn test_star() {
    assert_tokens("*", &[Token::Star]);
}

#[test]
fn test_lbracket() {
    assert_tokens("[", &[Token::LBracket]);
}

#[test]
fn test_rbracket() {
    assert_tokens("]", &[Token::RBracket]);
}

#[test]
fn test_lparen() {
    assert_tokens("(", &[Token::LParen]);
}

#[test]
fn test_rparen() {
    assert_tokens(")", &[Token::RParen]);
}

#[test]
fn test_eq_operator() {
    assert_tokens("=", &[Token::Eq]);
}

#[test]
fn test_tilde_eq() {
    assert_tokens("~=", &[Token::TildeEq]);
}

#[test]
fn test_caret_eq() {
    assert_tokens("^=", &[Token::CaretEq]);
}

#[test]
fn test_dollar_eq() {
    assert_tokens("$=", &[Token::DolarEq]);
}

#[test]
fn test_star_eq() {
    assert_tokens("*=", &[Token::StarEq]);
}

#[test]
fn test_eq_eq() {
    assert_tokens("==", &[Token::EqEq]);
}

#[test]
fn test_not_eq() {
    assert_tokens("!=", &[Token::Ne]);
}

#[test]
fn test_less_eq() {
    assert_tokens("<=", &[Token::Le]);
}

#[test]
fn test_greater_eq() {
    assert_tokens(">=", &[Token::Ge]);
}

#[test]
fn test_less_than() {
    assert_tokens("<", &[Token::Lt]);
}

#[test]
fn test_greater_than() {
    assert_tokens(">", &[Token::Gt]);
}

#[test]
fn test_ident_simple() {
    assert_tokens("push_button", &[Token::Ident("push_button".into())]);
}

#[test]
fn test_ident_with_hyphen() {
    assert_tokens("nth-child", &[Token::Ident("nth-child".into())]);
}

#[test]
fn test_ident_with_underscore() {
    assert_tokens("_myIdent", &[Token::Ident("_myIdent".into())]);
}

#[test]
fn test_ident_single_letter() {
    assert_tokens("x", &[Token::Ident("x".into())]);
}

#[test]
fn test_int_zero() {
    assert_tokens("0", &[Token::Int(0)]);
}

#[test]
fn test_int_positive() {
    assert_tokens("42", &[Token::Int(42)]);
}

#[test]
fn test_int_negative() {
    assert_tokens("-5", &[Token::Int(-5)]);
}

#[test]
fn test_int_large() {
    assert_tokens("65535", &[Token::Int(65535)]);
}

#[test]
fn test_string_single_quoted() {
    assert_tokens("'Clear'", &[Token::String("'Clear'".into())]);
}

#[test]
fn test_string_double_quoted() {
    assert_tokens(r#""OK""#, &[Token::String(r#""OK""#.into())]);
}

#[test]
fn test_string_with_escape() {
    assert_tokens(
        r#""say \"hi\"""#,
        &[Token::String(r#""say \"hi\"""#.into())],
    );
}

#[test]
fn test_whitespace_single_space() {
    assert_tokens(" ", &[Token::Whitespace]);
}

#[test]
fn test_whitespace_multiple_spaces_merged() {
    assert_tokens("   ", &[Token::Whitespace]);
}

#[test]
fn test_whitespace_tab() {
    assert_tokens("\t", &[Token::Whitespace]);
}

#[test]
fn test_whitespace_newline() {
    assert_tokens("\n", &[Token::Whitespace]);
}

#[test]
fn test_selector_with_bracket_attr() {
    assert_tokens(
        "push_button[name='Clear']",
        &[
            Token::Ident("push_button".into()),
            Token::LBracket,
            Token::Ident("name".into()),
            Token::Eq,
            Token::String("'Clear'".into()),
            Token::RBracket,
        ],
    );
}

#[test]
fn test_selector_with_child_combinator() {
    assert_tokens(
        "dialog > push_button",
        &[
            Token::Ident("dialog".into()),
            Token::Whitespace,
            Token::Gt,
            Token::Whitespace,
            Token::Ident("push_button".into()),
        ],
    );
}

#[test]
fn test_selector_with_descendant_combinator() {
    assert_tokens(
        "dialog push_button",
        &[
            Token::Ident("dialog".into()),
            Token::Whitespace,
            Token::Ident("push_button".into()),
        ],
    );
}

#[test]
fn test_selector_with_state_class() {
    assert_tokens(
        ":require-state(enabled)",
        &[
            Token::Colon,
            Token::Ident("require-state".into()),
            Token::LParen,
            Token::Ident("enabled".into()),
            Token::RParen,
        ],
    );
}

#[test]
fn test_selector_with_pseudo_class() {
    assert_tokens(
        "push_button:not([name='x'])",
        &[
            Token::Ident("push_button".into()),
            Token::Colon,
            Token::Ident("not".into()),
            Token::LParen,
            Token::LBracket,
            Token::Ident("name".into()),
            Token::Eq,
            Token::String("'x'".into()),
            Token::RBracket,
            Token::RParen,
        ],
    );
}

#[test]
fn test_invalid_int_overflow() {
    let result: Result<Vec<Token>, LexingError> =
        Token::lexer("9999999999999999999999999").collect();
    assert!(result.is_err());
}

#[test]
fn test_comma_separated_alternatives() {
    assert_tokens(
        "a, b",
        &[
            Token::Ident("a".into()),
            Token::Comma,
            Token::Whitespace,
            Token::Ident("b".into()),
        ],
    );
}

#[test]
fn test_int_attr_operators() {
    assert_tokens(
        "[x==5]",
        &[
            Token::LBracket,
            Token::Ident("x".into()),
            Token::EqEq,
            Token::Int(5),
            Token::RBracket,
        ],
    );
}

#[test]
fn test_bool_attr() {
    assert_tokens(
        "[accessible_action]",
        &[
            Token::LBracket,
            Token::Ident("accessible_action".into()),
            Token::RBracket,
        ],
    );
}

#[test]
fn test_empty_input() {
    assert_tokens("", &[]);
}
