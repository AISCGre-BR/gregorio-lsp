//! Optional integration with the `tree-sitter-gregorio` grammar (enabled via the
//! `tree-sitter` Cargo feature). Provides parse + helpers analogous to the TS shim.

use crate::parser::types::{ParseError, Position, Range, Severity};
use tree_sitter::{InputEdit, Node, Parser, Point, Query, QueryCursor, Tree};

pub struct TreeSitterParser {
    parser: Parser,
}

impl TreeSitterParser {
    pub fn new() -> Option<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_gregorio::language();
        parser.set_language(&language).ok()?;
        Some(Self { parser })
    }

    pub fn parse(&mut self, text: &str) -> Option<Tree> {
        self.parser.parse(text, None)
    }

    pub fn parse_with_old(&mut self, text: &str, old_tree: &Tree) -> Option<Tree> {
        self.parser.parse(text, Some(old_tree))
    }

    /// Applies a LSP-like edit range (line/character in Unicode code points) to the
    /// provided text and returns both the updated text and the corresponding InputEdit.
    pub fn apply_incremental_edit(
        old_text: &str,
        range: Range,
        replacement: &str,
    ) -> Option<(String, InputEdit)> {
        let start_byte = byte_offset_for_position(old_text, range.start)?;
        let old_end_byte = byte_offset_for_position(old_text, range.end)?;

        let mut new_text = String::with_capacity(
            old_text.len().saturating_sub(old_end_byte.saturating_sub(start_byte))
                + replacement.len(),
        );
        new_text.push_str(&old_text[..start_byte]);
        new_text.push_str(replacement);
        new_text.push_str(&old_text[old_end_byte..]);

        let new_end_byte = start_byte + replacement.len();

        let edit = InputEdit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position: point_for_byte(old_text, start_byte),
            old_end_position: point_for_byte(old_text, old_end_byte),
            new_end_position: point_for_byte(&new_text, new_end_byte),
        };

        Some((new_text, edit))
    }

    pub fn extract_errors(&self, tree: &Tree, text: &str) -> Vec<ParseError> {
        let mut errors = Vec::new();
        let mut cursor = tree.walk();
        let root = tree.root_node();
        visit_errors(root, &mut cursor, text, &mut errors);
        errors
    }

    pub fn extract_query_diagnostics(&self, tree: &Tree, text: &str) -> Vec<ParseError> {
        let mut diagnostics = Vec::new();
        let query = match Query::new(&tree_sitter_gregorio::language(), tree_sitter_gregorio::DIAGNOSTICS_QUERY) {
            Ok(q) => q,
            Err(_) => return diagnostics,
        };

        let mut cursor = QueryCursor::new();
        let capture_names = query.capture_names();
        for m in cursor.matches(&query, tree.root_node(), text.as_bytes()) {
            for c in m.captures {
                let capture_name = capture_names[c.index as usize];
                let range = self.node_range(text, c.node);
                let (message, code, severity) = match capture_name {
                    "error.syntax" => (
                        "Syntax error in gregorio notation".to_string(),
                        "ts-query-syntax-error",
                        Severity::Error,
                    ),
                    "warning.alternation" => (
                        "Ambiguous alternation snippet; verify GABC|NABC separation".to_string(),
                        "ts-query-ambiguous-alternation",
                        Severity::Warning,
                    ),
                    _ => (
                        format!("Syntax issue detected by tree-sitter ({capture_name})"),
                        "ts-query-generic",
                        Severity::Warning,
                    ),
                };
                diagnostics.push(ParseError::new(message, range, severity).with_code(code));
            }
        }

        diagnostics
    }

    pub fn find_node_at<'tree>(
        &self,
        tree: &'tree Tree,
        text: &str,
        position: Position,
    ) -> Option<Node<'tree>> {
        let target = point_for_position(position, text);
        Some(tree.root_node().descendant_for_point_range(
            target,
            target,
        )?)
    }

    pub fn node_text<'a>(&self, node: Node<'_>, text: &'a str) -> &'a str {
        &text[node.start_byte()..node.end_byte()]
    }

    pub fn node_range(&self, text: &str, node: Node<'_>) -> Range {
        Range::new(
            lsp_position_from_point(text, node.start_position()),
            lsp_position_from_point(text, node.end_position()),
        )
    }
}

fn visit_errors(
    node: Node<'_>,
    cursor: &mut tree_sitter::TreeCursor<'_>,
    text: &str,
    errors: &mut Vec<ParseError>,
) {
    if node.has_error() && (node.kind() == "ERROR" || node.is_missing()) {
        errors.push(ParseError::new(
            format!("Syntax error: unexpected {}", node.kind()),
            Range::new(
                lsp_position_from_point(text, node.start_position()),
                lsp_position_from_point(text, node.end_position()),
            ),
            Severity::Error,
        )
        .with_code("ts-syntax-error"));
    }
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            visit_errors(child, cursor, text, errors);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

fn byte_offset_for_position(text: &str, position: Position) -> Option<usize> {
    let mut line = 0usize;
    let mut col_chars = 0usize;
    for (idx, ch) in text.char_indices() {
        if line == position.line && col_chars == position.character {
            return Some(idx);
        }
        if ch == '\n' {
            line += 1;
            col_chars = 0;
            if line > position.line {
                return Some(idx + ch.len_utf8());
            }
        } else if line == position.line {
            col_chars += 1;
        }
    }

    if line == position.line && col_chars == position.character {
        return Some(text.len());
    }
    None
}

fn point_for_byte(text: &str, target_byte: usize) -> Point {
    let mut row = 0usize;
    let mut line_start = 0usize;

    for (idx, ch) in text.char_indices() {
        if idx >= target_byte {
            break;
        }
        if ch == '\n' {
            row += 1;
            line_start = idx + ch.len_utf8();
        }
    }

    Point {
        row,
        column: target_byte.saturating_sub(line_start),
    }
}

fn lsp_position_from_point(text: &str, point: Point) -> Position {
    let mut row = 0usize;
    let mut line_start = 0usize;

    for (idx, ch) in text.char_indices() {
        if row == point.row {
            break;
        }
        if ch == '\n' {
            row += 1;
            line_start = idx + ch.len_utf8();
        }
    }

    let target_byte = line_start.saturating_add(point.column);
    let mut chars = 0usize;
    for (idx, _) in text.char_indices() {
        if idx < line_start {
            continue;
        }
        if idx >= target_byte {
            break;
        }
        chars += 1;
    }

    Position::new(point.row, chars)
}

fn point_for_position(position: Position, text: &str) -> Point {
    if let Some(byte) = byte_offset_for_position(text, position) {
        point_for_byte(text, byte)
    } else {
        Point {
            row: position.line,
            column: position.character,
        }
    }
}
