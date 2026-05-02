//! Parity tests ported from the original TypeScript Jest suite.

use gregorio_lsp::parser::types::*;
use gregorio_lsp::parser::GabcParser;

fn parse(text: &str) -> ParsedDocument {
    GabcParser::new(text).parse()
}

#[test]
fn parses_simple_headers() {
    let text = "name: Test;\nmode: 1;\n%%\n(c4) Test(f)";
    let doc = parse(text);
    assert_eq!(doc.headers.get("name"), Some("Test"));
    assert_eq!(doc.headers.get("mode"), Some("1"));
}

#[test]
fn parses_multiline_headers() {
    let text = "name: This is a\nvery long name;;\n%%\n(c4) Test(f)";
    let doc = parse(text);
    let name = doc.headers.get("name").unwrap();
    assert!(name.contains("This is a"));
    assert!(name.contains("very long name"));
}

#[test]
fn handles_comments_in_headers() {
    let text = "name: Test; % This is a comment\nmode: 1; % Another comment\n%%\n(c4) Test(f)";
    let doc = parse(text);
    assert_eq!(doc.headers.get("name"), Some("Test"));
    assert_eq!(doc.headers.get("mode"), Some("1"));
    assert!(!doc.comments.is_empty());
}

#[test]
fn parses_simple_notation() {
    let text = "name: Test;\n%%\n(c4) Te(f)st(g)";
    let doc = parse(text);
    assert!(!doc.notation.syllables.is_empty());
}

#[test]
fn detects_clef() {
    let text = "name: Test;\n%%\n(c4) Test(f)";
    let doc = parse(text);
    let first = &doc.notation.syllables[0];
    assert!(first.clef.is_some());
    let clef = first.clef.as_ref().unwrap();
    assert!(matches!(clef.kind, ClefKind::C));
    assert_eq!(clef.line, 4);
}

#[test]
fn parses_punctum_inclinatum_uppercase() {
    let text = "name: Test;\n%%\n(c4) ad(GFE)";
    let doc = parse(text);
    let group = &doc.notation.syllables[1].notes[0];
    assert!(group.notes.iter().all(|n| n.shape == NoteShape::PunctumInclinatum));
    assert_eq!(group.notes.len(), 3);
}

#[test]
fn parses_quilisma() {
    let text = "name: Test;\n%%\n(c4) AL(fwf)";
    let doc = parse(text);
    let group = &doc.notation.syllables[1].notes[0];
    assert!(group.notes.iter().any(|n| n.shape == NoteShape::Quilisma));
}

#[test]
fn parses_note_with_modifiers() {
    let text = "name: Test;\n%%\n(c4) test(f.)";
    let doc = parse(text);
    let group = &doc.notation.syllables[1].notes[0];
    assert!(group
        .notes
        .iter()
        .any(|n| n.modifiers.iter().any(|m| m.kind == ModifierType::PunctumMora)));
}

#[test]
fn parses_nabc_when_header_present() {
    let text = "name: Test;\nnabc-lines: 1;\n%%\n(c4) test(f|vihk)";
    let doc = parse(text);
    let group = &doc.notation.syllables[1].notes[0];
    let nabc = group.nabc.as_ref().expect("nabc should be present");
    assert_eq!(nabc.len(), 1);
    assert_eq!(nabc[0], "vihk");
    let parsed = group.nabc_parsed.as_ref().unwrap();
    assert!(!parsed.is_empty());
    assert_eq!(parsed[0].basic_glyph, NabcBasicGlyph::Virga);
    assert_eq!(parsed[0].pitch, Some('k'));
}

#[test]
fn parses_custos_z0() {
    let text = "name: Test;\n%%\n(c4) test(z0)";
    let doc = parse(text);
    let group = &doc.notation.syllables[1].notes[0];
    let custos = group.custos.as_ref().expect("custos");
    assert!(matches!(custos.kind, CustosKind::Auto));
}

#[test]
fn parses_explicit_custos() {
    let text = "name: Test;\n%%\n(c4) test(+f)";
    let doc = parse(text);
    let group = &doc.notation.syllables[1].notes[0];
    let custos = group.custos.as_ref().expect("custos");
    assert!(matches!(custos.kind, CustosKind::Explicit));
    assert_eq!(custos.pitch, Some('f'));
}

#[test]
fn parses_attribute_no_value() {
    let text = "name: Test;\n%%\n(c4) test(f[nocustos])";
    let doc = parse(text);
    let group = &doc.notation.syllables[1].notes[0];
    let attrs = group.attributes.as_ref().expect("attributes");
    assert_eq!(attrs[0].name, "nocustos");
    assert!(attrs[0].value.is_none());
}

#[test]
fn parses_attribute_with_value() {
    let text = "name: Test;\n%%\n(c4) test(f[shape:stroke])";
    let doc = parse(text);
    let group = &doc.notation.syllables[1].notes[0];
    let attrs = group.attributes.as_ref().expect("attributes");
    assert_eq!(attrs[0].name, "shape");
    assert_eq!(attrs[0].value.as_deref(), Some("stroke"));
}
