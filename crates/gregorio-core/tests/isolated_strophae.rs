//! Tests for the `isolated-strophae` validation rule.

use gregorio_core::{lint_gabc_text, LintOptions, ParseError};

const CODE: &str = "isolated-strophae";

fn strophae_warnings(body: &str, nabc_lines: u32) -> Vec<ParseError> {
    let text = format!("name: Test;\nnabc-lines: {nabc_lines};\n%%\n(c4) {body}\n");
    lint_gabc_text(&text, &LintOptions::default())
        .into_iter()
        .filter(|d| d.code.as_deref() == Some(CODE))
        .collect()
}

fn count(body: &str) -> usize {
    strophae_warnings(body, 1).len()
}

#[test]
fn single_stropha_warns_without_virga_strata_hint() {
    let w = strophae_warnings("ad(f|st)", 1);
    assert_eq!(w.len(), 1);
    assert!(w[0].message.contains("found 1"), "{}", w[0].message);
    assert!(!w[0].message.contains("virga strata"), "{}", w[0].message);
}

#[test]
fn two_strophae_warn_and_suggest_virga_strata() {
    for body in ["ad(ff|ds)", "ad(ff|stst)", "ad(ff|st!st)"] {
        let w = strophae_warnings(body, 1);
        assert_eq!(w.len(), 1, "{body}");
        assert!(w[0].message.contains("found 2"), "{body}: {}", w[0].message);
        assert!(w[0].message.contains("'vs'"), "{body}: {}", w[0].message);
    }
}

#[test]
fn three_or_more_strophae_are_ok() {
    for body in [
        "ad(fff|ts)",
        "ad(fff|ststst)",
        "ad(fff|stds)",
        "ad(fff|dsst)",
        "ad(ffff|tsst)",
    ] {
        assert_eq!(count(body), 0, "{body}");
    }
}

#[test]
fn strophae_mixed_with_other_glyphs_are_not_isolated() {
    for body in ["ad(gf|vist)", "ad(hgff|clds)"] {
        assert_eq!(count(body), 0, "{body}");
    }
}

#[test]
fn snippets_not_starting_with_a_stropha_are_not_counted() {
    // Unrecognised leading characters must not let an inner `st` count.
    for body in ["ad(g|lsst)", "ad(a|lstu)", "ad(i|test2)"] {
        assert_eq!(count(body), 0, "{body}");
    }
    // Leading spacing descriptors are fine.
    assert_eq!(count("ad(f|//st)"), 1);
}

#[test]
fn modifiers_and_letters_do_not_change_the_count() {
    assert_eq!(count("ad(f|stlsc2)"), 1);
    assert_eq!(count("ad(ff|ds-)"), 1);
    assert_eq!(count("ad(fff|ts-lst1)"), 0);
}

#[test]
fn syllables_without_strophae_are_ignored() {
    assert_eq!(count("ad(f|vi) te(gh|pe) (,)"), 0);
}

#[test]
fn each_voice_is_checked_separately() {
    assert_eq!(strophae_warnings("ad(f|un|st)", 2).len(), 1);
    assert_eq!(strophae_warnings("ad(fff|un|ts)", 2).len(), 0);
}

#[test]
fn strophae_in_one_segment_of_a_longer_syllable_are_not_isolated() {
    // nabc-lines: 1 alternates gabc|nabc|gabc|nabc…: the distropha is only one
    // part of the syllable's neumes.
    assert_eq!(count("es(iv.|vi-|hh|//ds-hg|/fgf.|//to)"), 0);
    // nabc-lines: 2: voice 1 = ds1 + unlst2/cl/cl, voice 2 = ts + cllst2/po.
    assert_eq!(
        strophae_warnings("le(jjj|ds1|ts|kj/kjj|unlst2/cl/cl|cllst2/po)", 2).len(),
        0
    );
}

#[test]
fn strophae_split_across_segments_are_summed_per_voice() {
    // st + ds over two segments of the same voice = 3 strophae → ok.
    assert_eq!(count("ad(f|st|ff|ds)"), 0);
    // st + st over two segments = 2 → warning.
    assert_eq!(count("ad(f|st|f|st)"), 1);
}

#[test]
fn rule_can_be_ignored() {
    let text = "name: Test;\nnabc-lines: 1;\n%%\n(c4) ad(f|st)\n";
    let opts = LintOptions {
        ignore_codes: vec![CODE.to_string()],
        ..Default::default()
    };
    assert!(lint_gabc_text(text, &opts)
        .iter()
        .all(|d| d.code.as_deref() != Some(CODE)));
}
