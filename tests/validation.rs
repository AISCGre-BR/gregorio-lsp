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
    assert!(diags.iter().any(
        |d| d.message.contains("no name specified") || d.message.contains("No name specified")
    ));
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
    assert!(diags
        .iter()
        .any(|d| d.message.contains("Quilisma followed")));
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
    assert!(diags.iter().any(|d| d.code.starts_with("oriscus-scapus")));
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
    assert!(
        infos.iter().any(|d| d.severity == Severity::Info)
            || warnings.iter().all(|d| d.severity != Severity::Info)
    );
    assert!(warnings.iter().all(|d| d.severity != Severity::Info));
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
    assert_eq!(
        fix.range.end, fix.range.start,
        "insertion fix must be zero-width"
    );
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
    assert_eq!(
        fix.range.start, fix.range.end,
        "insertion fix must be zero-width"
    );
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
    assert_eq!(
        fix.new_text, "",
        "standalone (z) should be removed entirely"
    );
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
fn punctuation_after_note_group_has_fix() {
    // Each offending syllable here follows an empty `()` group.
    let text = "name: Test;\n%%\nfoo(), bar(); baz():";
    let diags = lint(text);
    let punct_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.code.as_deref() == Some("punctuation-after-note-group"))
        .collect();

    assert_eq!(
        punct_diags.len(),
        3,
        "expected one diagnostic per misplaced punctuation mark"
    );
    assert_eq!(punct_diags[0].severity, Severity::Warning);
    let first_fix = punct_diags[0].fix.as_ref().expect("expected first fix");
    assert_eq!(first_fix.new_text, ",() bar");
    assert_eq!(first_fix.range.start.line, 2);
    assert_eq!(first_fix.range.start.character, 3);
    assert_eq!(first_fix.range.end.line, 2);
    assert_eq!(first_fix.range.end.character, 10);

    let second_fix = punct_diags[1].fix.as_ref().expect("expected second fix");
    assert_eq!(second_fix.new_text, ";() baz");
    assert_eq!(second_fix.range.start.line, 2);
    assert_eq!(second_fix.range.start.character, 10);
    assert_eq!(second_fix.range.end.line, 2);
    assert_eq!(second_fix.range.end.character, 17);

    let third_fix = punct_diags[2].fix.as_ref().expect("expected third fix");
    assert_eq!(third_fix.new_text, ":()");
    assert_eq!(third_fix.range.start.line, 2);
    assert_eq!(third_fix.range.start.character, 17);
    assert_eq!(third_fix.range.end.line, 2);
    assert_eq!(third_fix.range.end.character, 20);
}

#[test]
fn punctuation_after_note_group_no_false_positive_when_punctuation_is_before_parentheses() {
    let text = "name: Test;\n%%\nfoo,() bar;() baz:()";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("punctuation-after-note-group")),
        "correct punctuation placement should not trigger the rule"
    );
}

#[test]
fn punctuation_after_note_group_no_false_positive_for_clean_or_internal_punctuation() {
    let text = "name: Test;\n%%\nfoo() bar() foo,bar() foo.,()";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("punctuation-after-note-group")),
        "clean syllables and punctuation already inside syllable text should not trigger the rule"
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

// ---------- duplicate-headers ----------

#[test]
fn duplicate_headers_warns_on_repeated_name() {
    let text = "name: Foo;\nname: Bar;\n%%\n(c4) test(f)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("duplicate-headers"))
        .expect("expected duplicate-headers diagnostic");
    assert_eq!(d.severity, Severity::Warning);
    assert!(d.message.contains("'name'"), "message should name the key");
}

#[test]
fn duplicate_headers_no_false_positive_unique_headers() {
    let text = "name: Foo;\nmode: 1;\n%%\n(c4) test(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("duplicate-headers")),
        "unique headers should not trigger duplicate-headers"
    );
}

#[test]
fn duplicate_headers_allows_two_annotations() {
    let text = "name: Foo;\nannotation: 1;\nannotation: 2;\n%%\n(c4) test(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("duplicate-headers")),
        "two annotation headers are allowed by GregorioTeX"
    );
}

#[test]
fn duplicate_headers_warns_on_three_annotations() {
    let text = "name: Foo;\nannotation: 1;\nannotation: 2;\nannotation: 3;\n%%\n(c4) test(f)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("duplicate-headers")),
        "three annotation headers should trigger duplicate-headers"
    );
}

