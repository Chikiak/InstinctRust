//! Node-counting visitor test.

use instinct_core::expr::{parse, Binary, Call, ExprVisitor, LiteralInt, LiteralText, Name, Unary};
use instinct_core::functions::FunctionCatalog;
use instinct_core::value::Value;

/// Visitor that counts AST nodes. The real count lives in `self.count`;
/// each `visit_*` returns `Ok(Value::Int(0))` and composite nodes recurse
/// into their children with `accept`.
struct NodeCounter {
    count: usize,
}

impl ExprVisitor for NodeCounter {
    type Output = Result<Value, String>;

    fn visit_literal_int(&mut self, _node: &LiteralInt) -> Self::Output {
        self.count += 1;
        Ok(Value::Int(0))
    }

    fn visit_literal_text(&mut self, _node: &LiteralText) -> Self::Output {
        self.count += 1;
        Ok(Value::Int(0))
    }

    fn visit_name(&mut self, _node: &Name) -> Self::Output {
        self.count += 1;
        Ok(Value::Int(0))
    }

    fn visit_call(&mut self, node: &Call) -> Self::Output {
        self.count += 1;
        for arg in &node.args {
            arg.accept(self)?;
        }
        Ok(Value::Int(0))
    }

    fn visit_unary(&mut self, node: &Unary) -> Self::Output {
        self.count += 1;
        node.operand.accept(self)?;
        Ok(Value::Int(0))
    }

    fn visit_binary(&mut self, node: &Binary) -> Self::Output {
        self.count += 1;
        node.left.accept(self)?;
        node.right.accept(self)?;
        Ok(Value::Int(0))
    }
}

#[test]
fn test_node_counter_counts_all_nodes() {
    // `"1 + 2 * 3"` -> Binary(+) { LiteralInt(1), Binary(*) { LiteralInt(2), LiteralInt(3) } }
    let expr = parse("1 + 2 * 3", 1, "test.ins", &FunctionCatalog::default()).unwrap();
    let mut counter = NodeCounter { count: 0 };
    expr.accept(&mut counter).unwrap();
    assert_eq!(counter.count, 5, "must count the 5 AST nodes");
}
