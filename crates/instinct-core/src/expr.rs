//! Expressions: a hierarchy of nodes that evaluate themselves.
//!
//! The parser turns a line of expression text into a tree of nodes, each
//! implementing [`Expr`]. A function call in an expression is resolved at
//! compile time against the function catalog; the resolved function is stored
//! in a [`Call`] node, so evaluation never compares names.

use crate::diagnostic::Diagnostic;
use crate::functions::{Function, FunctionCatalog};
use crate::lexer::{lex, Token};
use crate::perception::CreatureView;
use crate::value::Value;
use std::rc::Rc;

/// A context that resolves names to values during evaluation.
///
/// The real implementation resolves against perceptions, constants, and
/// variables; tests use a fake context backed by a map.
pub trait EvalContext {
    /// Returns the value bound to `name`, or `None` if it is unknown.
    fn resolve(&self, name: &str) -> Option<Value>;
    /// Returns the creature's read-only view, or `None` when this context has
    /// no associated creature (e.g. test doubles). Functions need the view.
    fn view(&self) -> Option<&dyn CreatureView> {
        let _ = self;
        None
    }
}

/// An expression node that evaluates itself against a context.
///
/// Evaluation errors are plain strings without a line; the line is added by
/// whoever executes a source line.
pub trait Expr: std::fmt::Debug {
    /// Evaluates the node against `ctx`.
    fn eval(&self, ctx: &dyn EvalContext) -> Result<Value, String>;
    /// Clones the node into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Expr>;
    /// Returns the operator text for binary nodes (empty for other nodes).
    /// Exposed so tests can verify operators are stored unquoted.
    fn operator(&self) -> &str {
        ""
    }
}

/// Literal integer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralInt {
    pub value: i64,
}

impl Expr for LiteralInt {
    fn eval(&self, _ctx: &dyn EvalContext) -> Result<Value, String> {
        Ok(Value::Int(self.value))
    }

    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(LiteralInt { value: self.value })
    }
}

/// Literal text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralText {
    pub value: String,
}

impl Expr for LiteralText {
    fn eval(&self, _ctx: &dyn EvalContext) -> Result<Value, String> {
        Ok(Value::Text(self.value.clone()))
    }

    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(LiteralText {
            value: self.value.clone(),
        })
    }
}

/// A name (variable, perception, or constant) resolved through the context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    pub name: String,
}

impl Expr for Name {
    fn eval(&self, ctx: &dyn EvalContext) -> Result<Value, String> {
        ctx.resolve(&self.name)
            .ok_or_else(|| format!("unknown variable '{}'", self.name))
    }

    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(Name {
            name: self.name.clone(),
        })
    }
}

/// A resolved function call: the function is already resolved against the
/// catalog (validating name and arity at compile time) and the arguments are
/// expression nodes evaluated at runtime.
#[derive(Debug)]
pub struct Call {
    /// The resolved function (looked up by name at compile time).
    pub function: Rc<dyn Function>,
    /// The argument expressions, evaluated left to right at call time.
    pub args: Vec<Box<dyn Expr>>,
}

impl Clone for Call {
    fn clone(&self) -> Self {
        Call {
            function: self.function.clone_box(),
            args: self.args.iter().map(|a| a.clone_box()).collect(),
        }
    }
}

impl Expr for Call {
    fn eval(&self, ctx: &dyn EvalContext) -> Result<Value, String> {
        let view = ctx
            .view()
            .ok_or_else(|| format!("function '{}' has no creature view", self.function.name()))?;
        let mut evaluated = Vec::with_capacity(self.args.len());
        for arg in &self.args {
            evaluated.push(arg.eval(ctx)?);
        }
        self.function.call(view, &evaluated)
    }

    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(Call {
            function: self.function.clone_box(),
            args: self.args.iter().map(|a| a.clone_box()).collect(),
        })
    }
}

/// A unary operation (`-` or `not`).
#[derive(Debug)]
pub struct Unary {
    pub op: String,
    pub operand: Box<dyn Expr>,
}

impl Clone for Unary {
    fn clone(&self) -> Self {
        Unary {
            op: self.op.clone(),
            operand: self.operand.clone_box(),
        }
    }
}

impl Expr for Unary {
    fn eval(&self, ctx: &dyn EvalContext) -> Result<Value, String> {
        let value = self.operand.eval(ctx)?;
        value.unary_op(&self.op)
    }

    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(Unary {
            op: self.op.clone(),
            operand: self.operand.clone_box(),
        })
    }
}

/// A binary operation.
#[derive(Debug)]
pub struct Binary {
    pub op: String,
    pub left: Box<dyn Expr>,
    pub right: Box<dyn Expr>,
}

impl Clone for Binary {
    fn clone(&self) -> Self {
        Binary {
            op: self.op.clone(),
            left: self.left.clone_box(),
            right: self.right.clone_box(),
        }
    }
}

impl Expr for Binary {
    fn eval(&self, ctx: &dyn EvalContext) -> Result<Value, String> {
        let left = self.left.eval(ctx)?;
        // Short-circuit: only evaluate the right operand when needed.
        match self.op.as_str() {
            "and" => {
                if !left.is_truthy() {
                    return Ok(Value::Int(0));
                }
            }
            "or" => {
                if left.is_truthy() {
                    return Ok(Value::Int(1));
                }
            }
            _ => {}
        }
        let right = self.right.eval(ctx)?;
        match self.op.as_str() {
            "and" => Ok(Value::Int(if right.is_truthy() { 1 } else { 0 })),
            "or" => Ok(Value::Int(if right.is_truthy() { 1 } else { 0 })),
            _ => left.binary_op(&self.op, &right),
        }
    }

    fn operator(&self) -> &str {
        &self.op
    }

    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(Binary {
            op: self.op.clone(),
            left: self.left.clone_box(),
            right: self.right.clone_box(),
        })
    }
}

/// A fake evaluation context backed by a name -> value map, for tests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FakeContext {
    pub values: Vec<(String, Value)>,
}

impl FakeContext {
    pub fn new() -> Self {
        FakeContext { values: Vec::new() }
    }

    pub fn with(mut self, name: &str, value: Value) -> Self {
        self.values.push((name.to_string(), value));
        self
    }
}

impl EvalContext for FakeContext {
    fn resolve(&self, name: &str) -> Option<Value> {
        self.values
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    }

    fn view(&self) -> Option<&dyn CreatureView> {
        None
    }
}

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
    tokens: Vec<crate::lexer::SpannedToken>,
    pos: usize,
    file: String,
    line: usize,
    functions: &'a FunctionCatalog,
}

impl<'a> Parser<'a> {
    fn new(
        tokens: Vec<crate::lexer::SpannedToken>,
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

    fn peek(&self) -> Option<&crate::lexer::SpannedToken> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<crate::lexer::SpannedToken> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<crate::lexer::SpannedToken, ParseError> {
        match self.peek() {
            Some(tok) if tok.kind == expected => {
                // peek garantiza que hay un token; lo consumimos sin desenvuelto.
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
                        .map_err(|d| ParseError::new(self.file.clone(), self.line, d.message))?;
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

/// The textual operator for a token, used when building binary nodes.
///
/// Returns the bare operator text (`+`, `==`, `and`, ...), never quoted.
/// The quoted form produced by [`Display for Token`](crate::expr::Token)
/// exists only for human-readable error messages and must not be reused as
/// the operator stored in a [`Binary`] node.
fn operator_text(token: &Token) -> String {
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
        _ => token.to_string(),
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
