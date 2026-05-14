//! Semantic analyzer: deep checks over the parsed AST (musical constructions, NABC, headers).

use crate::parser::types::*;

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub code: String,
    pub message: String,
    pub range: Range,
    pub severity: Severity,
    pub related_info: Vec<RelatedInfo>,
    pub fix: Option<TextFix>,
}

#[derive(Debug, Clone)]
pub struct RelatedInfo {
    pub message: String,
    pub range: Range,
}

impl SemanticError {
    pub fn to_parse_error(&self) -> ParseError {
        ParseError {
            message: self.message.clone(),
            range: self.range,
            severity: self.severity,
            code: Some(self.code.clone()),
            fix: self.fix.clone(),
        }
    }
}

#[derive(Default)]
pub struct SemanticAnalyzer {
    errors: Vec<SemanticError>,
    warnings: Vec<SemanticError>,
    info: Vec<SemanticError>,
    notation_start: Position,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze(&mut self, doc: &ParsedDocument) -> Vec<SemanticError> {
        self.errors.clear();
        self.warnings.clear();
        self.info.clear();
        self.notation_start = doc.notation.range.start;

        self.validate_headers(&doc.headers);

        if !doc.notation.syllables.is_empty() {
            self.validate_first_syllable(&doc.notation.syllables[0]);
            self.validate_syllables(&doc.notation.syllables, &doc.headers);
        }

        let mut all = Vec::with_capacity(self.errors.len() + self.warnings.len() + self.info.len());
        all.extend(self.errors.iter().cloned());
        all.extend(self.warnings.iter().cloned());
        all.extend(self.info.iter().cloned());
        all
    }

    pub fn errors(&self) -> &[SemanticError] {
        &self.errors
    }
    pub fn warnings(&self) -> &[SemanticError] {
        &self.warnings
    }
    pub fn info(&self) -> &[SemanticError] {
        &self.info
    }

    // ---------- Header checks ----------

    fn validate_headers(&mut self, headers: &HeaderMap) {
        let missing_name = match headers.get("name") {
            None => true,
            Some(v) => v.trim().is_empty(),
        };
        if missing_name {
            self.warnings.push(SemanticError {
                code: "missing-name-header".into(),
                message: "No name specified. Put 'name:...;' at the beginning of the file. Can be dangerous with some output formats.".into(),
                range: Range::zero(),
                severity: Severity::Warning,
                related_info: Vec::new(),
                fix: None,
            });
        }
        // Duplicate-header detection requires preserving insertion list with dups; HeaderMap collapses
        // duplicates to mirror TypeScript Map. Skipped intentionally.
    }

    // ---------- Syllable checks ----------

    fn validate_first_syllable(&mut self, first: &Syllable) {
        if first.line_break.is_some() {
            self.errors.push(SemanticError {
                code: "line-break-on-first-syllable".into(),
                message: "Line break is not supported on the first syllable".into(),
                range: first.range,
                severity: Severity::Error,
                related_info: Vec::new(),
                fix: None,
            });
        }
    }

    fn validate_syllables(&mut self, syllables: &[Syllable], headers: &HeaderMap) {
        let has_nabc_lines = headers.has("nabc-lines");
        for i in 0..syllables.len() {
            let prev = if i == 0 { None } else { Some(&syllables[i - 1]) };
            let syll = &syllables[i];
            for note_group in &syll.notes {
                self.validate_note_group(note_group, has_nabc_lines, prev);
            }
        }
    }

