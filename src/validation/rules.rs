//! Validation rules: structural / lint-style checks over a parsed GABC document.

use crate::parser::nabc::parse_nabc_snippets;
use crate::parser::types::*;

/// A validation rule that produces zero or more errors when applied to a parsed document.
pub struct ValidationRule {
    pub name: &'static str,
    pub severity: Severity,
    pub validate: fn(&ParsedDocument) -> Vec<ParseError>,
}

// ---------- Helpers ----------

fn pitch_value(p: char) -> i32 {
    match p.to_ascii_lowercase() {
        'a' => 1,
        'b' => 2,
        'c' => 3,
        'd' => 4,
        'e' => 5,
        'f' => 6,
        'g' => 7,
        'h' => 8,
        'i' => 9,
        'j' => 10,
        'k' => 11,
        'l' => 12,
        'm' => 13,
        'n' => 14,
        'p' => 15,
        _ => 0,
    }
}

fn has_modifier(note: &Note, kind: ModifierType) -> bool {
    note.modifiers.iter().any(|m| m.kind == kind)
}

// ---------- Rules ----------

fn name_header(doc: &ParsedDocument) -> Vec<ParseError> {
    let missing = match doc.headers.get("name") {
        None => true,
        Some(v) => v.trim().is_empty(),
    };
    if missing {
        vec![ParseError::new(
            "no name specified, put 'name:...;' at the beginning of the file, can be dangerous with some output formats",
            Range::zero(),
            Severity::Warning,
        )]
    } else {
        Vec::new()
    }
}

fn duplicate_headers(_doc: &ParsedDocument) -> Vec<ParseError> {
    Vec::new() // Parser collapses duplicates (mirrors TS behavior).
}

fn first_syllable_line_break(doc: &ParsedDocument) -> Vec<ParseError> {
    if let Some(first) = doc.notation.syllables.first() {
        if first.line_break.is_some() {
            return vec![ParseError::new(
                "line break is not supported on the first syllable",
                first.range,
                Severity::Error,
            )];
        }
    }
    Vec::new()
}

fn first_syllable_clef_change(doc: &ParsedDocument) -> Vec<ParseError> {
    let s = &doc.notation.syllables;
    if s.len() > 1 {
        let first = &s[0];
        let second = &s[1];
        if second.clef.is_some() && first.clef.is_none() {
            return vec![ParseError::new(
                "clef change is not supported on the first syllable",
                second.range,
                Severity::Error,
            )];
        }
    }
    Vec::new()
}

fn nabc_without_header(doc: &ParsedDocument) -> Vec<ParseError> {
    let has_header = doc.headers.has("nabc-lines");
    let has_nabc = doc
        .notation
        .syllables
        .iter()
        .any(|s| s.notes.iter().any(|n| n.nabc.as_ref().is_some_and(|v| !v.is_empty())));
    if has_nabc && !has_header {
        vec![ParseError::new(
            "pipe '|' in note group without `nabc-lines` header",
            Range::zero(),
            Severity::Error,
        )]
    } else {
        Vec::new()
    }
}

fn quilisma_followed_by_lower(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        for note_group in &syllable.notes {
            for window in note_group.notes.windows(2) {
                let cur = &window[0];
                let next = &window[1];
                if cur.shape == NoteShape::Quilisma
                    && pitch_value(next.pitch) <= pitch_value(cur.pitch)
                {
                    out.push(ParseError::new(
                        "Quilisma followed by equal or lower pitch note may cause rendering issues",
                        cur.range,
                        Severity::Warning,
                    ));
                }
            }
        }
    }
    out
}

fn quilisma_pes_preceded_by_higher(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    let syllables = &doc.notation.syllables;
    for window in syllables.windows(2) {
        let s = &window[0];
        let next = &window[1];
        let last_group = match s.notes.last() {
            Some(g) => g,
            None => continue,
        };
        let next_group = match next.notes.first() {
            Some(g) => g,
            None => continue,
        };
        if last_group.notes.is_empty() || next_group.notes.len() < 2 {
            continue;
        }
        let last_note = last_group.notes.last().unwrap();
        let first_next = &next_group.notes[0];
        if first_next.shape == NoteShape::Quilisma
            && pitch_value(last_note.pitch) >= pitch_value(first_next.pitch)
        {
            out.push(ParseError::new(
                "Quilisma-pes preceded by equal or higher pitch note may cause rendering issues",
                first_next.range,
                Severity::Warning,
            ));
        }
    }
    out
}