// commentary is an OTHER_HEADER in GregorioTeX — unlimited entries, never warns.
#[test]
fn duplicate_headers_allows_multiple_commentary() {
    let text =
        "name: Foo;\ncommentary: First line.\ncommentary: Second line.\ncommentary: Third line.\n%%\n(c4) test(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("duplicate-headers")),
        "multiple commentary headers must not trigger duplicate-headers"
    );
}

#[test]
fn duplicate_headers_warns_on_repeated_name_not_commentary() {
    let text = "name: Foo;\nname: Bar;\ncommentary: A.\ncommentary: B.\n%%\n(c4) test(f)";
    let diags = lint(text);
    let dup: Vec<_> = diags
        .iter()
        .filter(|d| d.code.as_deref() == Some("duplicate-headers"))
        .collect();
    assert_eq!(
        dup.len(),
        1,
        "only the duplicate 'name' should trigger duplicate-headers"
    );
    assert!(
        dup[0].message.contains("name"),
        "warning should be about 'name', not 'commentary'"
    );
}

// ---------- duplicate-syllable-center ----------

#[test]
fn duplicate_syllable_center_warns_on_two_open_braces() {
    let text = "name: Test;\n%%\n(c4) {al}{le}(f)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("duplicate-syllable-center")),
        "two {{}} markers in one syllable should trigger duplicate-syllable-center"
    );
}

#[test]
fn duplicate_syllable_center_no_false_positive_single_center() {
    let text = "name: Test;\n%%\n(c4) {al}(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("duplicate-syllable-center")),
        "single {{}} marker should not trigger duplicate-syllable-center"
    );
}

// ---------- center-after-protrusion ----------

#[test]
fn center_after_protrusion_warns() {
    let text = "name: Test;\n%%\n(c4) al<pr>{le}(f)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("center-after-protrusion")),
        "{{}} after <pr> should trigger center-after-protrusion"
    );
}

#[test]
fn center_after_protrusion_no_false_positive_pr_after_center() {
    let text = "name: Test;\n%%\n(c4) {al}<pr>(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("center-after-protrusion")),
        "{{}} before <pr> should not trigger center-after-protrusion"
    );
}

// ---------- unmatched-center-close ----------

#[test]
fn unmatched_center_close_warns_and_fixes() {
    // Stray '}' with no preceding '{'
    let text = "name: Test;\n%%\n(c4) al}le(f)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("unmatched-center-close"))
        .expect("expected unmatched-center-close diagnostic");
    assert_eq!(d.severity, Severity::Warning);
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "alle", "fix should remove the stray '}}' ");
}

#[test]
fn unmatched_center_close_no_false_positive_matched() {
    let text = "name: Test;\n%%\n(c4) {al}(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("unmatched-center-close")),
        "matched {{}} should not trigger unmatched-center-close"
    );
}

// ---------- duplicate-protrusion ----------

#[test]
fn duplicate_protrusion_warns_and_fixes() {
    let text = "name: Test;\n%%\n(c4) al<pr>le<pr:0.5>(f)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("duplicate-protrusion"))
        .expect("expected duplicate-protrusion diagnostic");
    assert_eq!(d.severity, Severity::Warning);
    let fix = d.fix.as_ref().expect("expected a fix");
    // Fix should keep only the first <pr> tag
    assert_eq!(fix.new_text, "al<pr>le");
}

#[test]
fn duplicate_protrusion_no_false_positive_single_pr() {
    let text = "name: Test;\n%%\n(c4) al<pr>(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("duplicate-protrusion")),
        "single <pr> should not trigger duplicate-protrusion"
    );
}

// ---------- unclosed-center-before-protrusion ----------

#[test]
fn unclosed_center_before_protrusion_warns_and_fixes() {
    let text = "name: Test;\n%%\n(c4) {al<pr>le(f)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("unclosed-center-before-protrusion"))
        .expect("expected unclosed-center-before-protrusion diagnostic");
    assert_eq!(d.severity, Severity::Warning);
    let fix = d.fix.as_ref().expect("expected a fix");
    // Fix should insert '}' before the <pr> tag
    assert_eq!(fix.new_text, "{al}<pr>le");
}

#[test]
fn unclosed_center_before_protrusion_no_false_positive_closed() {
    let text = "name: Test;\n%%\n(c4) {al}<pr>(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("unclosed-center-before-protrusion")),
        "closed center before <pr> should not trigger the rule"
    );
}

// ============================================================
// oriscus-higher-pitch
// ============================================================

