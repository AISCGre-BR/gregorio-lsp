//! Gregorio LSP library: GABC/NABC parsers, validation, semantic analysis, lint, and formatting.
//!
//! See the binaries `gregorio-lsp` (LSP server), `grelint` (CLI linter), and `grefmt` (formatter).

pub mod format;
pub mod lint;
pub mod note_ops;
pub mod parser;
pub mod validation;

#[cfg(feature = "tree-sitter")]
pub mod tree_sitter_integration;

pub use format::{format_gabc_text, FormatOptions};
pub use lint::{lint_gabc_text, LintOptions, LintSeverity};
pub use parser::types::{ParseError, ParsedDocument, Position, Range};