    fn validate_note_group(
        &mut self,
        note_group: &NoteGroup,
        has_nabc_lines: bool,
        previous: Option<&Syllable>,
    ) {
        if let Some(nabc) = &note_group.nabc {
            if !nabc.is_empty() && !has_nabc_lines {
                let insert_line = self.notation_start.line.saturating_sub(1);
                let insert_pos = Position::new(insert_line, 0);
                self.errors.push(SemanticError {
                    code: "pipe-without-nabc-lines".into(),
                    message: "Pipe '|' in note group without 'nabc-lines' header. Add 'nabc-lines: 1;' (or higher) to the file header.".into(),
                    range: note_group.range,
                    severity: Severity::Error,
                    related_info: Vec::new(),
                    fix: Some(TextFix {
                        range: Range::new(insert_pos, insert_pos),
                        new_text: "nabc-lines: 1;\n".to_string(),
                    }),
                });
            }
        }

        self.validate_musical_constructions(note_group, previous);

        if let Some(parsed) = &note_group.nabc_parsed {
            self.validate_nabc(parsed);
        }
    }

    fn validate_musical_constructions(
        &mut self,
        note_group: &NoteGroup,
        previous: Option<&Syllable>,
    ) {
        let notes = &note_group.notes;
        for i in 0..notes.len() {
            let note = &notes[i];
            let next = notes.get(i + 1);
            let prev = if i == 0 { None } else { notes.get(i - 1) };

            let has_quadratum = has_mod(note, ModifierType::Quadratum);
            let has_fusion = has_mod(note, ModifierType::Fusion);
            if has_quadratum && next.is_none() && !has_fusion {
                self.warnings.push(SemanticError {
                    code: "pes-quadratum-missing-note".into(),
                    message: format!(
                        "Pes quadratum at '{}' requires a subsequent note. Example: ({}q{})",
                        note.pitch,
                        note.pitch,
                        next_pitch_example(note.pitch)
                    ),
                    range: note.range,
                    severity: Severity::Warning,
                    related_info: Vec::new(),
                    fix: None,
                });
            }

            let note_fusion = has_mod(note, ModifierType::Fusion);
            if note.shape == NoteShape::Quilisma && next.is_none() && !note_fusion {
                self.warnings.push(SemanticError {
                    code: "quilisma-missing-note".into(),
                    message: format!(
                        "Quilisma at '{}' requires a subsequent note. Example: ({}w{})",
                        note.pitch,
                        note.pitch,
                        next_pitch_example(note.pitch)
                    ),
                    range: note.range,
                    severity: Severity::Warning,
                    related_info: Vec::new(),
                    fix: None,
                });
            }

            let has_oriscus_scapus = has_mod(note, ModifierType::OriscusScapus);
            let prev_fusion = prev.map(|p| has_mod(p, ModifierType::Fusion)).unwrap_or(false);
            let cur_fusion = has_mod(note, ModifierType::Fusion);
            if has_oriscus_scapus {
                let valid_prev = prev.is_some() || prev_fusion;
                let valid_next = next.is_some() || cur_fusion;
                if !valid_prev && !valid_next {
                    self.warnings.push(SemanticError {
                        code: "oriscus-scapus-isolated".into(),
                        message: format!(
                            "Oriscus scapus at '{}' requires both preceding and subsequent notes. Example: ({}{}O{})",
                            note.pitch,
                            previous_pitch_example(note.pitch),
                            note.pitch,
                            next_pitch_example(note.pitch)
                        ),
                        range: note.range,
                        severity: Severity::Warning,
                        related_info: Vec::new(),
                        fix: None,
                    });
                } else if !valid_prev {
                    let next_pitch = next.map(|n| n.pitch).unwrap_or_else(|| next_pitch_example(note.pitch));
                    self.warnings.push(SemanticError {
                        code: "oriscus-scapus-missing-preceding".into(),
                        message: format!(
                            "Oriscus scapus at '{}' requires a preceding note. Example: ({}{}O{})",
                            note.pitch,
                            previous_pitch_example(note.pitch),
                            note.pitch,
                            next_pitch
                        ),
                        range: note.range,
                        severity: Severity::Warning,
                        related_info: Vec::new(),
                        fix: None,
                    });
                } else if !valid_next {
                    let prev_pitch = prev.map(|p| p.pitch).unwrap_or_else(|| previous_pitch_example(note.pitch));
                    self.warnings.push(SemanticError {
                        code: "oriscus-scapus-missing-subsequent".into(),
                        message: format!(
                            "Oriscus scapus at '{}' requires a subsequent note. Example: ({}{}O{})",
                            note.pitch,
                            prev_pitch,
                            note.pitch,
                            next_pitch_example(note.pitch)
                        ),
                        range: note.range,
                        severity: Severity::Warning,
                        related_info: Vec::new(),
                        fix: None,
                    });
                }
            }

            // Quilisma followed by equal/lower
            if note.shape == NoteShape::Quilisma {
                if let Some(n) = next {
                    if compare_pitch(n.pitch, note.pitch) <= 0 {
                        self.warnings.push(SemanticError {
                            code: "quilisma-equal-or-lower".into(),
                            message: format!(
                                "Quilisma note '{}' followed by equal or lower pitch '{}'. This may cause rendering issues.",
                                note.pitch, n.pitch
                            ),
                            range: note.range,
                            severity: Severity::Warning,
                            related_info: vec![RelatedInfo {
                                message: "Following note".into(),
                                range: n.range,
                            }],
                            fix: None,
                        });
                    } else {
                        // Quilisma-pes preceded by equal/higher pitch
                        let prev_from_prev_syll = previous_note(previous);
                        let actual_prev = prev.cloned().or(prev_from_prev_syll);
                        if let Some(ap) = actual_prev {
                            if compare_pitch(ap.pitch, note.pitch) >= 0 {
                                self.warnings.push(SemanticError {
                                    code: "quilisma-pes-preceded-by-higher".into(),
                                    message: format!(
                                        "Quilisma-pes at '{}' preceded by equal or higher pitch '{}'. This may cause spacing issues.",
                                        note.pitch, ap.pitch
                                    ),
                                    range: note.range,
                                    severity: Severity::Warning,
                                    related_info: Vec::new(),
                                    fix: None,
                                });
                            }
                        }
                    }
                }
            }

            // Virga strata followed by equal/higher pitch
            if note.shape == NoteShape::Virga
                && has_mod(note, ModifierType::Strata)
            {
                if let Some(n) = next {
                    if compare_pitch(n.pitch, note.pitch) >= 0 {
                        self.warnings.push(SemanticError {
                            code: "virga-strata-equal-or-higher".into(),
                            message: format!(
                                "Virga strata at '{}' followed by equal or higher pitch '{}'. This may cause placement issues.",
                                note.pitch, n.pitch
                            ),
                            range: note.range,
                            severity: Severity::Warning,
                            related_info: vec![RelatedInfo {
                                message: "Following note".into(),
                                range: n.range,
                            }],
                            fix: None,
                        });
                    }
                }
            }

            // Pes stratus
            if note.shape == NoteShape::Punctum
                && has_mod(note, ModifierType::Strata)
            {
                if let (Some(pes), Some(after_pes)) = (notes.get(i + 1), notes.get(i + 2)) {
                    if has_mod(pes, ModifierType::Strata)
                        && compare_pitch(pes.pitch, note.pitch) > 0
                        && compare_pitch(after_pes.pitch, pes.pitch) >= 0
                    {
                        self.warnings.push(SemanticError {
                            code: "pes-stratus-equal-or-higher".into(),
                            message: format!(
                                "Pes stratus ending at '{}' followed by equal or higher pitch '{}'. This may cause placement issues.",
                                pes.pitch, after_pes.pitch
                            ),
                            range: pes.range,
                            severity: Severity::Warning,
                            related_info: vec![RelatedInfo {
                                message: "Following note".into(),
                                range: after_pes.range,
                            }],
                            fix: None,
                        });
                    }
                }
            }
        }

        self.check_quilismatic_connector(note_group);
    }

