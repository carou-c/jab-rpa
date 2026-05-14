use logos::Logos;

use super::error::LexingError;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(error = LexingError)]
pub enum Token {
    #[token(",")]
    Comma,

    #[token("+")]
    Plus,

    #[token("~")]
    Tilde,

    #[token(".")]
    Dot,

    #[token(":")]
    Colon,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("=")]
    Eq,

    #[token("~=")]
    TildeEq,

    #[token("^=")]
    CaretEq,

    #[token("$=")]
    DolarEq,

    #[token("*=")]
    StarEq,

    #[token("==")]
    EqEq,

    #[token("!=")]
    Ne,

    #[token("<=")]
    Le,

    #[token(">=")]
    Ge,

    #[token("<")]
    Lt,

    #[token(">")]
    Gt,

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse())]
    Int(i32),

    #[regex(r#""([^"\\]|\\.)*"|'([^'\\]|\\.)*'"#, |lex| lex.slice().to_string())]
    String(String),

    #[regex(r"[ \t\r\n]+")]
    Whitespace,

    #[regex("[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    Ident(String),
}