#[test]
fn oriscus_virga_strata_followed_by_higher_pitch_warns() {
    // (ggo) is virga strata: Punctum(g) + Oriscus(g). The oriscus is last in its
    // group. The next group starts on 'h' (higher pitch) → semiological violation.
    let text = "name: Test;\n%%\n(c4) test(ggo) next(h)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("oriscus-higher-pitch")),
        "virga strata followed by higher pitch must produce a diagnostic"
    );
}

#[test]
fn isolated_oriscus_followed_by_equal_pitch_warns() {
    // (go) is an isolated oriscus. The next group is on the same pitch 'g' → must warn.
    let text = "name: Test;\n%%\n(c4) test(go) next(g)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("oriscus-higher-pitch")),
        "isolated oriscus followed by equal pitch must produce a diagnostic"
    );
}

#[test]
fn oriscus_warning_message_mentions_semiological_rule() {
    // The diagnostic message must reference the semiological rule, not rendering.
    let text = "name: Test;\n%%\n(c4) test(go) next(h)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("oriscus-higher-pitch"))
        .expect("expected oriscus-higher-pitch diagnostic");
    assert!(
        d.message.to_lowercase().contains("semiolog"),
        "message should mention the semiological rule; got: {}",
        d.message
    );
}

#[test]
fn oriscus_in_salicus_no_warning() {
    // (fgoh): Punctum(f) + Oriscus(g) + Punctum(h). The oriscus is NOT the last
    // note in the group (it has a following note 'h') → salicus context → no warning.
    let text = "name: Test;\n%%\n(c4) test(fgoh)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("oriscus-higher-pitch")),
        "salicus (oriscus in middle) must not produce a diagnostic"
    );
}

#[test]
fn oriscus_in_pes_quassus_no_warning() {
    // (goh): Oriscus(g) + Punctum(h). The oriscus has a following note 'h' in
    // the same group → pes-quassus context → no warning.
    let text = "name: Test;\n%%\n(c4) test(goh)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("oriscus-higher-pitch")),
        "pes-quassus (oriscus followed by higher in same group) must not produce a diagnostic"
    );
}

#[test]
fn oriscus_followed_by_lower_pitch_no_warning() {
    // (go)(f): oriscus on 'g', next note on 'f' (lower) → correct descending use → no warning.
    let text = "name: Test;\n%%\n(c4) test(go) next(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("oriscus-higher-pitch")),
        "oriscus followed by lower pitch must not produce a diagnostic"
    );
}

#[test]
fn virga_strata_followed_by_lower_pitch_no_warning() {
    // (ggo)(f): virga strata on 'g', next note on 'f' (lower) → no warning.
    let text = "name: Test;\n%%\n(c4) test(ggo) next(f)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("oriscus-higher-pitch")),
        "virga strata followed by lower pitch must not produce a diagnostic"
    );
}

// ---------- <v>...</v> verbatim LaTeX blocks ----------

#[test]
fn verbatim_block_with_parens_does_not_create_spurious_notes() {
    // LaTeX code inside <v>...</v> may contain '(' which must NOT be treated as a
    // GABC note-group delimiter; only the real (fg) should be parsed as notes.
    let text = "name: Test;\n%%\n(c4) Glo<v>\\LatexCmd(arg)</v>(fg)";
    let doc = gregorio_lsp::parser::GabcParser::new(text).parse();
    assert_eq!(
        doc.notation.syllables.len(),
        2,
        "expected clef syllable + one lyric syllable"
    );
    let lyric = &doc.notation.syllables[1];
    assert_eq!(lyric.notes.len(), 1, "expected exactly one note group");
    let group = &lyric.notes[0];
    assert_eq!(
        group.notes.len(),
        2,
        "expected notes f and g only, not LaTeX content"
    );
    assert_eq!(group.notes[0].pitch, 'f');
    assert_eq!(group.notes[1].pitch, 'g');
}

#[test]
fn verbatim_only_syllable_no_false_positive_brace_rules() {
    // A syllable whose text is entirely <v>...</v> (with curly braces in LaTeX)
    // must not trigger duplicate-syllable-center or unmatched-center-close.
    let text = "name: Test;\n%%\n(c4) <v>\\cmd{arg1}{arg2}</v>(fg)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("duplicate-syllable-center")),
        "curly braces inside <v>...</v> must not trigger duplicate-syllable-center"
    );
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("unmatched-center-close")),
        "curly braces inside <v>...</v> must not trigger unmatched-center-close"
    );
}

