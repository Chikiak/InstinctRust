//! Unit tests for the `lexer` module.
//!
//! Split into single-responsibility submodules:
//! `diagnostics`, `numbers`, `names_keywords`, `operators`, `parens_comma`,
//! `texts`, `newlines`, `empty_comment`, `escapes`, `unterminated_text`,
//! `ascii`. Shared helpers (`lines_of`, `kinds_of`) live in `helpers`.

mod ascii;
mod diagnostics;
mod empty_comment;
mod escapes;
mod helpers;
mod names_keywords;
mod newlines;
mod numbers;
mod operators;
mod parens_comma;
mod texts;
mod unterminated_text;

pub use helpers::*;
