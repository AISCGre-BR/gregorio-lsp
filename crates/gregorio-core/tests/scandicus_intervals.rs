//! Tests for the `scandicus-intervals` validation rule.

use gregorio_core::{lint_gabc_text, LintOptions, ParseError};

const CODE: &str = "scandicus-intervals";

fn warnings_with_header(header: &str, body: &str) -> Vec<ParseError> {
    let text = format!("name: Test;\n{header}%%\n(c4) {body}\n");
    lint_gabc_text(&text, &LintOptions::default())
        .into_iter()
        .filter(|d| d.code.as_deref() == Some(CODE))
        .collect()
}

fn warnings(group: &str) -> Vec<ParseError> {
    warnings_with_header("", &format!("ad({group})"))
}

fn assert_ok(groups: &[&str]) {
    for g in groups {
        let w = warnings(g);
        assert!(w.is_empty(), "({g}) should be accepted: {w:?}");
    }
}

fn assert_warns(groups: &[&str], fragment: &str) {
    for g in groups {
        let w = warnings(g);
        assert_eq!(w.len(), 1, "({g}) should warn once: {w:?}");
        assert!(w[0].message.contains(fragment), "({g}): {}", w[0].message);
    }
}

const LEAP: &str = "must be a step apart";
const FIFTH: &str = "exceeds a perfect fifth";
const THIRD: &str = "exceeds a third";

#[test]
fn examples_from_the_specification() {
    assert_ok(&["dfh", "dgh", "dhi"]);
    assert_warns(&["dgi", "dhj"], LEAP);
}

#[test]
fn small_intervals_are_ok() {
    assert_ok(&["def", "deg", "dfg", "dfh"]);
}

#[test]
fn first_interval_beyond_a_fifth_warns() {
    assert_warns(&["dij", "dkl"], FIFTH);
}

#[test]
fn second_interval_beyond_a_third_warns() {
    assert_warns(&["dfj", "deh"], THIRD);
}

#[test]
fn both_limits_exceeded_report_both() {
    let w = warnings("dim");
    assert_eq!(w.len(), 2, "{w:?}");
    assert!(w.iter().any(|d| d.message.contains(FIFTH)));
    assert!(w.iter().any(|d| d.message.contains(THIRD)));
}

#[test]
fn connectors_and_spaces_are_ignored() {
    assert_ok(&["d/g!h", "d@gh", "d g h", "d//g`h", "dg/ h"]);
    assert_warns(&["d g i", "d@gi", "dg//i", "d!g/i"], LEAP);
}

#[test]
fn salicus_and_other_shapes_follow_the_same_rule() {
    assert_ok(&["dgoh", "dgvh", "dfwh", "d.g_h'"]);
    assert_warns(&["dgoi", "dGi"], LEAP);
}

#[test]
fn accidentals_are_not_counted_as_notes() {
    assert_ok(&["dgixh"]);
    assert_warns(&["dgixi"], LEAP);
}

#[test]
fn groups_that_are_not_three_note_scandicus_are_ignored() {
    assert_ok(&[
        "dg", "dgik", "fdgi", "dgig", "igd", "dgg", "ddi", "dgi.dgi", "c4",
    ]);
}

#[test]
fn diagnostic_spans_the_three_notes() {
    let w = warnings("d/g!i");
    assert_eq!(w.len(), 1);
    let r = w[0].range;
    assert_eq!(r.start.line, r.end.line);
    assert_eq!(r.end.character - r.start.character, "d/g!i".len());
}

#[test]
fn only_gabc_notes_are_considered_with_nabc() {
    let header = "nabc-lines: 1;\n";
    assert!(warnings_with_header(header, "ad(dgh|sc)").is_empty());
    assert_eq!(warnings_with_header(header, "ad(dgi|sc)").len(), 1);
}
