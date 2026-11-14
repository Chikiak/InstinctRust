//! Token types and keyword recognition for the expression lexer.

/// A single token produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Number(i64),
    Text(String),
    Name(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    LParen,
    RParen,
    Comma,
}

/// A token tagged with the line on which it appears.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub kind: Token,
    pub line: usize,
}

/// Keywords recognized by the expression lexer.
pub(super) fn keyword(name: &str) -> Option<Token> {
    match name {
        "and" => Some(Token::And),
        "or" => Some(Token::Or),
        "not" => Some(Token::Not),
        _ => None,
    }
}
