//! Visitor that evaluates an expression against a context.
//! Replaces node-based evaluation: traversal is now the visitor's.

use crate::expr::{Binary, Call, EvalContext, ExprVisitor, LiteralInt, LiteralText, Name, Unary};
use crate::value::Value;

pub struct InterpreterVisitor<'a> {
    pub ctx: &'a dyn EvalContext,
}

impl ExprVisitor for InterpreterVisitor<'_> {
    type Output = Result<Value, String>;

    fn visit_literal_int(&mut self, node: &LiteralInt) -> Self::Output {
        Ok(Value::Int(node.value))
    }

    fn visit_literal_text(&mut self, node: &LiteralText) -> Self::Output {
        Ok(Value::Text(node.value.clone()))
    }

    fn visit_name(&mut self, node: &Name) -> Self::Output {
        self.ctx
            .resolve(&node.name)
            .ok_or_else(|| format!("unknown variable '{}'", node.name))
    }

    fn visit_call(&mut self, node: &Call) -> Self::Output {
        let view = self
            .ctx
            .view()
            .ok_or_else(|| format!("function '{}' has no creature view", node.function.name()))?;
        let mut evaluated = Vec::with_capacity(node.args.len());
        for arg in &node.args {
            evaluated.push(arg.accept(self)?);
        }
        node.function.call(view, &evaluated)
    }

    fn visit_unary(&mut self, node: &Unary) -> Self::Output {
        let value = node.operand.accept(self)?;
        value.unary_op(&node.op)
    }

    fn visit_binary(&mut self, node: &Binary) -> Self::Output {
        let left = node.left.accept(self)?;
        // Short-circuit: only evaluate the right operand when needed.
        match node.op.as_str() {
            "and" => {
                if !left.is_truthy() {
                    return Ok(Value::Int(0));
                }
            }
            "or" if left.is_truthy() => {
                return Ok(Value::Int(1));
            }
            _ => {}
        }
        let right = node.right.accept(self)?;
        match node.op.as_str() {
            "and" => Ok(Value::Int(if right.is_truthy() { 1 } else { 0 })),
            "or" => Ok(Value::Int(if right.is_truthy() { 1 } else { 0 })),
            _ => left.binary_op(&node.op, &right),
        }
    }
}