#[test]
fn verbatim_block_gabc_nabc_rules_not_applied_to_latex_content() {
    // Content inside <v>...</v> must not be validated as GABC or NABC notation.
    // The quilisma rule would fire if 'w' inside the verbatim block were parsed as a note.
    let text = "name: Test;\n%%\n(c4) <v>\\GW</v>(fg)";
    let doc = gregorio_lsp::parser::GabcParser::new(text).parse();
    let diags = gregorio_lsp::validation::DocumentValidator::new().validate(&doc);
    assert!(
        !diags
            .iter()
            .any(|d| d.message.contains("Quilisma") || d.code.as_deref() == Some("quilisma-missing-note")),
        "characters in <v>...</v> must not be interpreted as GABC note modifiers"
    );
    // Verify the actual notes: only f and g from (fg)
    assert_eq!(doc.notation.syllables.len(), 2);
    let lyric = &doc.notation.syllables[1];
    assert_eq!(lyric.notes.len(), 1);
    assert_eq!(lyric.notes[0].notes.len(), 2);
}

// ---------- bar-mixed-with-notes ----------

#[test]
fn bar_mixed_with_notes_warns_comma_after_notes() {
    let text = "name: Test;\n%%\n(c4) test(fg,)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "(fg,) should trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_warns_semicolon_between_notes() {
    let text = "name: Test;\n%%\n(c4) test(fh;i)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "(fh;i) should trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_warns_colon_between_notes() {
    let text = "name: Test;\n%%\n(c4) test(f:g)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "(f:g) should trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_fix_comma_after_notes() {
    let text = "name: Test;\n%%\n(c4) test(fg,)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("bar-mixed-with-notes"))
        .expect("expected bar-mixed-with-notes diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(fg) (,)");
}

#[test]
fn bar_mixed_with_notes_fix_semicolon_between_notes() {
    let text = "name: Test;\n%%\n(c4) test(fh;i)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("bar-mixed-with-notes"))
        .expect("expected bar-mixed-with-notes diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(fh) (;) (i)");
}

#[test]
fn bar_mixed_with_notes_fix_colon_between_notes() {
    let text = "name: Test;\n%%\n(c4) test(f:g)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("bar-mixed-with-notes"))
        .expect("expected bar-mixed-with-notes diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(f) (:) (g)");
}

#[test]
fn bar_mixed_with_notes_no_false_positive_isolated_bar() {
    let text = "name: Test;\n%%\n(c4) test(fg) (,) next(h)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "isolated bar in its own group must not trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_no_false_positive_bar_with_custos() {
    let text = "name: Test;\n%%\n(c4) test(fg) (,z0) next(h)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "bar with custos and no pitch notes must not trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_no_false_positive_custos_f_plus_with_bar() {
    // (::f+Z-) — bar + explicit custos f+ + line-break Z- — must NOT trigger
    let text = "name: Test;\n%%\n(c4) test(fg) (::f+Z-)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "(::f+Z-) has no melody notes and must not trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_no_false_positive_z0_bar_clef() {
    // (z0::c3) — auto-custos + bar + clef change — must NOT trigger
    let text = "name: Test;\n%%\n(c4) test(fg) (z0::c3) next(h)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "(z0::c3) has no melody notes and must not trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_no_false_positive_explicit_custos_plus_pitch() {
    // (+f,) — explicit custos in +<pitch> form combined with bar — must NOT trigger
    let text = "name: Test;\n%%\n(c4) test(fg) (+f,)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "(+f,) has no melody notes and must not trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_no_false_positive_line_break_with_bar() {
    // (z,) — line-break + bar — must NOT trigger
    let text = "name: Test;\n%%\n(c4) test(fg) (z,)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "(z,) has no melody notes and must not trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_no_false_positive_attribute_with_colon() {
    // [nv:\cmd:value] contains ':' inside brackets; must not trigger the rule.
    let text = "name: Test;\n%%\n(c4) test(fg [nv:\\cmd:value])";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "colon inside [...] attribute must not trigger bar-mixed-with-notes"
    );
}

#[test]
fn bar_mixed_with_notes_fix_double_colon() {
    let text = "name: Test;\n%%\n(c4) test(fg::)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("bar-mixed-with-notes"))
        .expect("expected bar-mixed-with-notes diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(fg) (::)");
}

// ---------- line-break-not-at-end ----------

#[test]
fn line_break_not_at_end_warns_z_before_comma() {
    let text = "name: Test;\n%%\n(c4) test(fg) (z,)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-not-at-end")),
        "(z,) should trigger line-break-not-at-end"
    );
}

#[test]
fn line_break_not_at_end_warns_Z_plus_before_double_colon() {
    let text = "name: Test;\n%%\n(c4) test(fg) (Z+::)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-not-at-end")),
        "(Z+::) should trigger line-break-not-at-end"
    );
}

#[test]
fn line_break_not_at_end_warns_lb_in_middle_of_notes() {
    // (fg z h) — line break in middle of a note group: must trigger
    let text = "name: Test;\n%%\n(c4) test(fg z h)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-not-at-end")),
        "(fg z h) should trigger line-break-not-at-end"
    );
}

