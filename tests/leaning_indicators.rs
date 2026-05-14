//! Leaning indicator parsing for punctum inclinatum (digit 0-2 after uppercase pitch).

use gregorio_lsp::parser::types::*;
use gregorio_lsp::parser::GabcParser;

fn first_group(text: &str) -> NoteGroup {
    GabcParser::new(text).parse().notation.syllables[1].notes[0].clone()
}

#[test]
fn no_leaning_indicator_default() {
    let g = first_group("name: T;\n%%\n(c4) test(G)");
    assert_eq!(g.notes.len(), 1);
    assert_eq!(g.notes[0].shape, NoteShape::PunctumInclinatum);
}

#[test]
fn unforced_indicator_zero() {
    let g = first_group("name: T;\n%%\n(c4) test(G0)");
    assert_eq!(g.notes.len(), 1);
    assert_eq!(g.notes[0].pitch, 'g');
}

#[test]
fn ascending_indicator_one() {
    let g = first_group("name: T;\n%%\n(c4) test(G1)");
    assert_eq!(g.notes.len(), 1);
    assert_eq!(g.notes[0].pitch, 'g');
}

#[test]
fn descending_indicator_two() {
    let g = first_group("name: T;\n%%\n(c4) test(G2)");
    assert_eq!(g.notes.len(), 1);
    assert_eq!(g.notes[0].pitch, 'g');
}

#[test]
fn mixed_leaning_in_climacus() {
    // GFE with mixed indicators: G1 ascending, F0 unforced, E2 descending
    let g = first_group("name: T;\n%%\n(c4) test(G1F0E2)");
    assert_eq!(g.notes.len(), 3);
    assert!(g
        .notes
        .iter()
        .all(|n| n.shape == NoteShape::PunctumInclinatum));
    assert_eq!(g.notes[0].pitch, 'g');
    assert_eq!(g.notes[1].pitch, 'f');
    assert_eq!(g.notes[2].pitch, 'e');
}

#[test]
fn leaning_indicators_dont_apply_to_lowercase() {
    // Lowercase pitches don't accept leaning indicators; digit isn't consumed as such.
    let g = first_group("name: T;\n%%\n(c4) test(g)");
    assert_eq!(g.notes[0].shape, NoteShape::Punctum);
}
