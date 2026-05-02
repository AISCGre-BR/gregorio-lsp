//! Oriscus orientation digits: o0/o1 (lowercase) and O0/O1 (uppercase, scapus).

use gregorio_lsp::parser::types::*;
use gregorio_lsp::parser::GabcParser;

fn group(text: &str) -> NoteGroup {
    GabcParser::new(text).parse().notation.syllables[1].notes[0].clone()
}

#[test]
fn oriscus_default() {
    let g = group("name: T;\n%%\n(c4) test(fo)");
    assert_eq!(g.notes[0].shape, NoteShape::Oriscus);
    assert!(!g.notes[0]
        .modifiers
        .iter()
        .any(|m| m.kind == ModifierType::OriscusScapus));
}

#[test]
fn oriscus_orientation_zero() {
    let g = group("name: T;\n%%\n(c4) test(fo0)");
    assert_eq!(g.notes[0].shape, NoteShape::Oriscus);
}

#[test]
fn oriscus_orientation_one() {
    let g = group("name: T;\n%%\n(c4) test(fo1)");
    assert_eq!(g.notes[0].shape, NoteShape::Oriscus);
}

#[test]
fn oriscus_scapus_default() {
    let g = group("name: T;\n%%\n(c4) test(fO)");
    assert_eq!(g.notes[0].shape, NoteShape::Oriscus);
    assert!(g.notes[0]
        .modifiers
        .iter()
        .any(|m| m.kind == ModifierType::OriscusScapus));
}

#[test]
fn oriscus_scapus_orientation_zero() {
    let g = group("name: T;\n%%\n(c4) test(fO0)");
    assert!(g.notes[0]
        .modifiers
        .iter()
        .any(|m| m.kind == ModifierType::OriscusScapus));
}

#[test]
fn oriscus_scapus_orientation_one() {
    let g = group("name: T;\n%%\n(c4) test(fO1)");
    assert!(g.notes[0]
        .modifiers
        .iter()
        .any(|m| m.kind == ModifierType::OriscusScapus));
}
