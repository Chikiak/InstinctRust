//! Lexer of a single expression line.
//!
//! Produces tokens tagged with their line number. A `#` outside a quoted
//! text starts a comment that runs to the end of the line; inside a text it
//! is an ordinary character.
//!
//! Contract of the lexer:
//! - **Fail-fast**: the first error stops the analysis and no diagnostics
//!   are accumulated.
//! - **ASCII alphabet**: identifiers and digits are recognized with
//!   `is_ascii_*`; any non-ASCII character is an error `unknown character`.
//! - **Line breaks**: `\r\n`, `\n` and lone `\r` count exactly once as a
//!   line advance.

pub mod error;
pub mod scan;
pub mod token;

pub use error::LexError;
pub use scan::lex;
pub use token::{SpannedToken, Token};
