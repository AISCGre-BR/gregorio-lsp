//! NABC (St. Gall / Laon) glyph descriptor parser.

use super::types::*;

pub fn parse_nabc_snippet(nabc: &str, start: Option<Position>) -> Option<NabcGlyphDescriptor> {
    parse_nabc_descriptors(nabc, start).into_iter().next()
}

pub fn parse_nabc_descriptors(nabc: &str, start: Option<Position>) -> Vec<NabcGlyphDescriptor> {
    let trimmed: String = nabc.trim().chars().filter(|c| !c.is_whitespace()).collect();
    if trimmed.is_empty() {
        return Vec::new();
    }
    parse_complex_neume_descriptors(&trimmed, start)
}

pub fn parse_nabc_snippets(
    snippets: &[String],
    start: Option<Position>,
) -> Vec<NabcGlyphDescriptor> {
    let mut all = Vec::new();
    for (idx, snippet) in snippets.iter().enumerate() {
        let pos = start.map(|p| Position::new(p.line, p.character + idx * 10));
        all.extend(parse_nabc_descriptors(snippet, pos));
    }
    all
}

fn parse_complex_neume_descriptors(
    nabc: &str,
    start: Option<Position>,
) -> Vec<NabcGlyphDescriptor> {
    let chars: Vec<char> = nabc.chars().collect();
    let mut descriptors = Vec::new();
    let mut pos = 0;

    while pos < chars.len() {
        while pos < chars.len() && (chars[pos] == '/' || chars[pos] == '`') {
            pos += 1;
        }
        if pos >= chars.len() {
            break;
        }
        let cur = start.map(|p| Position::new(p.line, p.character + pos));
        match parse_single_complex_descriptor(&chars, pos, cur) {
            Some((desc, consumed)) => {
                descriptors.push(desc);
                pos = consumed;
            }
            None => {
                pos += 1;
            }
        }
    }
    descriptors
}

fn substring_starts_with(chars: &[char], at: usize, prefix: &str) -> bool {
    let pchars: Vec<char> = prefix.chars().collect();
    if at + pchars.len() > chars.len() {
        return false;
    }
    chars[at..at + pchars.len()] == pchars[..]
}

fn slice_str(chars: &[char], start: usize, end: usize) -> String {
    chars[start..end.min(chars.len())].iter().collect()
}

fn parse_single_complex_descriptor(
    chars: &[char],
    start: usize,
    position: Option<Position>,
) -> Option<(NabcGlyphDescriptor, usize)> {
    let (head, consumed) = parse_complex_glyph_descriptor(chars, start, position)?;
    let mut chain: Vec<NabcGlyphDescriptor> = vec![head];
    let mut pos = consumed;

    // Fusion chain via '!'
    while pos < chars.len() && chars[pos] == '!' {
        let fusion_pos = pos + 1;
        if fusion_pos >= chars.len() {
            break;
        }
        if substring_starts_with(chars, fusion_pos, "su")
            || substring_starts_with(chars, fusion_pos, "pp")
        {
            break;
        }
        let next_pos = position.map(|p| Position::new(p.line, p.character + fusion_pos));
        match parse_complex_glyph_descriptor(chars, fusion_pos, next_pos) {
            Some((next, next_consumed)) => {
                chain.push(next);
                pos = next_consumed;
            }
            None => break,
        }
    }

    // Build the fusion chain back-to-front into a single linked descriptor
    let mut tail: Option<Box<NabcGlyphDescriptor>> = None;
    while chain.len() > 1 {
        let mut node = chain.pop().unwrap();
        node.fusion = tail.take();
        tail = Some(Box::new(node));
    }
    let mut first = chain.pop().unwrap();
    first.fusion = tail;

    // Trailing subpunctis / prepunctis attached to the first descriptor
    let mut subpunctis_list: Vec<NabcSubpunctis> = Vec::new();
    let mut prepunctis_list: Vec<NabcPrepunctis> = Vec::new();
    while pos < chars.len() {
        if !(substring_starts_with(chars, pos, "su") || substring_starts_with(chars, pos, "pp")) {
            break;
        }
        let pp_pos = position.map(|p| Position::new(p.line, p.character + pos));
        let remaining: String = slice_str(chars, pos, chars.len());
        match parse_subpunctis_prepunctis(&remaining, pp_pos) {
            Some((descriptor, consumed)) => {
                if let Some(sp) = descriptor.subpunctis {
                    subpunctis_list.push(sp);
                }
                if let Some(pp) = descriptor.prepunctis {
                    prepunctis_list.push(pp);
                }
                pos += consumed;
            }
            None => break,
        }
    }

    if let Some(last) = subpunctis_list.pop() {
        first.subpunctis = Some(last);
    }
    if let Some(last) = prepunctis_list.pop() {
        first.prepunctis = Some(last);
    }

    Some((first, pos))
}

