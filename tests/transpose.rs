use gregorio_lsp::transpose::{
    fill_empty_groups, is_gabc_pitch, parse_nabc_lines, shift_notes, shift_pitch, ShiftDirection,
};

// ---------------------------------------------------------------------------
// is_gabc_pitch
// ---------------------------------------------------------------------------

#[test]
fn test_is_gabc_pitch_all_lowercase() {
    for c in "abcdefghijklmnp".chars() {
        assert!(is_gabc_pitch(c), "Expected '{c}' to be a pitch");
    }
}

#[test]
fn test_is_gabc_pitch_all_uppercase() {
    for c in "ABCDEFGHIJKLMNP".chars() {
        assert!(is_gabc_pitch(c), "Expected '{c}' to be a pitch");
    }
}

#[test]
fn test_is_gabc_pitch_non_pitches() {
    for c in "oOqQrRsSwWxXyYzZ0123456789!.,;:|+%-/ ".chars() {
        assert!(!is_gabc_pitch(c), "Expected '{c}' NOT to be a pitch");
    }
}

// ---------------------------------------------------------------------------
// shift_pitch — individual characters
// ---------------------------------------------------------------------------

#[test]
fn test_shift_pitch_up_full_cycle() {
    let cycle: Vec<char> = "abcdefghijklmnp".chars().collect();
    for i in 0..cycle.len() {
        let expected = cycle[(i + 1) % cycle.len()];
        assert_eq!(
            shift_pitch(cycle[i], ShiftDirection::Up),
            expected,
            "shift_pitch('{}', Up) should be '{}'",
            cycle[i],
            expected
        );
    }
}

#[test]
fn test_shift_pitch_down_full_cycle() {
    let cycle: Vec<char> = "abcdefghijklmnp".chars().collect();
    for i in 0..cycle.len() {
        let expected = cycle[(i + cycle.len() - 1) % cycle.len()];
        assert_eq!(
            shift_pitch(cycle[i], ShiftDirection::Down),
            expected,
            "shift_pitch('{}', Down) should be '{}'",
            cycle[i],
            expected
        );
    }
}

#[test]
fn test_shift_pitch_up_uppercase_cycle() {
    let cycle: Vec<char> = "ABCDEFGHIJKLMNP".chars().collect();
    for i in 0..cycle.len() {
        let expected = cycle[(i + 1) % cycle.len()];
        assert_eq!(shift_pitch(cycle[i], ShiftDirection::Up), expected);
    }
}

#[test]
fn test_shift_pitch_down_uppercase_cycle() {
    let cycle: Vec<char> = "ABCDEFGHIJKLMNP".chars().collect();
    for i in 0..cycle.len() {
        let expected = cycle[(i + cycle.len() - 1) % cycle.len()];
        assert_eq!(shift_pitch(cycle[i], ShiftDirection::Down), expected);
    }
}

#[test]
fn test_shift_pitch_up_wraps_p_to_a() {
    assert_eq!(shift_pitch('p', ShiftDirection::Up), 'a');
    assert_eq!(shift_pitch('P', ShiftDirection::Up), 'A');
}

#[test]
fn test_shift_pitch_down_wraps_a_to_p() {
    assert_eq!(shift_pitch('a', ShiftDirection::Down), 'p');
    assert_eq!(shift_pitch('A', ShiftDirection::Down), 'P');
}

#[test]
fn test_shift_pitch_n_and_p() {
    // n → p (up), p → n (down)
    assert_eq!(shift_pitch('n', ShiftDirection::Up), 'p');
    assert_eq!(shift_pitch('p', ShiftDirection::Down), 'n');
}

#[test]
fn test_shift_pitch_non_pitch_unchanged() {
    for c in "oOzZ0!|".chars() {
        assert_eq!(shift_pitch(c, ShiftDirection::Up), c);
        assert_eq!(shift_pitch(c, ShiftDirection::Down), c);
    }
}

// ---------------------------------------------------------------------------
// parse_nabc_lines
// ---------------------------------------------------------------------------

#[test]
fn test_parse_nabc_lines_present() {
    let text = "name: Kyrie;\nnabc-lines: 2;\n%%\n";
    assert_eq!(parse_nabc_lines(text), 2);
}

#[test]
fn test_parse_nabc_lines_missing() {
    let text = "name: Kyrie;\n%%\n";
    assert_eq!(parse_nabc_lines(text), 0);
}

