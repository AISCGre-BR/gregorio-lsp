//! Corpus integration test: parse the bundled Kyrie XVI example end-to-end.

use std::fs;
use std::path::PathBuf;

use gregorio_lsp::lint::{lint_gabc_text, LintOptions, LintSeverity};
use gregorio_lsp::parser::types::Severity;
use gregorio_lsp::parser::GabcParser;

fn example_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("examples");
    p.push(name);
    p
}

fn read_example(name: &str) -> String {
    fs::read_to_string(example_path(name)).expect("example exists")
}

#[test]
fn parses_kyrie_xvi() {
    let text = read_example("kyrie-xvi.gabc");
    let doc = GabcParser::new(&text).parse();
    assert!(doc.headers.get("name").is_some(), "name header parsed");
    assert!(!doc.notation.syllables.is_empty(), "syllables collected");
    let parser_errors: Vec<_> = doc
        .errors
        .iter()
        .filter(|e| e.severity == Severity::Error)
        .collect();
    assert!(
        parser_errors.is_empty(),
        "no parse errors expected, got: {parser_errors:?}"
    );
}

#[test]
fn parses_nabc_example() {
    let text = read_example("nabc-example.gabc");
    let doc = GabcParser::new(&text).parse();
    assert!(doc.headers.get("nabc-lines").is_some());
    let has_nabc = doc
        .notation
        .syllables
        .iter()
        .flat_map(|s| s.notes.iter())
        .any(|g| g.nabc.as_ref().map(|v| !v.is_empty()).unwrap_or(false));
    assert!(has_nabc, "at least one note group has NABC data");
}

#[test]
fn lints_errors_example_produces_diagnostics() {
    let text = read_example("errors-example.gabc");
    let diags = lint_gabc_text(
        &text,
        &LintOptions {
            min_severity: Some(LintSeverity::Info),
            ignore_codes: Vec::new(),
        },
    );
    assert!(
        !diags.is_empty(),
        "errors-example should yield lint diagnostics"
    );
}
