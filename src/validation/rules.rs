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
                format!("Header '{key}' defined {total} time(s); only the last definition is used.")
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

fn oriscus_followed_by_equal_or_higher(doc: &ParsedDocument) -> Vec<ParseError> {
    // Collect all note groups across all syllables in document order.
    // Each syllable holds at most one note group; we flatten for generality.
    let all_groups: Vec<&NoteGroup> = doc
        .notation
        .syllables
        .iter()
        .flat_map(|s| s.notes.iter())
        .collect();

    let mut out = Vec::new();
    for g_idx in 0..all_groups.len() {
        let notes = &all_groups[g_idx].notes;
        for i in 0..notes.len() {
            let note = &notes[i];
            if note.shape != NoteShape::Oriscus {
                continue;
            }
            // If there is a subsequent note in the same group, the oriscus is part of
            // a salicus or pes-quassus — the following higher note is intentional.
            if notes.get(i + 1).is_some() {
                continue;
            }
            // The oriscus is the last (or only) note in its group.
            // Find the first note of the next non-empty group (cross-group boundary).
            let next_note = all_groups[g_idx + 1..]
                .iter()
                .flat_map(|g| g.notes.iter())
                .next();
            if let Some(n) = next_note {
                if pitch_value(n.pitch) >= pitch_value(note.pitch) {
                    out.push(
                        ParseError::new(
                            format!(
                                "Oriscus on '{}' followed by a note of equal or higher pitch \
                                 '{}': violates Gregorian semiological rule \
                                 (oriscus must always lead to a lower note)",
                                note.pitch, n.pitch
                            ),
                            note.range,
                            Severity::Warning,
                        )
                        .with_code("oriscus-higher-pitch"),
                    );
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

/// Returns the trailing line-break marker (`z`, `Z`, `z+`, `z-`, `Z+`, `Z-`) if the gabc
/// string ends with one, or `None` otherwise. `z0` (auto-custos) is naturally excluded
/// because it ends with `'0'`, not with `'z'`.
fn trailing_line_break_marker(gabc: &str) -> Option<&'static str> {
    // Longer variants first to avoid matching "z" inside "z+"
    ["z+", "z-", "Z+", "Z-", "z", "Z"]
        .iter()
        .find(|&&suffix| gabc.ends_with(suffix))
        .copied()
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

fn gabc_has_bar_outside_attributes(gabc: &str) -> bool {
    let mut depth = 0usize;
    for ch in gabc.chars() {
        match ch {
            '[' => depth += 1,
            ']' if depth > 0 => depth -= 1,
            ',' | ';' | ':' if depth == 0 => return true,
            _ => {}
        }
    }
    false
}

/// Split `gabc` into alternating (note-content, bar) segments at every bar
/// character (`,`, `;`, `:`), treating `::` as a single unit. Content inside
/// `[…]` attribute brackets is left untouched in the surrounding note segment.
fn split_gabc_at_bars(gabc: &str) -> Vec<(bool, String)> {
    let mut parts: Vec<(bool, String)> = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = gabc.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch == '[' {
            let mut depth = 0usize;
            while i < chars.len() {
                let c = chars[i];
                current.push(c);
                match c {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        i += 1;
                        if depth == 0 {
                            break;
                        }
                        continue;
                    }
                    _ => {}
                }
                i += 1;
            }
            continue;
        }

        if ch == ':' && chars.get(i + 1) == Some(&':') {
            parts.push((false, std::mem::take(&mut current)));
            parts.push((true, "::".to_string()));
            i += 2;
            continue;
        }

        if matches!(ch, ',' | ';' | ':') {
            parts.push((false, std::mem::take(&mut current)));
            parts.push((true, ch.to_string()));
            i += 1;
            continue;
        }

        current.push(ch);
        i += 1;
    }
    parts.push((false, current));
    parts
}

fn fix_bar_mixed_with_notes(note_group: &NoteGroup) -> TextFix {
    let parts = split_gabc_at_bars(&note_group.gabc);
    let mut groups: Vec<String> = Vec::new();
    let mut nabc_attached = false;

    for (is_bar, content) in &parts {
        if *is_bar {
            groups.push(format!("({})", content));
        } else {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut group = format!("({}", trimmed);
            if !nabc_attached {
                nabc_attached = true;
                if let Some(nabc_lines) = &note_group.nabc {
                    for line in nabc_lines {
                        group.push('|');
                        group.push_str(line);
                    }
                }
            }
            group.push(')');
            groups.push(group);
        }
    }

    let new_text = groups.join(" ");
    let fix_start = Position::new(
        note_group.range.start.line,
        note_group.range.start.character.saturating_sub(1),
    );
    TextFix {
        range: Range::new(fix_start, note_group.range.end),
        new_text,
    }
}

fn bar_mixed_with_notes(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        for note_group in &syllable.notes {
            if note_group.notes.is_empty() {
                continue;
            }
            if !gabc_has_bar_outside_attributes(&note_group.gabc) {
                continue;
            }
            out.push(
                ParseError::new(
                    "Bar symbol mixed with notes in the same group; \
                     bars must appear in their own group: (,) (;) (:) (::)",
                    note_group.range,
                    Severity::Warning,
                )
                .with_code("bar-mixed-with-notes")
                .with_fix(fix_bar_mixed_with_notes(note_group)),
            );
        }
    }
    out
}

fn nabc_space_in_code(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for syllable in &doc.notation.syllables {
        for note_group in &syllable.notes {
            let Some(nabc_lines) = &note_group.nabc else {
                continue;
            };
            if !nabc_lines.iter().any(|l| l.contains(' ')) {
                continue;
            }
            let fixed_nabc: Vec<String> = nabc_lines
                .iter()
                .map(|l| l.chars().filter(|&c| c != ' ').collect())
                .collect();
            let mut new_text = String::from("(");
            new_text.push_str(&note_group.gabc);
            for line in &fixed_nabc {
                new_text.push('|');
                new_text.push_str(line);
            }
            new_text.push(')');
            let fix_start = Position::new(
                note_group.range.start.line,
                note_group.range.start.character.saturating_sub(1),
            );
            out.push(
                ParseError::new(
                    "Whitespace inside NABC code is rendered incorrectly by Gregorio 6.2.0; remove spaces.",
                    note_group.range,
                    Severity::Warning,
                )
                .with_code("nabc-space-in-code")
                .with_fix(TextFix {
                    range: Range::new(fix_start, note_group.range.end),
                    new_text,
                }),
            );
        }
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

fn is_misplaced_post_note_punctuation(ch: char) -> bool {
    matches!(ch, ',' | ';' | ':' | '.' | '!' | '?')
}

fn leading_misplaced_punctuation_prefix(text: &str) -> Option<usize> {
    let mut end = 0usize;
    for (idx, ch) in text.char_indices() {
        if is_misplaced_post_note_punctuation(ch) {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    (end > 0).then_some(end)
}

fn note_group_source(group: &NoteGroup) -> String {
    let mut text = String::from("(");
    text.push_str(&group.gabc);
    if let Some(nabc) = &group.nabc {
        for line in nabc {
            text.push('|');
            text.push_str(line);
        }
    }
    text.push(')');
    text
}

fn trailing_note_group_source(syllable: &Syllable) -> Option<(Position, String)> {
    if let Some(group) = syllable.notes.last() {
        return Some((
            Position::new(
                group.range.start.line,
                group.range.start.character.saturating_sub(1),
            ),
            note_group_source(group),
        ));
    }
    (syllable.range.end != syllable.text_range.end)
        .then_some((syllable.text_range.end, "()".into()))
}

// ---------- Syllable text-markup rules ----------

fn punctuation_after_note_group(doc: &ParsedDocument) -> Vec<ParseError> {
    let mut out = Vec::new();
    for window in doc.notation.syllables.windows(2) {
        let previous = &window[0];
        let current = &window[1];
        let Some((fix_start, previous_group_text)) = trailing_note_group_source(previous) else {
            continue;
        };
        let raw_text = syllable_raw_text(current);
        let Some(prefix_end) = leading_misplaced_punctuation_prefix(raw_text) else {
            continue;
        };
        let remainder = &raw_text[prefix_end..];
        // A punctuation-only syllable with its own note group can be intentional.
        // Only flag whitespace-only remainders when the punctuation is trailing text
        // that was typed after the previous syllable's note group.
        if remainder.trim_start().is_empty() && !current.notes.is_empty() {
            continue;
        }

        let punctuation = &raw_text[..prefix_end];
        let fixed = format!("{}{}{}", punctuation, previous_group_text, remainder);

        out.push(
            ParseError::new(
                format!(
                    "Punctuation '{}' appears after the previous note group in syllable '{}'; \
                     move it before the previous syllable's parentheses or GregorioTeX may hyphenate the text incorrectly.",
                    punctuation, current.text
                ),
                current.text_range,
                Severity::Warning,
            )
            .with_code("punctuation-after-note-group")
            .with_fix(TextFix {
                range: Range::new(fix_start, current.text_range.end),
                new_text: fixed,
            }),
        );
    }
    out
}

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
pub const VALIDATE_PUNCTUATION_AFTER_NOTE_GROUP: ValidationRule = ValidationRule {
    name: "punctuation-after-note-group",
    severity: Severity::Warning,
    validate: punctuation_after_note_group,
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
pub const VALIDATE_ORISCUS_HIGHER: ValidationRule = ValidationRule {
    name: "oriscus-higher-pitch",
    severity: Severity::Warning,
    validate: oriscus_followed_by_equal_or_higher,
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

pub const VALIDATE_LINE_BREAK_AT_END_OF_SCORE: ValidationRule = ValidationRule {
    name: "line-break-at-end-of-score",
    severity: Severity::Warning,
    validate: line_break_at_end_of_score,
};

pub const VALIDATE_NABC_SPACE_IN_CODE: ValidationRule = ValidationRule {
    name: "nabc-space-in-code",
    severity: Severity::Warning,
    validate: nabc_space_in_code,
};

pub const VALIDATE_BAR_MIXED_WITH_NOTES: ValidationRule = ValidationRule {
    name: "bar-mixed-with-notes",
    severity: Severity::Warning,
    validate: bar_mixed_with_notes,
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
        &VALIDATE_ORISCUS_HIGHER,
        &VALIDATE_STAFF_LINES,
        &VALIDATE_BALANCED_PITCH_DESCRIPTORS_FUSED,
        &VALIDATE_MODIFIERS_FUSED,
        &VALIDATE_LINE_BREAK_AT_END_OF_SCORE,
        &VALIDATE_NABC_SPACE_IN_CODE,
        &VALIDATE_BAR_MIXED_WITH_NOTES,
        &VALIDATE_DUPLICATE_SYLLABLE_CENTER,
        &VALIDATE_PUNCTUATION_AFTER_NOTE_GROUP,
        &VALIDATE_CENTER_AFTER_PROTRUSION,
        &VALIDATE_UNMATCHED_CENTER_CLOSE,
        &VALIDATE_DUPLICATE_PROTRUSION,
        &VALIDATE_UNCLOSED_CENTER_BEFORE_PROTRUSION,
    ]
}
