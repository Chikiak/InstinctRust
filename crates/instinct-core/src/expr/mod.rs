//! Expression nodes, recursive-descent parser, and evaluation context.
//!
//! Public surface: [`EvalContext`], [`ExprVisitor`], [`Expr`], the node types,
//! [`FakeContext`], [`ParseError`], and [`parse`].

pub mod context;
pub mod nodes;
pub mod operator;
pub mod parse;
pub mod traits;

pub use context::FakeContext;
pub use nodes::{Binary, Call, LiteralInt, LiteralText, Name, Unary};
pub use parse::{parse, ParseError};
pub use traits::{EvalContext, Expr, ExprVisitor};