fn virga_strata_followed_by_higher(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        for note_group in &syllable.notes {
            for window in note_group.notes.windows(2) {
                let cur = &window[0];
                let next = &window[1];
                if cur.shape == NoteShape::Virga
                    && has_modifier(cur, ModifierType::Strata)
                    && pitch_value(next.pitch) >= pitch_value(cur.pitch)
                {
                    out.push(ParseError::new(
                        "Virga strata followed by equal or higher pitch note may cause rendering issues",
                        cur.range,
                        Severity::Warning,
                    ));
                }
            }
        }
    }
    out
}

fn staff_lines(doc: &ParsedDocument) -> Vec<ParseError> {
    if let Some(value) = doc.headers.get("staff-lines") {
        let n: i32 = value.trim().parse().unwrap_or(4);
        if !(2..=5).contains(&n) {
            return vec![ParseError::new(
                "invalid number of staff lines (must be between 2 and 5)",
                Range::zero(),
                Severity::Error,
            )];
        }
    }
    Vec::new()
}

fn collect_fusion_chain(glyph: &NabcGlyphDescriptor) -> Vec<&NabcGlyphDescriptor> {
    let mut chain = Vec::new();
    let mut cur: Option<&NabcGlyphDescriptor> = Some(glyph);
    while let Some(c) = cur {
        chain.push(c);
        cur = c.fusion.as_deref();
    }
    chain
}

fn balanced_pitch_descriptors_in_fused_glyphs(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        for note_group in &syllable.notes {
            let parsed_owned: Option<Vec<NabcGlyphDescriptor>> = match (&note_group.nabc_parsed, &note_group.nabc) {
                (Some(p), _) if !p.is_empty() => Some(p.clone()),
                (_, Some(raw)) if !raw.is_empty() => {
                    Some(parse_nabc_snippets(raw, Some(note_group.range.start)))
                }
                _ => None,
            };
            let Some(parsed) = parsed_owned else { continue };
            for glyph in &parsed {
                let chain = collect_fusion_chain(glyph);
                if chain.len() < 2 {
                    continue;
                }
                let pitches: Vec<bool> = chain.iter().map(|g| g.pitch.is_some()).collect();
                let all_false = pitches.iter().all(|p| !*p);
                let all_true = pitches.iter().all(|p| *p);
                let only_last = pitches[..pitches.len() - 1].iter().all(|p| !*p)
                    && *pitches.last().unwrap();
                let balanced = all_false || all_true || only_last;
                if !balanced {
                    let range = glyph.range.unwrap_or(note_group.range);
                    out.push(ParseError::new(
                        "Unbalanced pitch descriptors in fused glyphs are not supported in Gregorio 6.1.0. Both glyphs must have pitch descriptors (e.g., 'vihk!tahk') or neither should have them.",
                        range,
                        Severity::Warning,
                    ));
                    break;
                }
            }
        }
    }
    out
}

fn modifiers_in_fused_glyphs(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    let modifier_chars = ['S', 'G', 'M', '-', '>', '~'];
    for syllable in &doc.notation.syllables {
        for note in &syllable.notes {
            let Some(nabc) = note.nabc.as_ref() else { continue };
            for line in nabc {
                if !line.contains('!') {
                    continue;
                }
                let parts: Vec<&str> = line.split('!').collect();
                if parts.len() < 2 {
                    continue;
                }
                let last = parts[parts.len() - 1];
                for part in &parts[..parts.len() - 1] {
                    if part.chars().any(|c| modifier_chars.contains(&c)) {
                        out.push(ParseError::new(
                            format!(
                                "Modifiers in fused glyphs are only allowed on the last glyph descriptor (Gregorio 6.1.0). Found modifier in '{part}' but only '{last}' (the last glyph) can have modifiers."
                            ),
                            note.range,
                            Severity::Warning,
                        ));
                        break;
                    }
                }
            }
        }
    }
    out
}

fn reconstruct_note_group_gabc(ng: &NoteGroup) -> String {
    let mut s = String::from("(");
    s.push_str(&ng.gabc);
    if let Some(nabc) = &ng.nabc {
        for line in nabc {
            s.push('|');
            s.push_str(line);
        }
    }
    s.push(')');
    s
}

