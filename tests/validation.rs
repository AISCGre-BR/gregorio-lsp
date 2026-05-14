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

#[test]
fn multi_word_syllable_empty_note_group_warning() {
    // "foo bar baz()" — three words sharing one empty note group
    let text = "name: Test;\n%%\n(c4) foo bar baz()";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("multi-word-syllable"))
        .expect("expected multi-word-syllable diagnostic");
    assert_eq!(d.severity, Severity::Warning);
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "foo() bar() baz()");
}

#[test]
fn multi_word_syllable_with_notes_fix_text() {
    // "foo bar(gh)" — two words, last carries actual notes
    let text = "name: Test;\n%%\n(c4) foo bar(gh)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("multi-word-syllable"))
        .expect("expected multi-word-syllable diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "foo() bar(gh)");
}

#[test]
fn multi_word_syllable_no_false_positive_on_separate_groups() {
    // Each word already has its own note group — no diagnostic expected
    let text = "name: Test;\n%%\n(c4) foo(gh) bar(e) baz(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("multi-word-syllable")),
        "unexpected multi-word-syllable diagnostic on well-formed input"
    );
}

// ---------- nabc-without-header auto-fix ----------

#[test]
fn nabc_without_header_has_fix() {
    let text = "name: Test;\n%%\n(c4) test(f|vi)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("nabc-without-header"))
        .expect("expected nabc-without-header diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "nabc-lines: 1;\n");
    assert_eq!(fix.range.start.line, 1);
    assert_eq!(fix.range.start.character, 0);
    assert_eq!(fix.range.end, fix.range.start, "insertion fix must be zero-width");
}

#[test]
fn nabc_without_header_no_fix_when_header_present() {
    let text = "name: Test;\nnabc-lines: 1;\n%%\n(c4) test(f|vi)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("nabc-without-header")),
        "unexpected nabc-without-header diagnostic when header is present"
    );
}

// ---------- quilisma-missing-connector auto-fix ----------

#[test]
fn quilisma_missing_connector_has_fix() {
    let text = "name: Test;\n%%\n(c4) test(fghw i)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("quilisma-missing-connector"))
        .expect("expected quilisma-missing-connector diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "@");
    assert_eq!(fix.range.start, fix.range.end, "insertion fix must be zero-width");
}

#[test]
fn quilisma_missing_connector_no_fix_when_fused() {
    let text = "name: Test;\n%%\n(c4) test(fg@hw i)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("quilisma-missing-connector")),
        "unexpected quilisma-missing-connector when @ already present"
    );
}

// ---------- line-break-at-end-of-score auto-fix ----------

#[test]
fn line_break_at_end_of_score_z_lowercase() {
    let text = "name: Test;\n%%\n(c4) Ky(fgf) ri(hg) e(fe) *(z)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("line-break-at-end-of-score"))
        .expect("expected line-break-at-end-of-score diagnostic");
    assert_eq!(d.severity, gregorio_lsp::parser::types::Severity::Warning);
    assert!(d.message.contains("'z'"), "message should name the marker");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "", "standalone (z) should be removed entirely");
}

#[test]
fn line_break_at_end_of_score_z_uppercase() {
    let text = "name: Test;\n%%\n(c4) Ky(fgf) ri(hg) e(fe) *(Z)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("line-break-at-end-of-score"))
        .expect("expected line-break-at-end-of-score diagnostic for Z");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "");
}

#[test]
fn line_break_at_end_of_score_z_plus_variant() {
    let text = "name: Test;\n%%\n(c4) Ky(fgf) *(z+)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("line-break-at-end-of-score"))
        .expect("expected diagnostic for z+");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "");
}

#[test]
fn line_break_at_end_of_score_mixed_with_notes() {
    // Line break mixed in the same group as real notes
    let text = "name: Test;\n%%\n(c4) Ky(fgf) e(fgh z)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("line-break-at-end-of-score"))
        .expect("expected diagnostic when z follows notes in same group");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(
        fix.new_text, "(fgh)",
        "fix should strip z and keep the notes"
    );
}

#[test]
fn line_break_at_end_of_score_no_false_positive_mid_score() {
    // Line break in the MIDDLE of the score is fine — not at end
    let text = "name: Test;\n%%\n(c4) Ky(fgf) *(z) ri(hg) e(fe)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-at-end-of-score")),
        "line break in middle of score should not trigger the rule"
    );
}

#[test]
fn line_break_at_end_of_score_no_false_positive_clean_score() {
    let text = "name: Test;\n%%\n(c4) Ky(fgf) ri(hg) e(fe)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-at-end-of-score")),
        "score without trailing line break should not trigger the rule"
    );
}

#[test]
fn line_break_at_end_of_score_custos_z0_is_not_flagged() {
    // z0 is an auto-custos, NOT a line break — must not be flagged
    let text = "name: Test;\n%%\n(c4) Ky(fgf) ri(hg) e(fez0)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-at-end-of-score")),
        "z0 (auto-custos) must not be treated as a line break"
    );
}


#[test]
fn modifiers_in_fused_glyphs_has_fix() {
    let text = "name: Test;\nnabc-lines: 1;\n%%\n(c4) test(f|viS!ta)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("modifiers-in-fused-glyphs"))
        .expect("expected modifiers-in-fused-glyphs diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert!(
        fix.new_text.contains("vi!taS"),
        "fix should move modifier S to last glyph; got: {}",
        fix.new_text
    );
}

#[test]
fn modifiers_in_fused_glyphs_no_fix_when_last_has_modifier() {
    let text = "name: Test;\nnabc-lines: 1;\n%%\n(c4) test(f|vi!taS)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("modifiers-in-fused-glyphs")),
        "unexpected modifiers-in-fused-glyphs when modifier is on last glyph"
    );
}
