//! WASM bindings for gregorio-core.
//!
//! Built with `wasm-pack build --target nodejs --out-dir ../../vscode-gregorio/wasm/gregorio_core`.
//! All public functions are callable synchronously from the extension host (Node.js).
//!
//! # Byte-range convention for shift/fill operations
//!
//! Pass `start_byte = 0, end_byte = u32::MAX` to apply the operation to the entire
//! document (equivalent to no selection).  To restrict to a selection, pass the
//! actual byte offsets of the selection boundaries in the UTF-8-encoded document.

use wasm_bindgen::prelude::*;

use gregorio_core::format::{format_gabc_text, FormatOptions};
use gregorio_core::lint::{lint_gabc_text, LintOptions, LintSeverity};
use gregorio_core::note_ops::{
    body_start_byte as core_body_start_byte, fill_empty_groups, ligatures_to_tags as core_ltt,
    parse_nabc_lines, shift_notes, tags_to_ligatures as core_ttl, ShiftDirection,
};
use gregorio_core::parser::GabcParser;

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

/// Runs the full lint pipeline on `text` and returns a JSON array of diagnostics.
///
/// `options_json` is a JSON object with optional fields:
/// - `"minSeverity"`: `"error"` | `"warning"` | `"info"` (default: `"info"`)
/// - `"ignoreCodes"`: string array of diagnostic codes to suppress
#[wasm_bindgen]
pub fn diagnostics(text: &str, options_json: &str) -> String {
    let opts = parse_lint_options(options_json);
    let errors = lint_gabc_text(text, &opts);
    serde_json::to_string(&errors.iter().map(diag_to_json).collect::<Vec<_>>())
        .unwrap_or_else(|_| "[]".to_string())
}

fn parse_lint_options(json: &str) -> LintOptions {
    let v: serde_json::Value = serde_json::from_str(json).unwrap_or_default();
    LintOptions {
        min_severity: v
            .get("minSeverity")
            .and_then(|s| s.as_str())
            .and_then(LintSeverity::parse),
        ignore_codes: v
            .get("ignoreCodes")
            .and_then(|a| a.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn diag_to_json(e: &gregorio_core::parser::types::ParseError) -> serde_json::Value {
    serde_json::json!({
        "message": e.message,
        "severity": e.severity.as_str(),
        "code": e.code,
        "range": {
            "start": { "line": e.range.start.line, "character": e.range.start.character },
            "end":   { "line": e.range.end.line,   "character": e.range.end.character   }
        },
        "fix": e.fix.as_ref().map(|f| serde_json::json!({
            "start_line": f.range.start.line,
            "start_character": f.range.start.character,
            "end_line": f.range.end.line,
            "end_character": f.range.end.character,
            "new_text": f.new_text
        }))
    })
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Formats `text` and returns the result.
///
/// `options_json` is a JSON object with optional fields:
/// - `"maxLineWidth"`: integer (default: 80)
/// - `"breakAfterClef"`: boolean
/// - `"breakAfterBar"`: boolean
#[wasm_bindgen]
pub fn format(text: &str, options_json: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(options_json).unwrap_or_default();
    let defaults = FormatOptions::default();
    let opts = FormatOptions {
        max_line_width: v
            .get("maxLineWidth")
            .and_then(|n| n.as_u64())
            .map(|n| n as usize)
            .unwrap_or(defaults.max_line_width),
        break_after_clef: v
            .get("breakAfterClef")
            .and_then(|b| b.as_bool())
            .unwrap_or(defaults.break_after_clef),
        break_after_bar: v
            .get("breakAfterBar")
            .and_then(|b| b.as_bool())
            .unwrap_or(defaults.break_after_bar),
    };
    format_gabc_text(text, &opts)
}

// ---------------------------------------------------------------------------
// Document symbols
// ---------------------------------------------------------------------------

/// Returns a JSON array of document symbols (one per header field).
#[wasm_bindgen]
pub fn document_symbols(text: &str) -> String {
    let parsed = GabcParser::new(text).parse();
    let symbols: Vec<serde_json::Value> = parsed
        .headers
        .iter()
        .map(|(name, value)| {
            let truncated: String = value.chars().take(30).collect();
            let suffix = if value.chars().count() > 30 { "..." } else { "" };
            serde_json::json!({ "name": format!("{name}: {truncated}{suffix}") })
        })
        .collect();
    serde_json::to_string(&symbols).unwrap_or_else(|_| "[]".to_string())
}

// ---------------------------------------------------------------------------
// Note shifting
// ---------------------------------------------------------------------------

/// Shifts GABC pitch letters up by one step.
///
/// `start_byte` and `end_byte` define the selection as UTF-8 byte offsets in `text`.
/// Pass `0` and `u32::MAX` to shift the entire document body.
#[wasm_bindgen]
pub fn shift_notes_up(text: &str, start_byte: u32, end_byte: u32) -> String {
    shift_notes(text, ShiftDirection::Up, byte_range(text, start_byte, end_byte))
}

/// Shifts GABC pitch letters down by one step.
#[wasm_bindgen]
pub fn shift_notes_down(text: &str, start_byte: u32, end_byte: u32) -> String {
    shift_notes(text, ShiftDirection::Down, byte_range(text, start_byte, end_byte))
}

/// Fills empty `()` groups with the last known pitch letter.
#[wasm_bindgen]
pub fn fill_empty_groups_wasm(text: &str, start_byte: u32, end_byte: u32) -> String {
    fill_empty_groups(text, byte_range(text, start_byte, end_byte))
}

/// Returns `None` when start/end cover the whole document (sentinel: end == u32::MAX).
fn byte_range(text: &str, start: u32, end: u32) -> Option<std::ops::Range<usize>> {
    if end == u32::MAX {
        None
    } else {
        Some(start as usize..end as usize)
    }
}

// ---------------------------------------------------------------------------
// Ligature ↔ <sp> tag conversions
// ---------------------------------------------------------------------------

/// Replaces æ/ǽ/œ ligatures with `<sp>` tag equivalents.
#[wasm_bindgen]
pub fn ligatures_to_tags(text: &str) -> String {
    core_ltt(text)
}

/// Replaces `<sp>` ligature tags with Unicode ligature characters.
#[wasm_bindgen]
pub fn tags_to_ligatures(text: &str) -> String {
    core_ttl(text)
}

// ---------------------------------------------------------------------------
// Header helpers
// ---------------------------------------------------------------------------

/// Returns the `nabc-lines` header value, or `0` if the header is absent.
#[wasm_bindgen]
pub fn nabc_lines(text: &str) -> u32 {
    parse_nabc_lines(text) as u32
}

/// Returns the UTF-8 byte offset of the first character after the `%%` separator.
/// Returns `0` when no separator is found.
#[wasm_bindgen]
pub fn body_start_byte(text: &str) -> u32 {
    core_body_start_byte(text) as u32
}