#[test]
fn test_parse_nabc_lines_case_insensitive() {
    let text = "NABC-Lines: 3;\n%%\n";
    assert_eq!(parse_nabc_lines(text), 3);
}

#[test]
fn test_parse_nabc_lines_zero() {
    let text = "nabc-lines: 0;\n%%\n";
    assert_eq!(parse_nabc_lines(text), 0);
}

// ---------------------------------------------------------------------------
// shift_notes — structure preservation
// ---------------------------------------------------------------------------

#[test]
fn test_shift_notes_preserves_headers() {
    let text = "name: Kyrie;\nmode: 1;\n%%\n(c4) ge(f)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert!(result.starts_with("name: Kyrie;\nmode: 1;\n%%\n"), "Headers changed: {result}");
}

#[test]
fn test_shift_notes_preserves_clef_c4() {
    let text = "%%\n(c4)(fg)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert!(result.contains("(c4)"), "Clef changed: {result}");
    assert!(result.contains("(gh)"), "Notes not shifted: {result}");
}

#[test]
fn test_shift_notes_preserves_clef_f3() {
    let text = "%%\n(f3)(fg)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert!(result.contains("(f3)"), "Clef changed: {result}");
    assert!(result.contains("(gh)"), "Notes not shifted: {result}");
}

#[test]
fn test_shift_notes_preserves_clef_flat_cb3() {
    let text = "%%\n(cb3)(fg)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert!(result.contains("(cb3)"), "Flat clef changed: {result}");
    assert!(result.contains("(gh)"), "Notes not shifted: {result}");
}

#[test]
fn test_shift_notes_note_c_alone_is_not_clef() {
    // (c) has no digit after — it is pitch 'c', not a clef.
    let text = "%%\n(c)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert_eq!(result, "%%\n(d)\n", "Got: {result}");
}

#[test]
fn test_shift_notes_note_f_alone_is_not_clef() {
    // (fg) — 'f' not followed by digit, so not a clef.
    let text = "%%\n(fg)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert_eq!(result, "%%\n(gh)\n", "Got: {result}");
}

#[test]
fn test_shift_notes_preserves_lyric_text() {
    // Lyric letters outside () must not be changed.
    let text = "%%\nKy(fg)ri(h)e\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert!(result.contains("Ky("), "Lyric 'Ky' changed: {result}");
    assert!(result.contains(")ri("), "Lyric 'ri' changed: {result}");
    assert!(result.contains(")e"), "Lyric 'e' changed: {result}");
    assert!(result.contains("(gh)"), "Note not shifted: {result}");
    assert!(result.contains("(i)"), "Note not shifted: {result}");
}

#[test]
fn test_shift_notes_preserves_nabc_simple() {
    // With no nabc-lines declared, content after '|' is NABC and must not shift.
    let text = "%%\na(fg|nabc)b(h)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert!(result.contains("|nabc)"), "NABC segment changed: {result}");
    assert!(result.contains("(gh|nabc)"), "GABC notes not shifted: {result}");
    assert!(result.contains("(i)"), "Note not shifted: {result}");
}

// ---------------------------------------------------------------------------
// shift_notes — multi-NABC
// ---------------------------------------------------------------------------

#[test]
fn test_shift_notes_multi_nabc_2lines() {
    // nabc-lines: 2 → period = 3 → segments 0,3 are GABC; 1,2,4,5 are NABC.
    let text = "nabc-lines: 2;\n%%\nfoo(fgh|pu|ta|ij|vi|pe)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert!(
        result.contains("(ghi|pu|ta|jk|vi|pe)"),
        "Multi-NABC (2 lines) shift wrong: {result}"
    );
}

#[test]
fn test_shift_notes_multi_nabc_1line_multiple_segments() {
    // nabc-lines: 1 → period = 2 → segments 0,2 are GABC; 1,3 are NABC.
    let text = "nabc-lines: 1;\n%%\nfoo(fg|pu|hi|na)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert!(
        result.contains("(gh|pu|ij|na)"),
        "Multi-NABC (1 line) shift wrong: {result}"
    );
}

#[test]
fn test_shift_notes_nabc_letters_not_shifted() {
    // Letters in NABC segments that happen to be pitch letters must not shift.
    let text = "nabc-lines: 2;\n%%\nfoo(fgh|abc|def|ij|ghi|jkl)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    // GABC segments: fgh→ghi, ij→jk; NABC unchanged.
    assert!(
        result.contains("(ghi|abc|def|jk|ghi|jkl)"),
        "NABC letters were incorrectly shifted: {result}"
    );
}