#[test]
fn line_break_not_at_end_fix_z_before_comma() {
    // bar group (no melody notes): reorder lb to end
    let text = "name: Test;\n%%\n(c4) test(fg) (z,)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("line-break-not-at-end"))
        .expect("expected line-break-not-at-end diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(,z)");
}

#[test]
fn line_break_not_at_end_fix_z_plus_before_semicolon_with_custos() {
    // (z+;f+) — bar group: lb z+ before semicolon, custos f+ after bar → (;f+z+)
    let text = "name: Test;\n%%\n(c4) test(fg) (z+;f+)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("line-break-not-at-end"))
        .expect("expected line-break-not-at-end diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(;f+z+)");
}

#[test]
fn line_break_not_at_end_fix_z0_is_not_moved() {
    // z0 is auto-custos, not a line break: (z0 z,) → (z0 ,z)
    let text = "name: Test;\n%%\n(c4) test(fg) (z0 z,)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("line-break-not-at-end"))
        .expect("expected line-break-not-at-end diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(z0 ,z)");
}

#[test]
fn line_break_not_at_end_fix_lb_in_middle_of_notes() {
    // note group: split at first lb position
    let text = "name: Test;\n%%\n(c4) test(fg z h)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("line-break-not-at-end"))
        .expect("expected line-break-not-at-end diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(fg z) (h)");
}

#[test]
fn line_break_not_at_end_no_false_positive_lb_after_bar() {
    let text = "name: Test;\n%%\n(c4) test(fg) (,z)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-not-at-end")),
        "(,z) is correct order and must not trigger line-break-not-at-end"
    );
}

#[test]
fn line_break_not_at_end_no_false_positive_custos_bar_lb() {
    // (::f+Z-) — bar first, then custos f+, then line-break Z-: correct
    let text = "name: Test;\n%%\n(c4) test(fg) (::f+Z-)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-not-at-end")),
        "(::f+Z-) has line-break at end and must not trigger line-break-not-at-end"
    );
}

#[test]
fn line_break_not_at_end_no_false_positive_z0_only() {
    // z0 is auto-custos, not a line break: (z0,) must not trigger
    let text = "name: Test;\n%%\n(c4) test(fg) (z0,)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-not-at-end")),
        "(z0,) has only auto-custos before bar and must not trigger line-break-not-at-end"
    );
}

#[test]
fn line_break_not_at_end_no_false_positive_lb_at_group_end() {
    // (fg z) — lb at end of note group: correct, must not trigger
    let text = "name: Test;\n%%\n(c4) test(fg z)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-not-at-end")),
        "(fg z) has lb at end of group and must not trigger line-break-not-at-end"
    );
}

#[test]
fn line_break_not_at_end_also_fires_alongside_bar_mixed_with_notes() {
    // (fgz,) triggers bar-mixed-with-notes AND line-break-not-at-end
    let text = "name: Test;\n%%\n(c4) test(fgz,)";
    let diags = lint(text);
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("line-break-not-at-end")),
        "line-break-not-at-end must fire for (fgz,) even when bar-mixed-with-notes also fires"
    );
    assert!(
        diags
            .iter()
            .any(|d| d.code.as_deref() == Some("bar-mixed-with-notes")),
        "bar-mixed-with-notes must still fire for (fgz,)"
    );
}

// ---------- nabc-space-in-code ----------

#[test]
fn nabc_space_in_code_warns_on_space() {
    let text = "name: Test;\nnabc-lines: 1;\n%%\n(c4) test(f| vi hk)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("nabc-space-in-code"))
        .expect("expected nabc-space-in-code diagnostic");
    assert_eq!(d.severity, Severity::Warning);
}

#[test]
fn nabc_space_in_code_has_fix_removing_spaces() {
    let text = "name: Test;\nnabc-lines: 1;\n%%\n(c4) test(f| vi hk)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("nabc-space-in-code"))
        .expect("expected nabc-space-in-code diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(f|vihk)");
}

