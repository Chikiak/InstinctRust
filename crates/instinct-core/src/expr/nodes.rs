//! AST node types for expressions.

use crate::expr::traits::{Expr, ExprVisitor};
use crate::functions::Function;
use crate::value::Value;
use std::rc::Rc;

/// Literal integer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralInt {
    pub value: i64,
}

impl Expr for LiteralInt {
    fn accept(
        &self,
        visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>,
    ) -> Result<Value, String> {
        visitor.visit_literal_int(self)
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
    fn accept(
        &self,
        visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>,
    ) -> Result<Value, String> {
        visitor.visit_literal_text(self)
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
    fn accept(
        &self,
        visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>,
    ) -> Result<Value, String> {
        visitor.visit_name(self)
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
    fn accept(
        &self,
        visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>,
    ) -> Result<Value, String> {
        visitor.visit_call(self)
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
    fn accept(
        &self,
        visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>,
    ) -> Result<Value, String> {
        visitor.visit_unary(self)
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
    fn accept(
        &self,
        visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>,
    ) -> Result<Value, String> {
        visitor.visit_binary(self)
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
