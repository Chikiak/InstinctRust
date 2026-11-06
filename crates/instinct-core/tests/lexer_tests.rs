//! Unit tests for the `lexer` module.

use instinct_core::lexer::{lex, SpannedToken, Token};

fn lines_of(tokens: &[SpannedToken]) -> Vec<usize> {
    tokens.iter().map(|t| t.line).collect()
}

fn kinds_of(tokens: &[SpannedToken]) -> Vec<&Token> {
    tokens.iter().map(|t| &t.kind).collect()
}

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

// ---- L4: números ----

#[test]
fn test_lex_number() {
    let tokens = lex("42", 1).unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Number(42),
            line: 1,
        }]
    );
}

#[test]
fn test_lex_integer_out_of_range() {
    let err = lex("999999999999999999999", 1).unwrap_err();
    assert_eq!(err.line, 1);
    assert_eq!(err.message, "integer literal out of range");
}

// ---- L4: nombres y palabras clave ----

#[test]
fn test_lex_name() {
    let tokens = lex("foo_bar", 1).unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Name("foo_bar".to_string()),
            line: 1,
        }]
    );
}

#[test]
fn test_lex_keywords() {
    let tokens = lex("and or not", 1).unwrap();
    let kinds = kinds_of(&tokens);
    assert_eq!(kinds, vec![&Token::And, &Token::Or, &Token::Not]);
    assert_eq!(lines_of(&tokens), vec![1, 1, 1]);
}

// ---- L4: operadores simples y de dos caracteres ----

#[test]
fn test_lex_operators() {
    let tokens = lex("+ - * / %", 1).unwrap();
    let kinds = kinds_of(&tokens);
    assert_eq!(
        kinds,
        vec![
            &Token::Plus,
            &Token::Minus,
            &Token::Star,
            &Token::Slash,
            &Token::Percent,
        ]
    );
}

#[test]
fn test_lex_two_char_operators() {
    let tokens = lex("== != <= >=", 1).unwrap();
    let kinds = kinds_of(&tokens);
    assert_eq!(kinds, vec![&Token::Eq, &Token::Ne, &Token::Le, &Token::Ge]);
}

// ---- L4: paréntesis y coma ----

#[test]
fn test_lex_parens_and_comma() {
    let tokens = lex("(,)", 1).unwrap();
    let kinds = kinds_of(&tokens);
    assert_eq!(kinds, vec![&Token::LParen, &Token::Comma, &Token::RParen]);
}

// ---- L4: textos ----

#[test]
fn test_lex_empty_text() {
    let tokens = lex(r#""""#, 1).unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Text("".to_string()),
            line: 1,
        }]
    );
}

#[test]
fn test_lex_text_with_hash() {
    let tokens = lex(r#""a#b""#, 1).unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Text("a#b".to_string()),
            line: 1,
        }]
    );
}

// ---- Saltos de línea: CR, LF y CRLF cuentan una vez ----

#[test]
fn test_lex_crlf_counts_one_line() {
    let tokens = lex("a\r\nb", 1).unwrap();
    assert_eq!(
        kinds_of(&tokens),
        vec![&Token::Name("a".to_string()), &Token::Name("b".to_string())]
    );
    assert_eq!(lines_of(&tokens), vec![1, 2]);
}

#[test]
fn test_lex_lone_cr_counts_one_line() {
    let tokens = lex("a\rb", 1).unwrap();
    assert_eq!(lines_of(&tokens), vec![1, 2]);
}

#[test]
fn test_lex_multiple_lines_numbered_correctly() {
    let tokens = lex("a 1\r\nb 2\nc 3", 1).unwrap();
    assert_eq!(lines_of(&tokens), vec![1, 1, 2, 2, 3, 3]);
}

// ---- Línea vacía y comentario: cero tokens ----

#[test]
fn test_lex_empty_line_no_tokens() {
    assert!(lex("", 1).unwrap().is_empty());
    assert!(lex("   \t", 4).unwrap().is_empty());
}

// ---- Escapes en textos ----

#[test]
fn test_lex_text_escapes() {
    let tokens = lex(r#""a\"b\\c\d""#, 1).unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Text("a\"b\\c\\d".to_string()),
            line: 1,
        }]
    );
}

// ---- Texto sin cerrar ----

#[test]
fn test_lex_unterminated_text() {
    let err = lex("\"abc", 1).unwrap_err();
    assert_eq!(err.line, 1);
    assert_eq!(err.message, "unterminated text literal");

    // Un salto de línea dentro del texto es error con la línea del salto.
    let err = lex("\"a\n", 1).unwrap_err();
    assert_eq!(err.line, 2);
    assert_eq!(err.message, "unterminated text literal");
}

#[test]
fn test_lex_text_with_interior_newline_is_error() {
    let err = lex("\"a\nb\"", 1).unwrap_err();
    assert_eq!(err.line, 2);
    assert_eq!(err.message, "unterminated text literal");
}

// ---- Identificadores son ASCII ----

#[test]
fn test_lex_non_ascii_name_errors() {
    let err = lex("naïve", 1).unwrap_err();
    assert_eq!(err.line, 1);
    assert!(err.message.contains('ï'));
}
