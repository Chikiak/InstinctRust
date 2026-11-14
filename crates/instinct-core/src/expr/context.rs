//! Fake evaluation context backed by a name -> value map, for tests.

use crate::expr::traits::EvalContext;
use crate::perceptions::CreatureView;
use crate::value::Value;

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