    fn validate_nabc(&mut self, descriptors: &[NabcGlyphDescriptor]) {
        for descriptor in descriptors {
            if let Some(mods) = &descriptor.modifiers {
                let aug = mods.contains(&NabcGlyphModifier::AugmentiveLiquescence);
                let dim = mods.contains(&NabcGlyphModifier::DiminutiveLiquescence);
                if aug && dim {
                    if let Some(range) = descriptor.range {
                        self.warnings.push(SemanticError {
                            code: "nabc-conflicting-liquescence".into(),
                            message: "NABC descriptor cannot have both augmentive (>) and diminutive (~) liquescence".into(),
                            range,
                            severity: Severity::Warning,
                            related_info: Vec::new(),
                            fix: None,
                        });
                    }
                }
            }

            if let Some(p) = descriptor.pitch {
                if !matches!(p, 'a'..='n' | 'p') {
                    if let Some(range) = descriptor.range {
                        self.warnings.push(SemanticError {
                            code: "nabc-invalid-pitch".into(),
                            message: format!(
                                "Invalid NABC pitch descriptor: {p}. Must be a-n or p."
                            ),
                            range,
                            severity: Severity::Warning,
                            related_info: Vec::new(),
                            fix: None,
                        });
                    }
                }
            }

            if let Some(fusion) = &descriptor.fusion {
                self.validate_nabc(std::slice::from_ref(fusion.as_ref()));
            }
        }
    }

