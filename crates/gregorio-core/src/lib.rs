//! Gregorio core library: GABC/NABC parsers, validation, semantic analysis, lint, and formatting.
//!
//! Used by `gregorio-server` (native LSP binary), `gregorio-cli` (grelint/grefmt),
//! and `gregorio-wasm` (in-process WASM for the VS Code extension).

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
