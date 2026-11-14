//! Recursive-descent parser for expression lines.

use crate::diagnostic::Diagnostic;
use crate::expr::nodes::{Binary, Call, LiteralInt, LiteralText, Name, Unary};
use crate::expr::operator::operator_text;
use crate::expr::traits::Expr;
use crate::functions::FunctionCatalog;
use crate::lexer::{lex, SpannedToken, Token};

/// A parsing error with file, line, and message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub file: String,
    pub line: usize,
    pub message: String,
}

impl ParseError {
    pub fn new(file: impl Into<String>, line: usize, message: impl Into<String>) -> Self {
        ParseError {
            file: file.into(),
            line,
            message: message.into(),
        }
    }
}

/// A recursive-descent parser for expressions.
///
/// Precedence (high to low): parentheses; unary `-` and `not`; `*` `/` `%`;
/// `+` `-`; comparisons (left to right); `and`; `or`.
struct Parser<'a> {
    tokens: Vec<SpannedToken>,
    pos: usize,
    file: String,
    line: usize,
    functions: &'a FunctionCatalog,
}

impl<'a> Parser<'a> {
    fn new(
        tokens: Vec<SpannedToken>,
        file: String,
        line: usize,
        functions: &'a FunctionCatalog,
    ) -> Self {
        Parser {
            tokens,
            pos: 0,
            file,
            line,
            functions,
        }
    }

    /// Parses the full token stream into one expression.
    fn parse_all(mut self) -> Result<Box<dyn Expr>, ParseError> {
        let expr = self.parse_or()?;
        if self.peek().is_some() {
            return self.error("unexpected token after expression");
        }
        Ok(expr)
    }

    fn error(&self, message: impl Into<String>) -> Result<Box<dyn Expr>, ParseError> {
        Err(ParseError::new(
            self.file.clone(),
            self.current_line(),
            message.into(),
        ))
    }

    fn current_line(&self) -> usize {
        self.tokens
            .get(self.pos)
            .map(|t| t.line)
            .unwrap_or(self.line)
    }

    fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<SpannedToken> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<SpannedToken, ParseError> {
        match self.peek() {
            Some(tok) if tok.kind == expected => {
                let pos = self.pos;
                self.pos += 1;
                Ok(self.tokens[pos].clone())
            }
            Some(tok) => Err(ParseError::new(
                self.file.clone(),
                self.current_line(),
                format!("expected {:?}, found {:?}", expected, tok.kind),
            )),
            None => Err(ParseError::new(
                self.file.clone(),
                self.current_line(),
                format!("expected {:?}, found end of expression", expected),
            )),
        }
    }

    // or_expr := and_expr (or and_expr)*
    fn parse_or(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut left = self.parse_and()?;
        while let Some(tok) = self.peek() {
            if tok.kind != Token::Or {
                break;
            }
            let op = match self.advance() {
                Some(tok) => tok.kind,
                None => return self.error("unexpected end of expression"),
            };
            let right = self.parse_and()?;
            left = Box::new(Binary {
                op: operator_text(&op),
                left,
                right,
            });
        }
        Ok(left)
    }

    // and_expr := comparison (and comparison)*
    fn parse_and(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut left = self.parse_comparison()?;
        while let Some(tok) = self.peek() {
            if tok.kind != Token::And {
                break;
            }
            let op = match self.advance() {
                Some(tok) => tok.kind,
                None => return self.error("unexpected end of expression"),
            };
            let right = self.parse_comparison()?;
            left = Box::new(Binary {
                op: operator_text(&op),
                left,
                right,
            });
        }
        Ok(left)
    }

