//! Catalog of functions and constants by name.

use crate::diagnostic::Diagnostic;
use crate::functions::{Function, Name, See};
use std::rc::Rc;

/// Catalog of functions and constants by name.
#[derive(Debug, Clone)]
pub struct FunctionCatalog {
    functions: Vec<Rc<dyn Function>>,
    /// Constant name -> value pairs.
    constants: Vec<(&'static str, i64)>,
}

impl FunctionCatalog {
    pub fn new() -> Self {
        FunctionCatalog {
            functions: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Registers a function type. Returns an error if a function with the
    /// same name is already registered.
    pub fn register(&mut self, function: Rc<dyn Function>) -> Result<(), Diagnostic> {
        if self.functions.iter().any(|f| f.name() == function.name()) {
            return Err(Diagnostic::new(
                "catalog",
                1,
                format!("function '{}' already registered", function.name()),
            ));
        }
        self.functions.push(function);
        Ok(())
    }

    /// Registers a constant (name -> value). Returns an error if a constant
    /// with the same name is already registered.
    pub fn register_constant(&mut self, name: &'static str, value: i64) -> Result<(), Diagnostic> {
        if self.constants.iter().any(|(n, _)| *n == name) {
            return Err(Diagnostic::new(
                "catalog",
                1,
                format!("constant '{name}' already registered"),
            ));
        }
        self.constants.push((name, value));
        Ok(())
    }

    /// Resolves a function by name, validating arity. Returns a diagnostic
    /// naming the function if unknown or wrong arity.
    pub fn resolve_function(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Result<Rc<dyn Function>, Diagnostic> {
        match self.functions.iter().find(|f| f.name() == name) {
            Some(function) => {
                if function.arity() != arg_count {
                    return Err(Diagnostic::new(
                        "catalog",
                        1,
                        format!(
                            "function '{}' expects {} arguments, got {}",
                            name,
                            function.arity(),
                            arg_count
                        ),
                    ));
                }
                Ok(Rc::clone(function))
            }
            None => Err(Diagnostic::new(
                "catalog",
                1,
                format!("unknown function '{name}'"),
            )),
        }
    }

    /// Returns true if `name` is a registered function.
    pub fn is_function_name(&self, name: &str) -> bool {
        self.functions.iter().any(|f| f.name() == name)
    }

    /// Resolves a constant by name. Returns `None` when the name is not a
    /// registered constant.
    pub fn resolve_constant(&self, name: &str) -> Option<i64> {
        self.constants
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
    }

    /// Returns true if `name` is a registered constant.
    pub fn is_constant_name(&self, name: &str) -> bool {
        self.constants.iter().any(|(n, _)| *n == name)
    }
}

impl Default for FunctionCatalog {
    fn default() -> Self {
        let mut c = FunctionCatalog::new();
        let _ = c.register(Rc::new(See));
        let _ = c.register(Rc::new(Name));
        let _ = c.register_constant("NONE", -1);
        let _ = c.register_constant("GROUND", 0);
        let _ = c.register_constant("OBJECT", 1);
        let _ = c.register_constant("ALLY", 2);
        let _ = c.register_constant("ENEMY", 3);
        c
    }
}
