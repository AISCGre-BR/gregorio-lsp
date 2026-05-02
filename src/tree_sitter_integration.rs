//! Optional integration with the `tree-sitter-gregorio` grammar (enabled via the
//! `tree-sitter` Cargo feature). Provides parse + helpers analogous to the TS shim.

use crate::parser::types::{ParseError, Position, Range, Severity};
use tree_sitter::{Node, Parser, Point, Tree};

pub struct TreeSitterParser {
    parser: Parser,
}

impl TreeSitterParser {
    pub fn new() -> Option<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_gregorio::language();
        parser.set_language(language).ok()?;
        Some(Self { parser })
    }

    pub fn parse(&mut self, text: &str) -> Option<Tree> {
        self.parser.parse(text, None)
    }

    pub fn extract_errors(&self, tree: &Tree) -> Vec<ParseError> {
        let mut errors = Vec::new();
        let mut cursor = tree.walk();
        let root = tree.root_node();
        visit_errors(root, &mut cursor, &mut errors);
        errors
    }

    pub fn find_node_at<'tree>(&self, tree: &'tree Tree, position: Position) -> Option<Node<'tree>> {
        Some(tree.root_node().descendant_for_point_range(
            Point {
                row: position.line,
                column: position.character,
            },
            Point {
                row: position.line,
                column: position.character,
            },
        )?)
    }

    pub fn node_text<'a>(&self, node: Node<'_>, text: &'a str) -> &'a str {
        &text[node.start_byte()..node.end_byte()]
    }

    pub fn node_range(&self, node: Node<'_>) -> Range {
        Range::new(
            Position::new(node.start_position().row, node.start_position().column),
            Position::new(node.end_position().row, node.end_position().column),
        )
    }
}

fn visit_errors(node: Node<'_>, cursor: &mut tree_sitter::TreeCursor<'_>, errors: &mut Vec<ParseError>) {
    if node.has_error() && (node.kind() == "ERROR" || node.is_missing()) {
        errors.push(ParseError::new(
            format!("Syntax error: unexpected {}", node.kind()),
            Range::new(
                Position::new(node.start_position().row, node.start_position().column),
                Position::new(node.end_position().row, node.end_position().column),
            ),
            Severity::Error,
        ));
    }
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            visit_errors(child, cursor, errors);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}