    // comparison := term ((<|<=|>|>=|==|!=) term)*
    fn parse_comparison(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut left = self.parse_term()?;
        while let Some(tok) = self.peek() {
            let op = match tok.kind {
                Token::Lt => "<",
                Token::Le => "<=",
                Token::Gt => ">",
                Token::Ge => ">=",
                Token::Eq => "==",
                Token::Ne => "!=",
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = Box::new(Binary {
                op: op.to_string(),
                left,
                right,
            });
        }
        Ok(left)
    }

    // term := factor ((+|-) factor)*
    fn parse_term(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut left = self.parse_factor()?;
        while let Some(tok) = self.peek() {
            let op = match tok.kind {
                Token::Plus => "+",
                Token::Minus => "-",
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Box::new(Binary {
                op: op.to_string(),
                left,
                right,
            });
        }
        Ok(left)
    }

    // factor := unary ((*|/|%) unary)*
    fn parse_factor(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut left = self.parse_unary()?;
        while let Some(tok) = self.peek() {
            let op = match tok.kind {
                Token::Star => "*",
                Token::Slash => "/",
                Token::Percent => "%",
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Box::new(Binary {
                op: op.to_string(),
                left,
                right,
            });
        }
        Ok(left)
    }

    // unary := (-|not) unary | primary
    fn parse_unary(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        match self.peek() {
            Some(tok) if tok.kind == Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Box::new(Unary {
                    op: "-".to_string(),
                    operand,
                }))
            }
            Some(tok) if tok.kind == Token::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Box::new(Unary {
                    op: "not".to_string(),
                    operand,
                }))
            }
            _ => self.parse_primary(),
        }
    }

    // primary := number | text | name | '(' expr ')' | name ( ... ) -> error
    fn parse_primary(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let tok = match self.peek() {
            Some(t) => t.clone(),
            None => return self.error("unexpected end of expression"),
        };
        match tok.kind {
            Token::Number(n) => {
                self.advance();
                Ok(Box::new(LiteralInt { value: n }))
            }
            Token::Text(s) => {
                self.advance();
                Ok(Box::new(LiteralText { value: s }))
            }
            Token::Name(name) => {
                // A name immediately followed by `(` is a function call,
                // resolved against the function catalog at compile time.
                if self
                    .tokens
                    .get(self.pos + 1)
                    .map(|t| t.kind == Token::LParen)
                    == Some(true)
                {
                    self.advance(); // consume the name
                    self.expect(Token::LParen)?;
                    let mut args: Vec<Box<dyn Expr>> = Vec::new();
                    if self
                        .peek()
                        .map(|t| t.kind != Token::RParen)
                        .unwrap_or(false)
                    {
                        loop {
                            args.push(self.parse_or()?);
                            match self.peek() {
                                Some(tok) if tok.kind == Token::Comma => {
                                    self.advance();
                                }
                                _ => break,
                            }
                        }
                    }
                    self.expect(Token::RParen)?;
                    let function = self
                        .functions
                        .resolve_function(&name, args.len())
                        // Bug fix: attribute the error to the line of the
                        // `name` token, not the block line stored in `self.line`.
                        .map_err(|d| ParseError::new(self.file.clone(), tok.line, d.message))?;
                    return Ok(Box::new(Call { function, args }));
                }
                self.advance();
                Ok(Box::new(Name { name }))
            }
            Token::LParen => {
                self.advance();
                let inner = self.parse_or()?;
                self.expect(Token::RParen)?;
                Ok(inner)
            }
            _ => self.error(format!("unexpected token {:?}", tok.kind)),
        }
    }
}

/// Parses a single line of expression text.
///
/// `line` is the source line number attributed to any syntax error, and
/// `file` is the source file name. `functions` is the function catalog used
/// to resolve function calls and validate arity.
pub fn parse(
    text: &str,
    line: usize,
    file: &str,
    functions: &FunctionCatalog,
) -> Result<Box<dyn Expr>, Diagnostic> {
    let tokens = match lex(text, line, file) {
        Ok(tokens) => tokens,
        Err(err) => return Err(Diagnostic::from(err)),
    };
    let parser = Parser::new(tokens, file.to_string(), line, functions);
    let captured_file = parser.file.clone();
    parser
        .parse_all()
        .map_err(|err| Diagnostic::new(captured_file, err.line, err.message))
}
