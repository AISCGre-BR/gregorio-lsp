//! Tests for advanced GABC features ported from the TypeScript suite.

use gregorio_lsp::parser::types::*;
use gregorio_lsp::parser::GabcParser;

fn parse(text: &str) -> ParsedDocument {
    GabcParser::new(text).parse()
}

#[test]
fn parses_oriscus_lowercase_o() {
    let doc = parse("name: T;\n%%\n(c4) test(fo)");
    let group = &doc.notation.syllables[1].notes[0];
    assert_eq!(group.notes[0].shape, NoteShape::Oriscus);
    assert!(!group.notes.iter().any(|n| n
        .modifiers
        .iter()
        .any(|m| m.kind == ModifierType::OriscusScapus)));
}

#[test]
fn parses_oriscus_scapus_uppercase_o() {
    let doc = parse("name: T;\n%%\n(c4) test(fO)");
    let group = &doc.notation.syllables[1].notes[0];
    assert_eq!(group.notes[0].shape, NoteShape::Oriscus);
    assert!(group.notes[0]
        .modifiers
        .iter()
        .any(|m| m.kind == ModifierType::OriscusScapus));
}

#[test]
fn parses_oriscus_with_orientation_digit() {
    // o1 / o0 just consume the trailing digit
    let doc = parse("name: T;\n%%\n(c4) test(fo1)");
    let group = &doc.notation.syllables[1].notes[0];
    assert_eq!(group.notes[0].shape, NoteShape::Oriscus);
}

#[test]
fn parses_punctum_inclinatum_leaning_indicator() {
    // Uppercase pitch followed by digit 0-2 is a leaning indicator.
    let doc = parse("name: T;\n%%\n(c4) test(G1F0E2)");
    let group = &doc.notation.syllables[1].notes[0];
    assert_eq!(group.notes.len(), 3);
    assert!(group
        .notes
        .iter()
        .all(|n| n.shape == NoteShape::PunctumInclinatum));
}

#[test]
fn parses_virga_lowercase_v() {
    let doc = parse("name: T;\n%%\n(c4) test(fv)");
    let group = &doc.notation.syllables[1].notes[0];
    assert_eq!(group.notes[0].shape, NoteShape::Virga);
}

#[test]
fn parses_virga_reversa_uppercase_v() {
    let doc = parse("name: T;\n%%\n(c4) test(fV)");
    let group = &doc.notation.syllables[1].notes[0];
    assert_eq!(group.notes[0].shape, NoteShape::VirgaReversa);
}

#[test]
fn parses_stropha() {
    let doc = parse("name: T;\n%%\n(c4) test(fs)");
    let group = &doc.notation.syllables[1].notes[0];
    assert_eq!(group.notes[0].shape, NoteShape::Stropha);
}

#[test]
fn parses_quadratum_modifier() {
    let doc = parse("name: T;\n%%\n(c4) test(fqg)");
    let group = &doc.notation.syllables[1].notes[0];
    assert!(group.notes[0]
        .modifiers
        .iter()
        .any(|m| m.kind == ModifierType::Quadratum));
}

#[test]
fn parses_flat_accidental() {
    let doc = parse("name: T;\n%%\n(c4) test(fx)");
    let group = &doc.notation.syllables[1].notes[0];
    assert_eq!(group.notes[0].shape, NoteShape::Flat);
}

#[test]
fn parses_bar_double() {
    let doc = parse("name: T;\n%%\n(c4) Te(f) (::)");
    let bars: Vec<_> = doc
        .notation
        .syllables
        .iter()
        .filter_map(|s| s.bar.as_ref())
        .collect();
    assert!(bars
        .iter()
        .any(|b| matches!(b.kind, BarType::DivisioFinalis)));
}

#[test]
fn parses_styled_text_is_stripped_for_text() {
    let doc = parse("name: T;\n%%\n(c4) <i>Te</i>(f)");
    let syl = &doc.notation.syllables[1];
    assert_eq!(syl.text, "Te");
    assert!(syl
        .text_with_styles
        .as_deref()
        .map(|s| s.contains("<i>"))
        .unwrap_or(false));
}

#[test]
fn collects_comments() {
    let doc = parse("name: T;\n%%\n% a comment\n(c4) Te(f)");
    assert!(!doc.comments.is_empty());
}

#[test]
fn handles_empty_document() {
    let doc = parse("");
    assert!(doc.notation.syllables.is_empty());
    assert!(doc.headers.is_empty());
}

#[test]
fn handles_headers_without_body() {
    let doc = parse("name: T;\nmode: 1;\n%%\n");
    assert_eq!(doc.headers.get("name"), Some("T"));
    assert!(doc.notation.syllables.is_empty());
}
