//! Gregorio LSP library: GABC/NABC parsers, validation, semantic analysis, and lint.
//!
//! See the binaries `gregorio-lsp` (LSP server) and `gregolint` (CLI linter).

pub mod lint;
pub mod note_ops;
pub mod parser;
pub mod validation;

#[cfg(feature = "tree-sitter")]
pub mod tree_sitter_integration;

pub use lint::{lint_gabc_text, LintOptions, LintSeverity};
pub use parser::types::{ParseError, ParsedDocument, Position, Range};
