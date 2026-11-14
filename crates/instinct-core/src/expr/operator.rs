//! Textual operator form and quoted token display.

use crate::lexer::Token;

/// The textual operator for a token, used when building binary nodes.
///
/// Returns the bare operator text (`+`, `==`, `and`, ...), never quoted.
/// The quoted form produced by [`Display for Token`](crate::lexer::Token)
/// exists only for human-readable error messages and must not be reused as
/// the operator stored in a [`Binary`](crate::expr::nodes::Binary) node.
pub(crate) fn operator_text(token: &Token) -> String {
    match token {
        Token::Or => "or".to_string(),
        Token::And => "and".to_string(),
        Token::Plus => "+".to_string(),
        Token::Minus => "-".to_string(),
        Token::Star => "*".to_string(),
        Token::Slash => "/".to_string(),
        Token::Percent => "%".to_string(),
        Token::Eq => "==".to_string(),
        Token::Ne => "!=".to_string(),
        Token::Lt => "<".to_string(),
        Token::Le => "<=".to_string(),
        Token::Gt => ">".to_string(),
        Token::Ge => ">=".to_string(),
        _ => String::new(),
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Plus => write!(f, "'+'"),
            Token::Minus => write!(f, "'-'"),
            Token::Star => write!(f, "'*'"),
            Token::Slash => write!(f, "'/'"),
            Token::Percent => write!(f, "'%'"),
            Token::Eq => write!(f, "'=='"),
            Token::Ne => write!(f, "'!='"),
            Token::Lt => write!(f, "'<'"),
            Token::Le => write!(f, "'<='"),
            Token::Gt => write!(f, "'>'"),
            Token::Ge => write!(f, "'>='"),
            Token::And => write!(f, "'and'"),
            Token::Or => write!(f, "'or'"),
            Token::Not => write!(f, "'not'"),
            Token::LParen => write!(f, "'('"),
            Token::RParen => write!(f, "')'"),
            Token::Comma => write!(f, "','"),
            Token::Number(_) => write!(f, "an integer"),
            Token::Text(_) => write!(f, "a text"),
            Token::Name(_) => write!(f, "a name"),
        }
    }
}
