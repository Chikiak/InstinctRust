//! Evaluation context and expression AST traits.

use crate::expr::nodes::{Binary, Call, LiteralInt, LiteralText, Name, Unary};
use crate::perceptions::CreatureView;
use crate::value::Value;

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

/// Visitor over the expression AST.
///
/// Allows new operations (printing, analysis, optimization) without touching
/// the nodes. The visitor drives child traversal by calling `accept` on
/// children, which gives it control over the order.
pub trait ExprVisitor {
    type Output;
    fn visit_literal_int(&mut self, node: &LiteralInt) -> Self::Output;
    fn visit_literal_text(&mut self, node: &LiteralText) -> Self::Output;
    fn visit_name(&mut self, node: &Name) -> Self::Output;
    fn visit_call(&mut self, node: &Call) -> Self::Output;
    fn visit_unary(&mut self, node: &Unary) -> Self::Output;
    fn visit_binary(&mut self, node: &Binary) -> Self::Output;
}

/// An expression node.
///
/// Evaluation has been moved to [`ExprVisitor`] (see
/// [`InterpreterVisitor`](crate::interp::InterpreterVisitor)); nodes only know
/// how to accept themselves against a visitor.
pub trait Expr: std::fmt::Debug {
    /// Accepts a visitor and returns its output.
    fn accept(
        &self,
        visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>,
    ) -> Result<Value, String>;
    /// Clones the node into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Expr>;
    /// Returns the operator text for binary nodes (empty for other nodes).
    /// Exposed so tests can verify operators are stored unquoted.
    fn operator(&self) -> &str {
        ""
    }
}