fn parse_complex_glyph_descriptor(
    chars: &[char],
    start: usize,
    position: Option<Position>,
) -> Option<(NabcGlyphDescriptor, usize)> {
    let mut pos = start;

    // Subpunctis/prepunctis at the head?
    if substring_starts_with(chars, pos, "su") || substring_starts_with(chars, pos, "pp") {
        let remaining: String = slice_str(chars, pos, chars.len());
        if let Some((desc, consumed)) = parse_subpunctis_prepunctis(&remaining, position) {
            return Some((desc, pos + consumed));
        }
    }

    if pos + 1 >= chars.len() {
        return None;
    }
    let code: String = chars[pos..pos + 2].iter().collect();
    let basic_glyph = NabcBasicGlyph::from_code(&code)?;
    pos += 2;

    let mut result = NabcGlyphDescriptor::new(basic_glyph);

    // Modifiers (S, G, M, -, >, ~) and variant numbers
    let mut modifiers: Vec<NabcGlyphModifier> = Vec::new();
    while pos < chars.len() {
        if pos + 1 < chars.len() {
            let lookahead: String = chars[pos..pos + 2].iter().collect();
            if NabcBasicGlyph::from_code(&lookahead).is_some() {
                break;
            }
        }
        let ch = chars[pos];
        match ch {
            'S' => {
                modifiers.push(NabcGlyphModifier::MarkModification);
                pos += 1;
            }
            'G' => {
                modifiers.push(NabcGlyphModifier::GroupingModification);
                pos += 1;
            }
            'M' => {
                modifiers.push(NabcGlyphModifier::MelodicModification);
                pos += 1;
            }
            '-' => {
                modifiers.push(NabcGlyphModifier::Episema);
                pos += 1;
            }
            '>' => {
                modifiers.push(NabcGlyphModifier::AugmentiveLiquescence);
                pos += 1;
            }
            '~' => {
                modifiers.push(NabcGlyphModifier::DiminutiveLiquescence);
                pos += 1;
            }
            '1'..='9' => pos += 1,
            _ => break,
        }
    }
    if !modifiers.is_empty() {
        result.modifiers = Some(modifiers);
    }

    // Pitch descriptor: h<pitch>
    if pos < chars.len() && chars[pos] == 'h' {
        pos += 1;
        if pos < chars.len() && matches!(chars[pos], 'a'..='n' | 'p') {
            result.pitch = Some(chars[pos]);
            pos += 1;
        }
    }

    // Significant letters (ls/lt prefix), repeated
    let mut sig_letters: Vec<NabcSignificantLetter> = Vec::new();
    loop {
        if pos + 1 >= chars.len() {
            break;
        }
        let lookahead: String = chars[pos..pos + 2].iter().collect();
        if NabcBasicGlyph::from_code(&lookahead).is_some() {
            break;
        }
        if chars[pos] != 'l' {
            break;
        }
        let nxt = chars[pos + 1];
        if nxt != 's' && nxt != 't' {
            break;
        }
        let letter_pos = position.map(|p| Position::new(p.line, p.character + pos));
        let remaining: String = slice_str(chars, pos, chars.len());
        match parse_significant_letter(&remaining, letter_pos) {
            Some((letter, length)) => {
                sig_letters.push(letter);
                pos += length;
            }
            None => break,
        }
    }
    if !sig_letters.is_empty() {
        result.significant_letters = Some(sig_letters);
    }

    if let Some(p) = position {
        result.range = Some(Range::new(
            p,
            Position::new(p.line, p.character + (pos - start)),
        ));
    }

    Some((result, pos))
}

