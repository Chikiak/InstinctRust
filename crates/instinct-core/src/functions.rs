//! Reserved functions: `see(x, y)` and `name(x, y)`.
//!
//! Functions are stateless types resolved against a catalog by name, with
//! arity validated at compile time. There are no `if nombre == ...` chains by
//! name — resolution is purely catalog-based, mirroring `Action`/`Catalog`.

use crate::diagnostic::Diagnostic;
use crate::perception::CreatureView;
use crate::value::Value;
use std::rc::Rc;

/// A reserved function of the language. Stateless: no per-creature state.
pub trait Function: std::fmt::Debug {
    /// Function name as it appears in the language (e.g. `see`, `name`).
    fn name(&self) -> &str;
    /// Number of evaluated arguments the function expects.
    fn arity(&self) -> usize;
    /// Call the function with already-evaluated arguments and the creature's
    /// view. Returns a runtime error string on failure (e.g. text argument).
    fn call(&self, view: &dyn CreatureView, args: &[Value]) -> Result<Value, String>;
    /// Clones the function into a boxed trait object.
    fn clone_box(&self) -> Rc<dyn Function>;
}

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

// ---------------------------------------------------------------------------
// see(x, y): returns a content constant for the cell at (x, y).
// ---------------------------------------------------------------------------

/// `see(x, y)`: returns -1 (NONE) out of radius, 0 (GROUND) for terrain,
/// 1 (OBJECT) for an object or out-of-map cell, 2 (ALLY) for same-faction
/// creature (including own cell), 3 (ENEMY) for other-faction creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct See;

impl Function for See {
    fn name(&self) -> &str {
        "see"
    }

    fn arity(&self) -> usize {
        2
    }

    fn call(&self, view: &dyn CreatureView, args: &[Value]) -> Result<Value, String> {
        let x = arg_as_int(args, 0, "see")?;
        let y = arg_as_int(args, 1, "see")?;
        Ok(Value::Int(view.see(x, y)))
    }

    fn clone_box(&self) -> Rc<dyn Function> {
        Rc::new(*self)
    }
}

// ---------------------------------------------------------------------------
// name(x, y): returns the text name of the element at (x, y).
// ---------------------------------------------------------------------------

/// `name(x, y)`: returns the species name for a creature, the object name
/// for an object, the terrain name otherwise, and empty text out of radius
/// or out of map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Name;

impl Function for Name {
    fn name(&self) -> &str {
        "name"
    }

    fn arity(&self) -> usize {
        2
    }

    fn call(&self, view: &dyn CreatureView, args: &[Value]) -> Result<Value, String> {
        let x = arg_as_int(args, 0, "name")?;
        let y = arg_as_int(args, 1, "name")?;
        Ok(Value::Text(view.name(x, y).to_string()))
    }

    fn clone_box(&self) -> Rc<dyn Function> {
        Rc::new(*self)
    }
}

/// Extracts the i64 argument at `index`, naming the function and argument on
/// error. Text arguments are rejected.
fn arg_as_int(args: &[Value], index: usize, func: &str) -> Result<i64, String> {
    match args.get(index) {
        Some(Value::Int(n)) => Ok(*n),
        Some(Value::Text(_)) => Err(format!(
            "argument {index} of '{func}' must be an integer, got text"
        )),
        None => Err(format!(
            "function '{func}' expects {} arguments, got {}",
            2,
            args.len()
        )),
    }
}