#[test]
fn nabc_space_in_code_no_false_positive_without_space() {
    let text = "name: Test;\nnabc-lines: 1;\n%%\n(c4) test(f|vihk)";
    let diags = lint(text);
    assert!(
        !diags
            .iter()
            .any(|d| d.code.as_deref() == Some("nabc-space-in-code")),
        "NABC code without spaces must not trigger nabc-space-in-code"
    );
}

#[test]
fn nabc_space_in_code_fix_preserves_gabc_content() {
    let text = "name: Test;\nnabc-lines: 1;\n%%\n(c4) test(fg| vi hk)";
    let diags = lint(text);
    let d = diags
        .iter()
        .find(|d| d.code.as_deref() == Some("nabc-space-in-code"))
        .expect("expected nabc-space-in-code diagnostic");
    let fix = d.fix.as_ref().expect("expected a fix");
    assert_eq!(fix.new_text, "(fg|vihk)");
}

// ---------- <alt>...</alt> verbatim blocks ----------

#[test]
fn alt_block_with_parens_does_not_create_spurious_notes() {
    // <alt>...</alt> content may contain '(' (e.g. a TeX macro call); that '('
    // must NOT be treated as a GABC note-group delimiter.
    let text = "name: Test;\n%%\n(c4) Glo<alt>\\above(text)</alt>(fg)";
    let doc = gregorio_lsp::parser::GabcParser::new(text).parse();
    assert_eq!(doc.notation.syllables.len(), 2);
    let lyric = &doc.notation.syllables[1];
    assert_eq!(lyric.notes.len(), 1, "only the real (fg) note group should exist");
    assert_eq!(lyric.notes[0].notes.len(), 2);
    assert_eq!(lyric.notes[0].notes[0].pitch, 'f');
    assert_eq!(lyric.notes[0].notes[1].pitch, 'g');
}

// ---------- [nv/gv/ev:...] attributes with nested brackets ----------

#[test]
fn nv_attribute_with_nested_brackets_not_parsed_as_notes() {
    // [nv:\someCmd[opt]{arg}] — the inner '[opt]' contains ']'; a naive find(']')
    // would stop there and leave '{arg}]' to be re-parsed as GABC note content
    // ('a', 'r', 'g' are all valid pitch letters and would generate spurious notes).
    let text = "name: Test;\n%%\n(c4) test(fg [nv:\\cmd[opt]{arg}])";
    let doc = gregorio_lsp::parser::GabcParser::new(text).parse();
    // Locate the note group that contains the attribute
    let all_groups: Vec<_> = doc
        .notation
        .syllables
        .iter()
        .flat_map(|s| s.notes.iter())
        .collect();
    let group_with_attr = all_groups
        .iter()
        .find(|g| g.attributes.as_ref().is_some_and(|a| !a.is_empty()));
    let group_with_attr = group_with_attr.expect("expected a note group with [nv:...] attribute");
    let attrs = group_with_attr.attributes.as_ref().unwrap();
    assert_eq!(attrs.len(), 1, "expected exactly one attribute");
    assert_eq!(attrs[0].name, "nv");
    assert_eq!(
        attrs[0].value.as_deref(),
        Some("\\cmd[opt]{arg}"),
        "attribute value must include the full LaTeX including the nested brackets"
    );
    // No spurious notes from the attribute content
    let notes_in_group = &group_with_attr.notes;
    assert!(
        notes_in_group.iter().all(|n| matches!(n.pitch, 'f' | 'g')),
        "note group must contain only f and g, not pitches from the LaTeX content"
    );
}

#[test]
fn gv_attribute_with_nested_brackets_not_parsed_as_notes() {
    let text = "name: Test;\n%%\n(c4) test(f [gv:\\macro[a][b]])";
    let doc = gregorio_lsp::parser::GabcParser::new(text).parse();
    let all_groups: Vec<_> = doc
        .notation
        .syllables
        .iter()
        .flat_map(|s| s.notes.iter())
        .collect();
    let group = all_groups
        .iter()
        .find(|g| g.attributes.as_ref().is_some_and(|a| !a.is_empty()))
        .expect("expected a note group with [gv:...] attribute");
    let attrs = group.attributes.as_ref().unwrap();
    assert_eq!(attrs[0].name, "gv");
    assert_eq!(attrs[0].value.as_deref(), Some("\\macro[a][b]"));
}
