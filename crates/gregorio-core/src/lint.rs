//! Lint pipeline: parse + run all rules + run semantic analyzer.

use crate::parser::types::{ParseError, Severity};
use crate::parser::GabcParser;
#[cfg(feature = "tree-sitter")]
use crate::tree_sitter_integration::TreeSitterParser;
use crate::validation::{analyze_semantics, DocumentValidator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LintSeverity {
    Error,
    Warning,
    #[default]
    Info,
}

impl LintSeverity {
    fn level(self) -> u8 {
        match self {
            LintSeverity::Error => 0,
            LintSeverity::Warning => 1,
            LintSeverity::Info => 2,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "error" => LintSeverity::Error,
            "warning" => LintSeverity::Warning,
            "info" => LintSeverity::Info,
            _ => return None,
        })
    }
}

fn severity_level(s: Severity) -> u8 {
    match s {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    }
}

#[derive(Debug, Clone, Default)]
pub struct LintOptions {
    pub min_severity: Option<LintSeverity>,
    pub ignore_codes: Vec<String>,
}

pub fn lint_gabc_text(text: &str, options: &LintOptions) -> Vec<ParseError> {
    let parser = GabcParser::new(text);
    let doc = parser.parse();

    let validator = DocumentValidator::new();
    let mut all = validator.validate(&doc);
    all.extend(analyze_semantics(&doc).iter().map(|s| s.to_parse_error()));

    #[cfg(feature = "tree-sitter")]
    {
        if let Some(mut ts) = TreeSitterParser::new() {
            if let Some(tree) = ts.parse(text) {
                all.extend(ts.extract_errors(&tree, text));
                all.extend(ts.extract_query_diagnostics(&tree, text));
            }
        }
    }

    let min_level = options.min_severity.unwrap_or(LintSeverity::Info).level();
    let ignore: std::collections::HashSet<&str> =
        options.ignore_codes.iter().map(|s| s.as_str()).collect();

    let mut seen = std::collections::HashSet::new();
    all.into_iter()
        .filter(|d| {
            if let Some(code) = &d.code {
                if ignore.contains(code.as_str()) {
                    return false;
                }
            }
            severity_level(d.severity) <= min_level
                && seen.insert((
                    d.range.start.line,
                    d.range.start.character,
                    d.range.end.line,
                    d.range.end.character,
                    d.message.clone(),
                    d.code.clone(),
                ))
        })
        .collect()
}