// ---------------------------------------------------------------------------
// shift_notes — pitch wrapping
// ---------------------------------------------------------------------------

#[test]
fn test_shift_notes_wrap_up_p_to_a() {
    let text = "%%\n(p)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert_eq!(result, "%%\n(a)\n", "Got: {result}");
}

#[test]
fn test_shift_notes_wrap_down_a_to_p() {
    let text = "%%\n(a)\n";
    let result = shift_notes(text, ShiftDirection::Down, None);
    assert_eq!(result, "%%\n(p)\n", "Got: {result}");
}

#[test]
fn test_shift_notes_n_up_to_p() {
    let text = "%%\n(n)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert_eq!(result, "%%\n(p)\n", "Got: {result}");
}

#[test]
fn test_shift_notes_uppercase_pitches() {
    let text = "%%\n(G)(H)(P)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert_eq!(result, "%%\n(H)(I)(A)\n", "Got: {result}");
}

// ---------------------------------------------------------------------------
// shift_notes — note modifiers (must pass through unchanged)
// ---------------------------------------------------------------------------

#[test]
fn test_shift_notes_modifiers_pass_through() {
    // 'o' (oriscus), '.' (mora), ',' (quilisma separator), etc. are not pitches.
    let text = "%%\n(go.)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert_eq!(result, "%%\n(ho.)\n", "Got: {result}");
}

#[test]
fn test_shift_notes_custos_pitch_shifted() {
    // '+' introduces an explicit custos; the pitch letter after '+' is shifted.
    let text = "%%\n(g+h)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert_eq!(result, "%%\n(h+i)\n", "Got: {result}");
}

// ---------------------------------------------------------------------------
// shift_notes — byte-range selection
// ---------------------------------------------------------------------------

#[test]
fn test_shift_notes_byte_range_first_group_only() {
    // "%%\n" = 3 bytes; "(fg)" = bytes 3..7; "(hi)" = bytes 7..11
    let text = "%%\n(fg)(hi)\n";
    let result = shift_notes(text, ShiftDirection::Up, Some(3..7));
    assert!(result.contains("(gh)"), "First group not shifted: {result}");
    assert!(result.contains("(hi)"), "Second group incorrectly shifted: {result}");
}

#[test]
fn test_shift_notes_byte_range_second_group_only() {
    let text = "%%\n(fg)(hi)\n";
    // "(hi)" starts at byte 7
    let result = shift_notes(text, ShiftDirection::Up, Some(7..11));
    assert!(result.contains("(fg)"), "First group incorrectly shifted: {result}");
    assert!(result.contains("(ij)"), "Second group not shifted: {result}");
}

#[test]
fn test_shift_notes_no_range_shifts_all() {
    let text = "%%\n(fg)(hi)\n";
    let result = shift_notes(text, ShiftDirection::Up, None);
    assert!(result.contains("(gh)"), "First group not shifted: {result}");
    assert!(result.contains("(ij)"), "Second group not shifted: {result}");
}

// ---------------------------------------------------------------------------
// shift_notes — round-trip (up then down restores original)
// ---------------------------------------------------------------------------

#[test]
fn test_shift_notes_round_trip() {
    let text = "name: Test;\nnabc-lines: 2;\n%%\nKy(fgh|pu|ta|ij|vi|pe)ri(c4 gh)\n";
    let up = shift_notes(text, ShiftDirection::Up, None);
    let back = shift_notes(&up, ShiftDirection::Down, None);
    assert_eq!(back, text, "Round-trip failed");
}

// ---------------------------------------------------------------------------
// fill_empty_groups — basic
// ---------------------------------------------------------------------------

#[test]
fn test_fill_basic() {
    let text = "%%\n(fgh) () ()\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(result, "%%\n(fgh) (h) (h)\n", "Got: {result}");
}

#[test]
fn test_fill_multiple_seeds() {
    // Each non-empty group sets a new seed; subsequent empty groups use it.
    let text = "%%\n(fg) () (hi) () ()\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(result, "%%\n(fg) (g) (hi) (i) (i)\n", "Got: {result}");
}

#[test]
fn test_fill_no_preceding_pitch_unchanged() {
    // Empty groups before any note group are left unchanged.
    let text = "%%\n() () (fg)\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(result, "%%\n() () (fg)\n", "Got: {result}");
}

