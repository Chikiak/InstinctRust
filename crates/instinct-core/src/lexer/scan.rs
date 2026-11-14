//! Scanning of a single expression line into tokens.

use crate::lexer::error::LexError;
use crate::lexer::token::{keyword, SpannedToken, Token};

/// Lexes a single line of expression text into tokens.
///
/// `line` is the source line number attributed to every token produced.
///
/// The lexer is **fail-fast**: on the first error it stops and returns that
/// error; no further tokens or diagnostics are produced. Inside a quoted
/// text (`"..."`) the escapes `\"` (double quote) and `\\` (backslash) are
/// recognized; a backslash followed by any other character is kept literal.
/// A text without a closing quote is an error `unterminated text literal`,
/// and an unescaped newline inside a text is the same error. Only `i64`
/// integers are supported; a literal outside `i64` range is an error.
pub fn lex(text: &str, line: usize, file: &str) -> Result<Vec<SpannedToken>, LexError> {
    let file = file.to_string();
    let mut tokens: Vec<SpannedToken> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut current_line = line;

    while i < chars.len() {
        let ch = chars[i];

        // Whitespace: spaces and tabs separate tokens; a newline bumps the
        // line counter. `\r\n` is a single unit and counts exactly once.
        if ch == ' ' || ch == '\t' {
            i += 1;
            continue;
        }
        if ch == '\r' || ch == '\n' {
            if ch == '\r' && i + 1 < chars.len() && chars[i + 1] == '\n' {
                i += 1;
            }
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
                        file.clone(),
                        "integer literal out of range".to_string(),
                        current_line,
                    ))
                }
            }
            continue;
        }

        // Text literal: `\"` and `\\` are escapes; any other backslash
        // sequence is kept literal; `#` inside is literal. A newline inside
        // an unclosed text is the same unterminated error.
        if ch == '"' {
            i += 1;
            let mut value = String::new();
            let mut terminated = false;
            while i < chars.len() {
                let c = chars[i];
                if c == '"' {
                    terminated = true;
                    i += 1;
                    break;
                }
                if c == '\\' && i + 1 < chars.len() {
                    match chars[i + 1] {
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        other => {
                            value.push('\\');
                            value.push(other);
                        }
                    }
                    i += 2;
                    continue;
                }
                if c == '\r' || c == '\n' {
                    // The newline counts as a line even if the text never
                    // closes; `i` is no longer used after the error.
                    current_line += 1;
                    return Err(LexError::new(
                        file.clone(),
                        "unterminated text literal".to_string(),
                        current_line,
                    ));
                }
                value.push(c);
                i += 1;
            }
            if !terminated {
                return Err(LexError::new(
                    file.clone(),
                    "unterminated text literal".to_string(),
                    current_line,
                ));
            }
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
                    file.clone(),
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
