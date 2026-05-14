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

fn duplicate_headers(doc: &ParsedDocument) -> Vec<ParseError> {
    // Count how many times each key was overwritten (one entry per overwrite in duplicate_keys).
    let mut overwrite_counts: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for key in &doc.headers.duplicate_keys {
        *overwrite_counts.entry(key.as_str()).or_insert(0) += 1;
    }

    // Maximum number of times a header may appear before a warning is emitted.
    // `usize::MAX` means unlimited (GregorioTeX accepts any count without warning).
    //
    // GregorioTeX source (v6.2.0):
    //   - `annotation`  → stored in `score->annotation[MAX_ANNOTATIONS]` (size 2);
    //                      third entry triggers "too many definitions of annotation" warning.
    //   - `commentary`  → is an `OTHER_HEADER` (no dedicated lexer token); all entries
    //                      are silently appended to the generic `score->headers` linked list
    //                      with no duplicate check and no warning at any count.
    //   - all others    → single-value via `check_multiple()`, warns on second definition.
    let max_allowed = |key: &str| -> usize {
        match key {
            "annotation" => 2,
            "commentary" => usize::MAX,
            _ => 1,
        }
    };

    let mut errors = Vec::new();
    for (key, &count) in &overwrite_counts {
        let total = count + 1; // total insertions = overwrites + 1
        let max = max_allowed(key);
        if total > max {
            let msg = if *key == "annotation" {
                format!(
                    "Too many 'annotation' definitions ({total}); \
                     GregorioTeX only uses the first 2."
                )
            } else {
                format!(
                    "Header '{key}' defined {total} time(s); only the last definition is used."
                )
            };
            errors.push(
                ParseError::new(msg, Range::zero(), Severity::Warning)
                    .with_code("duplicate-headers"),
            );
        }
    }
    errors
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
    let has_nabc = doc.notation.syllables.iter().any(|s| {
        s.notes
            .iter()
            .any(|n| n.nabc.as_ref().is_some_and(|v| !v.is_empty()))
    });
    if has_nabc && !has_header {
        let insert_line = doc.notation.range.start.line.saturating_sub(1);
        let insert_pos = Position::new(insert_line, 0);
        vec![ParseError::new(
            "pipe '|' in note group without `nabc-lines` header",
            Range::zero(),
            Severity::Error,
        )
        .with_code("nabc-without-header")
        .with_fix(TextFix {
            range: Range::new(insert_pos, insert_pos),
            new_text: "nabc-lines: 1;\n".to_string(),
        })]
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
            let parsed_owned: Option<Vec<NabcGlyphDescriptor>> =
                match (&note_group.nabc_parsed, &note_group.nabc) {
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
                let only_last =
                    pitches[..pitches.len() - 1].iter().all(|p| !*p) && *pitches.last().unwrap();
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

fn fix_nabc_fusion_modifiers(line: &str, modifier_chars: &[char]) -> String {
    if !line.contains('!') {
        return line.to_string();
    }
    let parts: Vec<&str> = line.split('!').collect();
    if parts.len() < 2 {
        return line.to_string();
    }
    let mut fixed_parts: Vec<String> = Vec::new();
    let mut collected: String = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i < parts.len() - 1 {
            let stripped: String = part
                .chars()
                .filter(|c| !modifier_chars.contains(c))
                .collect();
            let mods: String = part
                .chars()
                .filter(|c| modifier_chars.contains(c))
                .collect();
            fixed_parts.push(stripped);
            collected.push_str(&mods);
        } else {
            fixed_parts.push(format!("{part}{collected}"));
        }
    }
    fixed_parts.join("!")
}

fn modifiers_in_fused_glyphs(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    let modifier_chars = ['S', 'G', 'M', '-', '>', '~'];
    for syllable in &doc.notation.syllables {
        for note in &syllable.notes {
            let Some(nabc) = note.nabc.as_ref() else {
                continue;
            };
            let mut offending_part: Option<(&str, &str)> = None; // (part, last)
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
                        offending_part = Some((part, last));
                        break;
                    }
                }
                if offending_part.is_some() {
                    break;
                }
            }
            if let Some((part, last)) = offending_part {
                let fixed_nabc: Vec<String> = nabc
                    .iter()
                    .map(|line| fix_nabc_fusion_modifiers(line, &modifier_chars))
                    .collect();
                let mut fix_text = format!("({}", note.gabc);
                for line in &fixed_nabc {
                    fix_text.push('|');
                    fix_text.push_str(line);
                }
                fix_text.push(')');
                out.push(
                    ParseError::new(
                        format!(
                            "Modifiers in fused glyphs are only allowed on the last glyph descriptor (Gregorio 6.1.0). Found modifier in '{part}' but only '{last}' (the last glyph) can have modifiers."
                        ),
                        note.range,
                        Severity::Warning,
                    )
                    .with_code("modifiers-in-fused-glyphs")
                    .with_fix(TextFix {
                        range: note.range,
                        new_text: fix_text,
                    }),
                );
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

/// Returns the trailing line-break marker (`z`, `Z`, `z+`, `z-`, `Z+`, `Z-`) if the gabc
/// string ends with one, or `None` otherwise. `z0` (auto-custos) is naturally excluded
/// because it ends with `'0'`, not with `'z'`.
fn trailing_line_break_marker(gabc: &str) -> Option<&'static str> {
    // Longer variants first to avoid matching "z" inside "z+"
    for &suffix in &["z+", "z-", "Z+", "Z-", "z", "Z"] {
        if gabc.ends_with(suffix) {
            return Some(suffix);
        }
    }
    None
}

fn line_break_at_end_of_score(doc: &ParsedDocument) -> Vec<ParseError> {
    let last = match doc.notation.syllables.last() {
        Some(s) => s,
        None => return vec![],
    };
    let last_ng = match last.notes.last() {
        Some(ng) => ng,
        None => return vec![],
    };

    let gabc_trimmed = last_ng.gabc.trim_end();
    let lb_marker = match trailing_line_break_marker(gabc_trimmed) {
        Some(m) => m,
        None => return vec![],
    };

    // Compute fix range: include the opening '(' by subtracting 1 from the start character.
    // note_start (NoteGroup.range.start) is always at least character 1 because '(' was
    // consumed before it was recorded.
    let fix_start = Position::new(
        last_ng.range.start.line,
        last_ng.range.start.character.saturating_sub(1),
    );
    let fix_range = Range::new(fix_start, last_ng.range.end);

    let other_gabc = gabc_trimmed
        .strip_suffix(lb_marker)
        .map(|r| r.trim_end())
        .unwrap_or("");
    let has_other_content = !other_gabc.is_empty()
        || last_ng
            .nabc
            .as_ref()
            .map(|n| !n.is_empty())
            .unwrap_or(false);

    let new_text = if has_other_content {
        let mut s = String::from("(");
        s.push_str(other_gabc);
        if let Some(nabc) = &last_ng.nabc {
            for line in nabc {
                s.push('|');
                s.push_str(line);
            }
        }
        s.push(')');
        s
    } else {
        String::new()
    };

    vec![
        ParseError::new(
            format!(
                "Forced line break ('{lb_marker}') at end of score is ignored by GregorioTeX. Remove it."
            ),
            last_ng.range,
            Severity::Warning,
        )
        .with_code("line-break-at-end-of-score")
        .with_fix(TextFix {
            range: fix_range,
            new_text,
        }),
    ]
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

// ---------- Syllable text-markup helpers ----------

/// Returns the raw (original, with style tags intact) text of a syllable.
fn syllable_raw_text(syllable: &Syllable) -> &str {
    syllable
        .text_with_styles
        .as_deref()
        .unwrap_or(&syllable.text)
}

/// If `text[pos..]` starts with a `<pr…>` tag (`<pr>`, `<pr/>`, `<pr:…>`),
/// returns the byte offset just past the closing `>`. Otherwise returns `None`.
fn pr_tag_end(text: &str, pos: usize) -> Option<usize> {
    let rest = text.get(pos..)?;
    if !rest.starts_with("<pr") {
        return None;
    }
    let after = rest.as_bytes().get(3).copied()?;
    if after != b'>' && after != b':' && after != b'/' {
        return None;
    }
    let close = rest.find('>')?;
    Some(pos + close + 1)
}

/// Counts how many `<pr…>` tags appear in `text`.
fn count_pr_tags(text: &str) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while pos < text.len() {
        if let Some(end) = pr_tag_end(text, pos) {
            count += 1;
            pos = end;
        } else {
            pos += text[pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
        }
    }
    count
}

/// Returns `text` with all but the first `<pr…>` tag removed.
fn remove_duplicate_pr_tags(text: &str) -> String {
    let mut result = String::new();
    let mut pos = 0;
    let mut kept_first = false;
    while pos < text.len() {
        if let Some(end) = pr_tag_end(text, pos) {
            if !kept_first {
                kept_first = true;
                result.push_str(&text[pos..end]);
            }
            pos = end;
        } else {
            let ch_len = text[pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            result.push_str(&text[pos..pos + ch_len]);
            pos += ch_len;
        }
    }
    result
}

/// Returns `text` with unmatched `}` characters removed (i.e. `}` appearing when
/// `{`-depth is already zero).
fn remove_unmatched_close_braces(text: &str) -> String {
    let mut result = String::new();
    let mut depth = 0usize;
    for ch in text.chars() {
        match ch {
            '{' => {
                depth += 1;
                result.push(ch);
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                    result.push(ch);
                }
                // else: skip the unmatched '}'
            }
            _ => result.push(ch),
        }
    }
    result
}

/// Returns `text` with a closing `}` inserted immediately before the first `<pr…>` tag
/// found while a `{` is still open.
fn close_center_before_pr(text: &str) -> String {
    let mut result = String::new();
    let mut pos = 0;
    let mut depth = 0usize;
    let mut inserted = false;
    while pos < text.len() {
        if !inserted {
            if let Some(end) = pr_tag_end(text, pos) {
                if depth > 0 {
                    result.push('}');
                    inserted = true;
                }
                result.push_str(&text[pos..end]);
                pos = end;
                continue;
            }
        }
        let ch_len = text[pos..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        let ch = text[pos..].chars().next().unwrap_or('\0');
        match ch {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            _ => {}
        }
        result.push_str(&text[pos..pos + ch_len]);
        pos += ch_len;
    }
    result
}

// ---------- Syllable text-markup rules ----------

fn duplicate_syllable_center(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        let open_count = syllable.text.chars().filter(|&c| c == '{').count();
        if open_count >= 2 {
            out.push(
                ParseError::new(
                    format!(
                        "Syllable '{}' has {} forced-center '{{}}' markers; \
                         only the first is used by GregorioTeX.",
                        syllable.text, open_count
                    ),
                    syllable.text_range,
                    Severity::Warning,
                )
                .with_code("duplicate-syllable-center"),
            );
        }
    }
    out
}

fn center_after_protrusion(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        let text = &syllable.text;
        let mut pos = 0;
        let mut found_pr = false;
        let mut fire = false;
        while pos < text.len() {
            if let Some(end) = pr_tag_end(text, pos) {
                found_pr = true;
                pos = end;
                continue;
            }
            let ch_len = text[pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            let ch = text[pos..].chars().next().unwrap_or('\0');
            if found_pr && ch == '{' {
                fire = true;
                break;
            }
            pos += ch_len;
        }
        if fire {
            out.push(
                ParseError::new(
                    format!(
                        "Forced center '{{}}' appears after a '<pr>' protrusion in syllable '{}'; \
                         the center will be ignored by GregorioTeX.",
                        syllable.text
                    ),
                    syllable.text_range,
                    Severity::Warning,
                )
                .with_code("center-after-protrusion"),
            );
        }
    }
    out
}

fn unmatched_center_close(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        let mut depth = 0usize;
        let mut found = false;
        for ch in syllable.text.chars() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    if depth == 0 {
                        found = true;
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        if found {
            let fixed = remove_unmatched_close_braces(syllable_raw_text(syllable));
            out.push(
                ParseError::new(
                    format!(
                        "Unmatched '}}' in syllable '{}': \
                         closing center marker without an opening '{{'.",
                        syllable.text
                    ),
                    syllable.text_range,
                    Severity::Warning,
                )
                .with_code("unmatched-center-close")
                .with_fix(TextFix {
                    range: syllable.text_range,
                    new_text: fixed,
                }),
            );
        }
    }
    out
}

fn duplicate_protrusion(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        let count = count_pr_tags(&syllable.text);
        if count >= 2 {
            let fixed = remove_duplicate_pr_tags(syllable_raw_text(syllable));
            out.push(
                ParseError::new(
                    format!(
                        "Syllable '{}' has {} '<pr>' protrusion tags; \
                         only the first is used by GregorioTeX.",
                        syllable.text, count
                    ),
                    syllable.text_range,
                    Severity::Warning,
                )
                .with_code("duplicate-protrusion")
                .with_fix(TextFix {
                    range: syllable.text_range,
                    new_text: fixed,
                }),
            );
        }
    }
    out
}

fn unclosed_center_before_protrusion(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        let text = &syllable.text;
        let mut pos = 0;
        let mut depth = 0usize;
        let mut found = false;
        while pos < text.len() {
            if let Some(end) = pr_tag_end(text, pos) {
                if depth > 0 {
                    found = true;
                    break;
                }
                pos = end;
                continue;
            }
            let ch_len = text[pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            let ch = text[pos..].chars().next().unwrap_or('\0');
            match ch {
                '{' => depth += 1,
                '}' if depth > 0 => depth -= 1,
                _ => {}
            }
            pos += ch_len;
        }
        if found {
            let fixed = close_center_before_pr(syllable_raw_text(syllable));
            out.push(
                ParseError::new(
                    format!(
                        "Unclosed center '{{' before '<pr>' protrusion in syllable '{}'; \
                         GregorioTeX automatically closes the center before the protrusion.",
                        syllable.text
                    ),
                    syllable.text_range,
                    Severity::Warning,
                )
                .with_code("unclosed-center-before-protrusion")
                .with_fix(TextFix {
                    range: syllable.text_range,
                    new_text: fixed,
                }),
            );
        }
    }
    out
}

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
pub const VALIDATE_DUPLICATE_SYLLABLE_CENTER: ValidationRule = ValidationRule {
    name: "duplicate-syllable-center",
    severity: Severity::Warning,
    validate: duplicate_syllable_center,
};
pub const VALIDATE_CENTER_AFTER_PROTRUSION: ValidationRule = ValidationRule {
    name: "center-after-protrusion",
    severity: Severity::Warning,
    validate: center_after_protrusion,
};
pub const VALIDATE_UNMATCHED_CENTER_CLOSE: ValidationRule = ValidationRule {
    name: "unmatched-center-close",
    severity: Severity::Warning,
    validate: unmatched_center_close,
};
pub const VALIDATE_DUPLICATE_PROTRUSION: ValidationRule = ValidationRule {
    name: "duplicate-protrusion",
    severity: Severity::Warning,
    validate: duplicate_protrusion,
};
pub const VALIDATE_UNCLOSED_CENTER_BEFORE_PROTRUSION: ValidationRule = ValidationRule {
    name: "unclosed-center-before-protrusion",
    severity: Severity::Warning,
    validate: unclosed_center_before_protrusion,
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
pub const VALIDATE_LINE_BREAK_AT_END_OF_SCORE: ValidationRule = ValidationRule {
    name: "line-break-at-end-of-score",
    severity: Severity::Warning,
    validate: line_break_at_end_of_score,
};

pub fn all_validation_rules() -> Vec<&'static ValidationRule> {
    vec![
        &VALIDATE_NAME_HEADER,
        &VALIDATE_DUPLICATE_HEADERS,
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
        &VALIDATE_LINE_BREAK_AT_END_OF_SCORE,
        &VALIDATE_DUPLICATE_SYLLABLE_CENTER,
        &VALIDATE_CENTER_AFTER_PROTRUSION,
        &VALIDATE_UNMATCHED_CENTER_CLOSE,
        &VALIDATE_DUPLICATE_PROTRUSION,
        &VALIDATE_UNCLOSED_CENTER_BEFORE_PROTRUSION,
    ]
}
