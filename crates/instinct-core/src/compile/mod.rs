//! Compilation of `.ins` creature files: header validation, body parsing,
//! label table, and name/arity checks against the catalog.
//!
//! Produces a [`CompileResult`] holding a [`Program`] plus accumulated
//! diagnostics. Diagnostics never abort compilation of the rest of the file.

use crate::diagnostic::Diagnostic;
use crate::program::Program;

mod comment;
mod header;
mod body;
mod assignment;
mod action;

pub use header::Header;
pub use body::compile;

/// Result of compiling a `.ins` file: the compiled program plus diagnostics
/// and, when the header is complete and valid, the parsed header attributes.
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub program: Program,
    pub diagnostics: Vec<Diagnostic>,
    pub header: Option<Header>,
}

impl CompileResult {
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}