fn parse_subpunctis_prepunctis(
    nabc: &str,
    start: Option<Position>,
) -> Option<(NabcGlyphDescriptor, usize)> {
    if nabc.len() < 2 {
        return None;
    }
    let chars: Vec<char> = nabc.chars().collect();
    let kind = &nabc[0..2];
    if kind != "su" && kind != "pp" {
        return None;
    }
    let mut pos = 2;
    let modifier = if pos < chars.len()
        && matches!(
            chars[pos],
            't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'n' | 'q' | 'z'
        ) {
        let m = chars[pos];
        pos += 1;
        Some(m)
    } else {
        None
    };
    if pos >= chars.len() {
        return None;
    }
    let count_ch = chars[pos];
    if !matches!(count_ch, '1'..='9') {
        return None;
    }
    let count = count_ch.to_digit(10)? as u8;
    pos += 1;

    let mut descriptor = NabcGlyphDescriptor::new(NabcBasicGlyph::Punctum);
    let range = start.map(|s| Range::new(s, Position::new(s.line, s.character + pos)));

    if kind == "su" {
        descriptor.subpunctis = Some(NabcSubpunctis {
            count,
            modifier,
            range,
        });
    } else {
        descriptor.prepunctis = Some(NabcPrepunctis {
            count,
            modifier,
            range,
        });
    }

    Some((descriptor, pos))
}

fn parse_significant_letter(
    nabc: &str,
    start: Option<Position>,
) -> Option<(NabcSignificantLetter, usize)> {
    let chars: Vec<char> = nabc.chars().collect();
    if chars.len() < 4 {
        return None;
    }
    if chars[0] != 'l' {
        return None;
    }
    let kind = match chars[1] {
        's' => NabcLetterKind::Significant,
        't' => NabcLetterKind::Tironian,
        _ => return None,
    };

    let mut pos = 2;
    while pos < chars.len() && !matches!(chars[pos], '1'..='9') {
        pos += 1;
    }
    if pos <= 2 || pos >= chars.len() || !matches!(chars[pos], '1'..='9') {
        return None;
    }
    let position_digit = chars[pos].to_digit(10)? as u8;
    let code: String = chars[2..pos].iter().collect();
    if code.is_empty() {
        return None;
    }
    let total_length = pos + 1;
    let range = start.map(|s| Range::new(s, Position::new(s.line, s.character + total_length)));
    Some((
        NabcSignificantLetter {
            kind,
            code,
            position: position_digit,
            range,
        },
        total_length,
    ))
}

/// Validate a parsed NABC descriptor.
pub fn validate_nabc_descriptor(descriptor: &NabcGlyphDescriptor) -> Vec<String> {
    let mut errors = Vec::new();

    if let Some(p) = descriptor.pitch {
        if !matches!(p, 'a'..='n' | 'p') {
            errors.push(format!("Invalid NABC pitch descriptor: {p}"));
        }
    }

    if let Some(mods) = &descriptor.modifiers {
        let has_aug = mods.contains(&NabcGlyphModifier::AugmentiveLiquescence);
        let has_dim = mods.contains(&NabcGlyphModifier::DiminutiveLiquescence);
        if has_aug && has_dim {
            errors.push(
                "NABC descriptor cannot have both augmentive and diminutive liquescence".into(),
            );
        }
    }

    errors
}

pub fn get_all_nabc_glyph_codes() -> Vec<&'static str> {
    NabcBasicGlyph::all().iter().map(|g| g.code()).collect()
}

pub fn is_valid_nabc_glyph(code: &str) -> bool {
    NabcBasicGlyph::from_code(code).is_some()
}
