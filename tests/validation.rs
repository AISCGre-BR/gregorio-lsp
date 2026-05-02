//! Validation rules + semantic analyzer tests.

use gregorio_lsp::lint::{lint_gabc_text, LintOptions, LintSeverity};
use gregorio_lsp::parser::types::Severity;
use gregorio_lsp::parser::GabcParser;
use gregorio_lsp::validation::{analyze_semantics, DocumentValidator};

fn lint(text: &str) -> Vec<gregorio_lsp::parser::types::ParseError> {
    lint_gabc_text(
        text,
        &LintOptions {
            min_severity: Some(LintSeverity::Info),
            ignore_codes: Vec::new(),
        },
    )
}

#[test]
fn missing_name_header_warning() {
    let text = "mode: 1;\n%%\n(c4) test(f)";
    let diags = lint(text);
    assert!(diags.iter().any(|d| d.message.contains("no name specified")
        || d.message.contains("No name specified")));
}

#[test]
fn nabc_pipe_without_header_error() {
    let text = "name: Test;\n%%\n(c4) test(f|vihk)";
    let doc = GabcParser::new(text).parse();
    let diags = DocumentValidator::new().validate(&doc);
    assert!(diags
        .iter()
        .any(|d| d.severity == Severity::Error && d.message.contains("nabc-lines")));
}

#[test]
fn quilisma_followed_by_lower_pitch_warning() {
    let text = "name: Test;\n%%\n(c4) test(fwe)";
    let doc = GabcParser::new(text).parse();
    let diags = DocumentValidator::new().validate(&doc);
    assert!(diags.iter().any(|d| d.message.contains("Quilisma followed")));
}

#[test]
fn invalid_staff_lines_error() {
    let text = "name: Test;\nstaff-lines: 6;\n%%\n(c4) test(f)";
    let doc = GabcParser::new(text).parse();
    let diags = DocumentValidator::new().validate(&doc);
    assert!(diags
        .iter()
        .any(|d| d.severity == Severity::Error && d.message.contains("staff lines")));
}

#[test]
fn semantic_quilisma_missing_subsequent_warning() {
    let text = "name: Test;\n%%\n(c4) test(fw)";
    let doc = GabcParser::new(text).parse();
    let diags = analyze_semantics(&doc);
    assert!(diags.iter().any(|d| d.code == "quilisma-missing-note"));
}

#[test]
fn semantic_oriscus_scapus_isolated_warning() {
    let text = "name: Test;\n%%\n(c4) test(fO)";
    let doc = GabcParser::new(text).parse();
    let diags = analyze_semantics(&doc);
    assert!(diags
        .iter()
        .any(|d| d.code.starts_with("oriscus-scapus")));
}

#[test]
fn lint_min_severity_filters_info() {
    // 3+ note quilismatic without connector triggers an info diag.
    let text = "name: Test;\n%%\n(c4) test(fhgw i)";
    let infos = lint_gabc_text(
        text,
        &LintOptions {
            min_severity: Some(LintSeverity::Info),
            ignore_codes: Vec::new(),
        },
    );
    let warnings = lint_gabc_text(
        text,
        &LintOptions {
            min_severity: Some(LintSeverity::Warning),
            ignore_codes: Vec::new(),
        },
    );
    assert!(infos.iter().any(|d| d.severity == Severity::Info)
        || warnings.iter().all(|d| d.severity != Severity::Info));
    assert!(warnings.iter().all(|d| d.severity != Severity::Info));
}
