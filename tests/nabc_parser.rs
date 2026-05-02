//! NABC parser tests (port of the TypeScript Jest suite).

use gregorio_lsp::parser::nabc::*;
use gregorio_lsp::parser::types::*;

#[test]
fn parses_basic_glyph_codes() {
    for code in ["vi", "pu", "cl", "pe", "po", "to", "ci", "sc", "or", "sa", "ql"] {
        let d = parse_nabc_snippet(code, None).expect("descriptor");
        assert_eq!(d.basic_glyph.code(), code);
    }
}

#[test]
fn parses_glyph_with_pitch() {
    let d = parse_nabc_snippet("vihk", None).expect("descriptor");
    assert_eq!(d.basic_glyph, NabcBasicGlyph::Virga);
    assert_eq!(d.pitch, Some('k'));
}

#[test]
fn parses_glyph_with_modifiers() {
    let d = parse_nabc_snippet("viS", None).expect("descriptor");
    let mods = d.modifiers.as_ref().expect("modifiers");
    assert!(mods.contains(&NabcGlyphModifier::MarkModification));
}

#[test]
fn rejects_conflicting_liquescence() {
    let d = parse_nabc_snippet("vi>~", None).expect("descriptor");
    let errors = validate_nabc_descriptor(&d);
    assert!(errors
        .iter()
        .any(|e| e.contains("augmentive") && e.contains("diminutive")));
}

#[test]
fn parses_subpunctis_and_prepunctis_attached() {
    let d = parse_nabc_snippet("clhgsu2pp1", None).expect("descriptor");
    assert_eq!(d.basic_glyph, NabcBasicGlyph::Clivis);
    assert_eq!(d.subpunctis.as_ref().unwrap().count, 2);
    assert_eq!(d.prepunctis.as_ref().unwrap().count, 1);
}

#[test]
fn parses_significant_letters() {
    let d = parse_nabc_snippet("vihklsc2", None).expect("descriptor");
    let letters = d.significant_letters.expect("letters");
    assert_eq!(letters.len(), 1);
    assert_eq!(letters[0].kind, NabcLetterKind::Significant);
    assert_eq!(letters[0].code, "c");
    assert_eq!(letters[0].position, 2);
}

#[test]
fn parses_fusion_chain() {
    let d = parse_nabc_snippet("vihk!tahk", None).expect("descriptor");
    assert_eq!(d.basic_glyph, NabcBasicGlyph::Virga);
    let fusion = d.fusion.as_ref().expect("fusion");
    assert_eq!(fusion.basic_glyph, NabcBasicGlyph::Tractulus);
    assert_eq!(fusion.pitch, Some('k'));
}

#[test]
fn parses_multiple_descriptors_in_snippet() {
    let descs = parse_nabc_descriptors("vihkclhf", None);
    assert_eq!(descs.len(), 2);
    assert_eq!(descs[0].basic_glyph, NabcBasicGlyph::Virga);
    assert_eq!(descs[1].basic_glyph, NabcBasicGlyph::Clivis);
}

#[test]
fn validates_glyph_codes() {
    assert!(is_valid_nabc_glyph("vi"));
    assert!(is_valid_nabc_glyph("cl"));
    assert!(!is_valid_nabc_glyph("zz"));
}