    fn check_quilismatic_connector(&mut self, note_group: &NoteGroup) {
        let notes = &note_group.notes;
        if notes.len() < 3 {
            return;
        }
        for i in 0..notes.len() {
            let note = &notes[i];
            if note.shape == NoteShape::Quilisma && i > 0 && !has_mod(&notes[i - 1], ModifierType::Fusion) {
                let insert_pos = note.range.start;
                self.info.push(SemanticError {
                    code: "quilisma-missing-connector".into(),
                    message: format!(
                        "Consider adding the fusion operator '@' (preferred) to fuse the note before the quilisma. Alternative: use the spacing connector '!'. Examples: {}",
                        format_note_sequence(notes, i)
                    ),
                    range: note.range,
                    severity: Severity::Info,
                    related_info: Vec::new(),
                    fix: Some(TextFix {
                        range: Range::new(insert_pos, insert_pos),
                        new_text: "@".to_string(),
                    }),
                });
            }
        }
    }
}

// ---------- Helpers ----------

fn has_mod(note: &Note, kind: ModifierType) -> bool {
    note.modifiers.iter().any(|m| m.kind == kind)
}

fn compare_pitch(a: char, b: char) -> i32 {
    let order = "abcdefghijklmn";
    let ia = order.find(a.to_ascii_lowercase()).map(|x| x as i32).unwrap_or(-1);
    let ib = order.find(b.to_ascii_lowercase()).map(|x| x as i32).unwrap_or(-1);
    if ia < 0 || ib < 0 {
        return 0;
    }
    ia - ib
}

fn next_pitch_example(p: char) -> char {
    let order = "abcdefghijklmn";
    let idx = order.find(p.to_ascii_lowercase());
    match idx {
        Some(i) if i + 1 < order.len() => order.chars().nth(i + 1).unwrap(),
        _ => 'g',
    }
}

fn previous_pitch_example(p: char) -> char {
    let order = "abcdefghijklmn";
    let idx = order.find(p.to_ascii_lowercase());
    match idx {
        Some(i) if i > 0 => order.chars().nth(i - 1).unwrap(),
        _ => 'd',
    }
}

fn previous_note(prev: Option<&Syllable>) -> Option<Note> {
    let prev = prev?;
    let last_group = prev.notes.last()?;
    last_group.notes.last().cloned()
}

fn format_note_sequence(notes: &[Note], q_index: usize) -> String {
    let before = if q_index > 0 {
        notes[q_index - 1].pitch.to_string()
    } else {
        String::new()
    };
    let quilisma = format!("{}w", notes[q_index].pitch);
    let after = notes
        .get(q_index + 1)
        .map(|n| n.pitch.to_string())
        .unwrap_or_default();
    format!("({before}@{quilisma}{after}) or ({before}!{quilisma}{after})")
}

pub fn analyze_semantics(doc: &ParsedDocument) -> Vec<SemanticError> {
    SemanticAnalyzer::new().analyze(doc)
}