fn multi_word_syllable(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        let words: Vec<&str> = syllable.text.split_whitespace().collect();
        if words.len() < 2 {
            continue;
        }
        let last_idx = words.len() - 1;
        let last_ng = syllable.notes.last().map(reconstruct_note_group_gabc);
        let last_group_str = last_ng.as_deref().unwrap_or("()");
        let mut fix_text = String::new();
        for (i, word) in words.iter().enumerate() {
            if i > 0 {
                fix_text.push(' ');
            }
            fix_text.push_str(word);
            if i == last_idx {
                fix_text.push_str(last_group_str);
            } else {
                fix_text.push_str("()");
            }
        }
        out.push(
            ParseError::new(
                format!(
                    "Multiple space-separated words '{}' share a single note group; \
                     split into individual note groups: '{fix_text}'",
                    syllable.text,
                ),
                syllable.range,
                Severity::Warning,
            )
            .with_code("multi-word-syllable")
            .with_fix(TextFix {
                range: syllable.range,
                new_text: fix_text,
            }),
        );
    }
    out
}

// ---------- Public API ----------

pub const VALIDATE_NAME_HEADER: ValidationRule = ValidationRule {
    name: "name-header",
    severity: Severity::Warning,
    validate: name_header,
};
pub const VALIDATE_DUPLICATE_HEADERS: ValidationRule = ValidationRule {
    name: "duplicate-headers",
    severity: Severity::Warning,
    validate: duplicate_headers,
};
pub const VALIDATE_FIRST_SYLLABLE_LINE_BREAK: ValidationRule = ValidationRule {
    name: "first-syllable-line-break",
    severity: Severity::Error,
    validate: first_syllable_line_break,
};
pub const VALIDATE_FIRST_SYLLABLE_CLEF_CHANGE: ValidationRule = ValidationRule {
    name: "first-syllable-clef-change",
    severity: Severity::Error,
    validate: first_syllable_clef_change,
};
pub const VALIDATE_NABC_WITHOUT_HEADER: ValidationRule = ValidationRule {
    name: "nabc-without-header",
    severity: Severity::Error,
    validate: nabc_without_header,
};
pub const VALIDATE_QUILISMA_LOWER: ValidationRule = ValidationRule {
    name: "quilisma-lower-pitch",
    severity: Severity::Warning,
    validate: quilisma_followed_by_lower,
};
pub const VALIDATE_QUILISMA_PES_HIGHER: ValidationRule = ValidationRule {
    name: "quilisma-pes-higher-pitch",
    severity: Severity::Warning,
    validate: quilisma_pes_preceded_by_higher,
};
pub const VALIDATE_VIRGA_STRATA_HIGHER: ValidationRule = ValidationRule {
    name: "virga-strata-higher-pitch",
    severity: Severity::Warning,
    validate: virga_strata_followed_by_higher,
};
pub const VALIDATE_STAFF_LINES: ValidationRule = ValidationRule {
    name: "staff-lines",
    severity: Severity::Error,
    validate: staff_lines,
};
pub const VALIDATE_BALANCED_PITCH_DESCRIPTORS_FUSED: ValidationRule = ValidationRule {
    name: "balanced-pitch-descriptors-fused-glyphs",
    severity: Severity::Warning,
    validate: balanced_pitch_descriptors_in_fused_glyphs,
};
pub const VALIDATE_MODIFIERS_FUSED: ValidationRule = ValidationRule {
    name: "modifiers-in-fused-glyphs",
    severity: Severity::Warning,
    validate: modifiers_in_fused_glyphs,
};
pub const VALIDATE_MULTI_WORD_SYLLABLE: ValidationRule = ValidationRule {
    name: "multi-word-syllable",
    severity: Severity::Warning,
    validate: multi_word_syllable,
};

pub fn all_validation_rules() -> Vec<&'static ValidationRule> {
    vec![
        &VALIDATE_NAME_HEADER,
        &VALIDATE_FIRST_SYLLABLE_LINE_BREAK,
        &VALIDATE_FIRST_SYLLABLE_CLEF_CHANGE,
        &VALIDATE_NABC_WITHOUT_HEADER,
        &VALIDATE_QUILISMA_LOWER,
        &VALIDATE_QUILISMA_PES_HIGHER,
        &VALIDATE_VIRGA_STRATA_HIGHER,
        &VALIDATE_STAFF_LINES,
        &VALIDATE_BALANCED_PITCH_DESCRIPTORS_FUSED,
        &VALIDATE_MODIFIERS_FUSED,
        &VALIDATE_MULTI_WORD_SYLLABLE,
    ]
}
