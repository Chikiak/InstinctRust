//! Lexer of a single expression line.
//!
//! Produces tokens tagged with their line number. A `#` outside a quoted
//! text starts a comment that runs to the end of the line; inside a text it
//! is an ordinary character.

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

/// A lexing error with the line where it occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub line: usize,
}

impl LexError {
    pub fn new(message: impl Into<String>, line: usize) -> Self {
        LexError {
            message: message.into(),
            line,
        }
    }
}

/// Keywords recognized by the expression lexer.
fn keyword(name: &str) -> Option<Token> {
    match name {
        "and" => Some(Token::And),
        "or" => Some(Token::Or),
        "not" => Some(Token::Not),
        _ => None,
    }
}

/// Lexes a single line of expression text into tokens.
///
/// `line` is the source line number attributed to every token produced.
pub fn lex(text: &str, line: usize) -> Result<Vec<SpannedToken>, LexError> {
    let mut tokens: Vec<SpannedToken> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut current_line = line;

    while i < chars.len() {
        let ch = chars[i];

        // Whitespace: spaces and tabs separate tokens; a newline bumps the
        // line counter.
        if ch == ' ' || ch == '\t' {
            i += 1;
            continue;
        }
        if ch == '\r' || ch == '\n' {
            current_line += 1;
            i += 1;
            continue;
        }

        // A comment runs to the end of the line.
        if ch == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Integer literal.
        if ch.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let literal: String = chars[start..i].iter().collect();
            match literal.parse::<i64>() {
                Ok(n) => tokens.push(SpannedToken {
                    kind: Token::Number(n),
                    line: current_line,
                }),
                Err(_) => {
                    return Err(LexError::new(
                        "integer literal out of range".to_string(),
                        current_line,
                    ))
                }
            }
            continue;
        }

        // Text literal: no escape sequences; `#` inside is literal.
        if ch == '"' {
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\n' {
                    return Err(LexError::new(
                        "unterminated text literal".to_string(),
                        current_line,
                    ));
                }
                i += 1;
            }
            if i >= chars.len() {
                return Err(LexError::new(
                    "unterminated text literal".to_string(),
                    current_line,
                ));
            }
            let value: String = chars[start..i].iter().collect();
            i += 1; // skip closing quote
            tokens.push(SpannedToken {
                kind: Token::Text(value),
                line: current_line,
            });
            continue;
        }

        // Identifier or keyword.
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let name: String = chars[start..i].iter().collect();
            if let Some(tok) = keyword(&name) {
                tokens.push(SpannedToken {
                    kind: tok,
                    line: current_line,
                });
            } else {
                tokens.push(SpannedToken {
                    kind: Token::Name(name),
                    line: current_line,
                });
            }
            continue;
        }

        // Two-character operators first.
        if i + 1 < chars.len() {
            let two: String = chars[i..=i + 1].iter().collect();
            let matched = match two.as_str() {
                "==" => Some(Token::Eq),
                "!=" => Some(Token::Ne),
                "<=" => Some(Token::Le),
                ">=" => Some(Token::Ge),
                _ => None,
            };
            if let Some(tok) = matched {
                tokens.push(SpannedToken {
                    kind: tok,
                    line: current_line,
                });
                i += 2;
                continue;
            }
        }

        // Single-character operators and punctuation.
        let kind = match ch {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '<' => Token::Lt,
            '>' => Token::Gt,
            '(' => Token::LParen,
            ')' => Token::RParen,
            ',' => Token::Comma,
            _ => {
                return Err(LexError::new(
                    format!("unknown character '{ch}'"),
                    current_line,
                ))
            }
        };
        tokens.push(SpannedToken {
            kind,
            line: current_line,
        });
        i += 1;
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_hash_inside_string_is_not_a_comment() {
        let tokens = lex(r#""a#b""#, 1).unwrap();
        let kinds: Vec<&Token> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(kinds, vec![&Token::Text("a#b".to_string())]);
    }

    #[test]
    fn test_lexer_trailing_comment_ignored() {
        let tokens = lex("x + 1   # this is a comment", 3).unwrap();
        let last_line = tokens.last().map(|t| t.line).unwrap_or(0);
        assert_eq!(last_line, 3);
        assert!(!tokens.iter().any(|t| matches!(t.kind, Token::Text(_))));
    }

    #[test]
    fn test_lexer_unknown_character_reports_line() {
        let err = lex("x @ 1", 5).unwrap_err();
        assert_eq!(err.line, 5);
        assert!(err.message.contains('@'));
    }

    #[test]
    fn test_lexer_unterminated_string_reports_line() {
        let err = lex("unterminated \"", 2).unwrap_err();
        assert_eq!(err.line, 2);
    }
}