#[test]
fn test_fill_whitespace_only_empty() {
    // ( ) (with a space) counts as empty.
    let text = "%%\n(fgh) ( ) ( )\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(result, "%%\n(fgh) (h) (h)\n", "Got: {result}");
}

#[test]
fn test_fill_no_empty_groups_unchanged() {
    let text = "%%\n(fg)(hi)\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(result, text, "Got: {result}");
}

// ---------------------------------------------------------------------------
// fill_empty_groups — structure preservation
// ---------------------------------------------------------------------------

#[test]
fn test_fill_preserves_headers() {
    let text = "name: Kyrie;\nmode: 1;\n%%\n(fg) ()\n";
    let result = fill_empty_groups(text, None);
    assert!(result.starts_with("name: Kyrie;\nmode: 1;\n%%\n"), "Headers changed: {result}");
}

#[test]
fn test_fill_clef_not_a_seed() {
    // Clef group (c4) must not set last_pitch. The (fg) after it does.
    let text = "%%\n(c4) (fg) () ()\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(result, "%%\n(c4) (fg) (g) (g)\n", "Got: {result}");
}

#[test]
fn test_fill_clef_between_groups() {
    // A clef change between a note group and an empty group must not reset the
    // last_pitch.
    let text = "%%\n(fgh) (c4) () ()\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(result, "%%\n(fgh) (c4) (h) (h)\n", "Got: {result}");
}

#[test]
fn test_fill_uppercase_pitch_used_as_seed() {
    // PunctumInclinatum (uppercase) letters are valid seeds.
    let text = "%%\n(FGH) ()\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(result, "%%\n(FGH) (H)\n", "Got: {result}");
}

// ---------------------------------------------------------------------------
// fill_empty_groups — multi-NABC
// ---------------------------------------------------------------------------

#[test]
fn test_fill_multi_nabc_seed() {
    // nabc-lines: 2 → period = 3 → only segments 0,3,… are GABC.
    // Seed from (fgh|pu|ta) is 'h' (last GABC pitch).
    // After (ij|vi|pe) seed updates to 'j'.
    let text = "nabc-lines: 2;\n%%\n(fgh|pu|ta) () (ij|vi|pe) ()\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(
        result,
        "nabc-lines: 2;\n%%\n(fgh|pu|ta) (h) (ij|vi|pe) (j)\n",
        "Got: {result}"
    );
}

#[test]
fn test_fill_nabc_content_not_used_as_seed() {
    // With no nabc-lines, content after '|' is NABC.  Pitch letters in NABC
    // segments must not be used as the seed.
    let text = "%%\na(fg|xyz) () (hi)\n";
    let result = fill_empty_groups(text, None);
    // Seed from (fg|xyz) should be 'g' (GABC only), not 'z'.
    assert_eq!(result, "%%\na(fg|xyz) (g) (hi)\n", "Got: {result}");
}

// ---------------------------------------------------------------------------
// fill_empty_groups — byte-range selection
// ---------------------------------------------------------------------------

#[test]
fn test_fill_byte_range_fills_only_in_range() {
    // "%%\n" = 3 bytes
    // "(fg)" = bytes 3..7   → non-empty, updates last_pitch to 'g'
    // "()"   = bytes 7..9   → empty, NOT in range 9..11 → unchanged
    // "()"   = bytes 9..11  → empty, in range → filled with 'g'
    let text = "%%\n(fg)()()\n";
    let result = fill_empty_groups(text, Some(9..11));
    assert_eq!(result, "%%\n(fg)()(g)\n", "Got: {result}");
}

#[test]
fn test_fill_byte_range_non_empty_outside_range_updates_seed() {
    // Non-empty groups outside the range still update last_pitch so that
    // in-range empty groups receive the correct seed.
    // "%%\n" = 3 bytes; "(fgh)" = bytes 3..8; "()" = bytes 8..10 (in range)
    let text = "%%\n(fgh)()(hi)\n";
    let result = fill_empty_groups(text, Some(8..10));
    assert_eq!(result, "%%\n(fgh)(h)(hi)\n", "Got: {result}");
}

// ---------------------------------------------------------------------------
// fill_empty_groups — idempotency / no-op check
// ---------------------------------------------------------------------------

#[test]
fn test_fill_already_full_unchanged() {
    // A document with no empty groups must come back identical.
    let text = "name: Test;\n%%\nKy(fgh)ri(e)e\n";
    let result = fill_empty_groups(text, None);
    assert_eq!(result, text, "Got: {result}");
}
