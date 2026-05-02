//! Fallback GABC parser (pure Rust port of the TypeScript reference implementation).

use once_cell::sync::Lazy;
use regex::Regex;

use super::nabc::parse_nabc_snippets;
use super::types::*;

/// Cursor that tracks byte offset, 0-based line and 0-based character (UTF-16-ish columns
/// approximated using code points; consistent with the original TypeScript implementation).
pub struct GabcParser<'a> {
    text: &'a [u8],
    pos: usize,
    line: usize,
    character: usize,
    errors: Vec<ParseError>,
    comments: Vec<Comment>,
    nabc_lines: Option<u32>,
}

#[derive(Debug, Clone)]
struct Segment {
    is_nabc: bool,
    content: String,
    start: Position,
}

static HEADER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([A-Za-z0-9-]+):").unwrap());
static CLEF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(c|f)(b)?([1-4])").unwrap());

impl<'a> GabcParser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text: text.as_bytes(),
            pos: 0,
            line: 0,
            character: 0,
            errors: Vec::new(),
            comments: Vec::new(),
            nabc_lines: None,
        }
    }

    pub fn parse(mut self) -> ParsedDocument {
        let headers = self.parse_headers();
        if let Some(v) = headers.get("nabc-lines") {
            self.nabc_lines = v.trim().parse::<u32>().ok();
        }
        self.skip_whitespace_and_comments();
        let notation = self.parse_notation();
        ParsedDocument {
            headers,
            notation,
            comments: std::mem::take(&mut self.comments),
            errors: std::mem::take(&mut self.errors),
        }
    }

    // ---- Header parsing ----

    fn parse_headers(&mut self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.text.len() {
                break;
            }

            if self.peek_str(2) == "%%" {
                self.advance(2);
                break;
            }

            let remaining = self.remaining_str();
            if let Some(caps) = HEADER_RE.captures(remaining) {
                let full_len = caps.get(0).unwrap().as_str().len();
                let name = caps.get(1).unwrap().as_str().to_string();
                self.advance(full_len);

                let mut value = String::new();
                while self.pos < self.text.len() {
                    let ch = self.peek_char();
                    if ch == '%' {
                        self.parse_comment();
                        continue;
                    }
                    if ch == ';' {
                        self.advance(1);
                        if self.peek_char() == ';' {
                            self.advance(1);
                        }
                        break;
                    }
                    if ch == '\n' {
                        self.advance(1);
                        continue;
                    }
                    value.push(ch);
                    self.advance(1);
                }

                headers.insert(name, value.trim());
            } else {
                self.advance(1);
            }
        }

        headers
    }

    // ---- Notation parsing ----

    fn parse_notation(&mut self) -> NotationSection {
        let start = self.current_position();
        let mut syllables = Vec::new();

        while self.pos < self.text.len() {
            self.skip_whitespace_and_comments();
            if self.pos >= self.text.len() {
                break;
            }
            if let Some(s) = self.parse_syllable() {
                syllables.push(s);
            } else {
                self.advance(1);
            }
        }

        NotationSection {
            syllables,
            range: Range::new(start, self.current_position()),
        }
    }

    fn parse_syllable(&mut self) -> Option<Syllable> {
        let text_start = self.current_position();
        let mut text_with_styles = String::new();

        while self.pos < self.text.len() {
            let ch = self.peek_char();
            if ch == '(' || ch == '%' {
                break;
            }
            if ch == '\\' && self.pos + 1 < self.text.len() && self.text[self.pos + 1] == b'\n' {
                self.advance(2);
                continue;
            }
            text_with_styles.push(ch);
            self.advance(1);
        }

        let trimmed = text_with_styles.trim();
        let plain_text = remove_style_tags(trimmed);
        let text_end = self.current_position();

        if self.peek_char() != '(' {
            if !plain_text.is_empty() || !trimmed.is_empty() {
                let with_styles = if trimmed != plain_text {
                    Some(trimmed.to_string())
                } else {
                    None
                };
                return Some(Syllable {
                    text: if plain_text.is_empty() {
                        trimmed.to_string()
                    } else {
                        plain_text
                    },
                    text_with_styles: with_styles,
                    notes: Vec::new(),
                    range: Range::new(text_start, text_end),
                    clef: None,
                    bar: None,
                    line_break: None,
                });
            }
            return None;
        }

        self.advance(1); // '('
        let note_start = self.current_position();

        // Collect alternating GABC/NABC segments
        let mut segments: Vec<Segment> = Vec::new();
        let mut is_nabc = false;

        while self.pos < self.text.len() && self.peek_char() != ')' {
            if self.peek_char() == '|' {
                self.advance(1);
                if self.nabc_lines.is_some() {
                    is_nabc = true;
                } else {
                    is_nabc = !is_nabc;
                }
                continue;
            }
            let segment_start = self.current_position();
            let mut content = String::new();
            while self.pos < self.text.len() {
                let c = self.peek_char();
                if c == ')' || c == '|' {
                    break;
                }
                content.push(c);
                self.advance(1);
            }
            if !content.is_empty() {
                segments.push(Segment {
                    is_nabc,
                    content,
                    start: segment_start,
                });
            }
        }

        if self.peek_char() == ')' {
            self.advance(1);
        }

        let gabc_segments: Vec<&Segment> = segments.iter().filter(|s| !s.is_nabc).collect();
        let nabc_snippets: Vec<String> = segments
            .iter()
            .filter(|s| s.is_nabc)
            .map(|s| s.content.clone())
            .collect();

        let gabc_content: String = gabc_segments.iter().map(|s| s.content.as_str()).collect();

        // Build position map (one entry per char of gabc_content)
        let mut position_map: Vec<Position> = Vec::with_capacity(gabc_content.chars().count());
        for seg in &gabc_segments {
            for (i, _) in seg.content.chars().enumerate() {
                position_map.push(Position::new(seg.start.line, seg.start.character + i));
            }
        }

        let clef = parse_clef_with_position(&gabc_content, note_start);
        let bar = parse_bar_with_position(&gabc_content, note_start);

        let mut notes = Vec::new();
        if !gabc_content.trim().is_empty() {
            let group_end = self.current_position();
            if let Some(group) = parse_note_group(
                &gabc_content,
                &nabc_snippets,
                note_start,
                group_end,
                &position_map,
            ) {
                notes.push(group);
            }
        }

        let end = self.current_position();
        let with_styles = if trimmed != plain_text && !trimmed.is_empty() {
            Some(trimmed.to_string())
        } else {
            None
        };

        Some(Syllable {
            text: if plain_text.is_empty() {
                trimmed.to_string()
            } else {
                plain_text
            },
            text_with_styles: with_styles,
            notes,
            range: Range::new(text_start, end),
            clef,
            bar,
            line_break: None,
        })
    }

    // ---- Comments / whitespace ----

    fn parse_comment(&mut self) {
        if self.peek_char() != '%' {
            return;
        }
        let start = self.current_position();
        self.advance(1);
        let mut text = String::new();
        while self.pos < self.text.len() && self.peek_char() != '\n' {
            text.push(self.peek_char());
            self.advance(1);
        }
        let end = self.current_position();
        self.comments.push(Comment {
            text,
            range: Range::new(start, end),
        });
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.text.len() {
            let ch = self.peek_char();
            if ch == '%' && self.peek_str(2) == "%%" {
                break;
            }
            if ch == '%' {
                self.parse_comment();
                continue;
            }
            if ch.is_whitespace() {
                self.advance(1);
                continue;
            }
            break;
        }
    }

    // ---- Cursor helpers ----

    fn peek_char(&self) -> char {
        if self.pos >= self.text.len() {
            return '\0';
        }
        // Decode UTF-8 character; the column tracking matches TypeScript's per-codepoint counting.
        std::str::from_utf8(&self.text[self.pos..])
            .ok()
            .and_then(|s| s.chars().next())
            .unwrap_or('\0')
    }

    fn peek_str(&self, n: usize) -> &str {
        let end = (self.pos + n).min(self.text.len());
        std::str::from_utf8(&self.text[self.pos..end]).unwrap_or("")
    }

    fn remaining_str(&self) -> &str {
        std::str::from_utf8(&self.text[self.pos..]).unwrap_or("")
    }

    fn advance(&mut self, count: usize) {
        for _ in 0..count {
            if self.pos >= self.text.len() {
                break;
            }
            let ch = self.peek_char();
            let len = ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.character = 0;
            } else {
                self.character += 1;
            }
            self.pos += len;
        }
    }

    fn current_position(&self) -> Position {
        Position::new(self.line, self.character)
    }

    pub fn add_error(&mut self, message: impl Into<String>, range: Range, severity: Severity) {
        self.errors.push(ParseError::new(message, range, severity));
    }
}

// ---------- Free functions ----------

fn remove_style_tags(text: &str) -> String {
    static PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
        vec![
            Regex::new(r"<b>").unwrap(),
            Regex::new(r"</b>").unwrap(),
            Regex::new(r"<i>").unwrap(),
            Regex::new(r"</i>").unwrap(),
            Regex::new(r"<sc>").unwrap(),
            Regex::new(r"</sc>").unwrap(),
            Regex::new(r"<ul>").unwrap(),
            Regex::new(r"</ul>").unwrap(),
            Regex::new(r"<tt>").unwrap(),
            Regex::new(r"</tt>").unwrap(),
            Regex::new(r"<c>").unwrap(),
            Regex::new(r"</c>").unwrap(),
            Regex::new(r"<v>.*?</v>").unwrap(),
            Regex::new(r"<alt>.*?</alt>").unwrap(),
            Regex::new(r"</alt>").unwrap(),
        ]
    });
    let mut out = text.to_string();
    for re in PATTERNS.iter() {
        out = re.replace_all(&out, "").into_owned();
    }
    out
}

fn parse_clef_with_position(content: &str, base: Position) -> Option<Clef> {
    let caps = CLEF_RE.captures(content)?;
    let kind = match caps.get(1).unwrap().as_str() {
        "c" => ClefKind::C,
        "f" => ClefKind::F,
        _ => return None,
    };
    let has_flat = caps.get(2).is_some();
    let line: u8 = caps.get(3).unwrap().as_str().parse().ok()?;
    let len = caps.get(0).unwrap().as_str().len();
    Some(Clef {
        kind,
        line,
        has_flat,
        range: Range::new(base, Position::new(base.line, base.character + len)),
    })
}

fn parse_bar_with_position(content: &str, base: Position) -> Option<Bar> {
    let trimmed = content.trim();
    let kind = match trimmed {
        "`" | "`0" => BarType::Virgula,
        "," | ",0" => BarType::DivisioMinima,
        ";" => BarType::DivisioMinor,
        ":" => BarType::DivisioMaior,
        "::" => BarType::DivisioFinalis,
        _ => return None,
    };
    let bar_index = content.find(trimmed).unwrap_or(0);
    let start = Position::new(base.line, base.character + bar_index);
    let end = Position::new(base.line, base.character + bar_index + trimmed.len());
    Some(Bar {
        kind,
        range: Range::new(start, end),
    })
}

fn parse_attribute(text: &str, start: Position) -> Option<(GabcAttribute, usize)> {
    if !text.starts_with('[') {
        return None;
    }
    let close = text.find(']')?;
    let content = &text[1..close];
    let (name, value) = match content.find(':') {
        Some(i) => (
            content[..i].trim().to_string(),
            Some(content[i + 1..].trim().to_string()),
        ),
        None => (content.trim().to_string(), None),
    };
    let length = close + 1;
    let end = Position::new(start.line, start.character + length);
    Some((
        GabcAttribute {
            name,
            value,
            range: Range::new(start, end),
        },
        length,
    ))
}

fn position_at(map: &[Position], idx: usize, fallback_base: Position) -> Position {
    map.get(idx)
        .copied()
        .unwrap_or_else(|| Position::new(fallback_base.line, fallback_base.character + idx))
}

fn parse_note_group(
    gabc: &str,
    nabc: &[String],
    start: Position,
    end: Position,
    position_map: &[Position],
) -> Option<NoteGroup> {
    let chars: Vec<char> = gabc.chars().collect();
    let mut notes: Vec<Note> = Vec::new();
    let mut custos: Option<Custos> = None;
    let mut attributes: Vec<GabcAttribute> = Vec::new();

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        // Skip whitespace and connectors
        if matches!(ch, ' ' | '\t' | '\n' | '\r' | '/' | '`' | '!') {
            i += 1;
            continue;
        }

        // Custos z0
        if ch == 'z' && i + 1 < chars.len() && chars[i + 1] == '0' {
            custos = Some(Custos {
                kind: CustosKind::Auto,
                pitch: None,
                range: Range::new(
                    position_at(position_map, i, start),
                    position_at(position_map, i + 2, start),
                ),
            });
            i += 2;
            continue;
        }

        // Explicit custos +<pitch>
        if ch == '+' && i + 1 < chars.len() && matches!(chars[i + 1], 'a'..='n') {
            custos = Some(Custos {
                kind: CustosKind::Explicit,
                pitch: Some(chars[i + 1]),
                range: Range::new(
                    position_at(position_map, i, start),
                    position_at(position_map, i + 2, start),
                ),
            });
            i += 2;
            continue;
        }

        // Attribute [name(:value)?]
        if ch == '[' {
            let suffix: String = chars[i..].iter().collect();
            if let Some((attr, len)) = parse_attribute(&suffix, position_at(position_map, i, start))
            {
                attributes.push(attr);
                i += len;
                continue;
            }
        }

        // Pitch letters: a-n, p (case-insensitive)
        if matches!(ch, 'a'..='n' | 'A'..='N' | 'p' | 'P') {
            let note_start_idx = i;
            let note_start = position_at(position_map, i, start);
            let is_upper = ch.is_ascii_uppercase();
            let pitch = ch.to_ascii_lowercase();
            let mut shape = if is_upper {
                NoteShape::PunctumInclinatum
            } else {
                NoteShape::Punctum
            };
            let mut modifiers: Vec<NoteModifier> = Vec::new();
            i += 1;

            while i < chars.len() {
                let mod_ch = chars[i];

                // Punctum inclinatum leaning indicators (digits 0-2 after uppercase pitch)
                if is_upper && matches!(mod_ch, '0'..='2') {
                    i += 1;
                    continue;
                }

                match mod_ch {
                    'o' => {
                        shape = NoteShape::Oriscus;
                        i += 1;
                        if i < chars.len() && matches!(chars[i], '0' | '1') {
                            i += 1;
                        }
                    }
                    'O' => {
                        shape = NoteShape::Oriscus;
                        modifiers.push(NoteModifier::simple(ModifierType::OriscusScapus));
                        i += 1;
                        if i < chars.len() && matches!(chars[i], '0' | '1') {
                            i += 1;
                        }
                    }
                    'w' | 'W' => {
                        shape = NoteShape::Quilisma;
                        i += 1;
                    }
                    'v' => {
                        if i + 1 < chars.len() && chars[i + 1] == 'v' {
                            i += 2;
                            if i < chars.len() && chars[i] == 'v' {
                                i += 1;
                            }
                        } else {
                            shape = NoteShape::Virga;
                            i += 1;
                        }
                    }
                    'V' => {
                        shape = NoteShape::VirgaReversa;
                        i += 1;
                    }
                    's' => {
                        if i + 1 < chars.len() && chars[i + 1] == 's' {
                            i += 2;
                            if i < chars.len() && chars[i] == 's' {
                                i += 1;
                            }
                        } else {
                            shape = NoteShape::Stropha;
                            i += 1;
                        }
                    }
                    'r' => {
                        // r1..r8 = rhythmic sign; otherwise cavum modifier
                        if i + 1 < chars.len() && matches!(chars[i + 1], '1'..='8') {
                            i += 2;
                        } else {
                            shape = NoteShape::Cavum;
                            i += 1;
                            if i < chars.len() && chars[i].is_ascii_digit() {
                                i += 1;
                            }
                        }
                    }
                    'R' => {
                        shape = NoteShape::Cavum;
                        i += 1;
                    }
                    '=' => {
                        shape = NoteShape::Linea;
                        i += 1;
                    }
                    'q' => {
                        modifiers.push(NoteModifier::simple(ModifierType::Quadratum));
                        i += 1;
                    }
                    'x' => {
                        shape = NoteShape::Flat;
                        i += 1;
                        if i < chars.len() && chars[i] == '?' {
                            i += 1;
                        }
                    }
                    'X' => {
                        shape = NoteShape::Flat;
                        i += 1;
                    }
                    '#' => {
                        shape = NoteShape::Sharp;
                        i += 1;
                        if i < chars.len() && (chars[i] == '#' || chars[i] == '?') {
                            i += 1;
                        }
                    }
                    'y' => {
                        shape = NoteShape::Natural;
                        i += 1;
                        if i < chars.len() && chars[i] == '?' {
                            i += 1;
                        }
                    }
                    'Y' => {
                        shape = NoteShape::Natural;
                        i += 1;
                    }
                    '.' => {
                        modifiers.push(NoteModifier::simple(ModifierType::PunctumMora));
                        i += 1;
                        if i < chars.len() && chars[i] == '.' {
                            i += 1;
                        }
                    }
                    '_' => {
                        modifiers.push(NoteModifier::simple(ModifierType::HorizontalEpisema));
                        i += 1;
                        while i < chars.len() && matches!(chars[i], '0'..='5') {
                            i += 1;
                        }
                    }
                    '\'' => {
                        modifiers.push(NoteModifier::simple(ModifierType::VerticalEpisema));
                        i += 1;
                        if i < chars.len() && matches!(chars[i], '0' | '1') {
                            i += 1;
                        }
                    }
                    '-' => {
                        modifiers.push(NoteModifier::simple(ModifierType::InitioDebilis));
                        i += 1;
                    }
                    '~' | '<' | '>' => {
                        shape = NoteShape::Liquescent;
                        modifiers.push(NoteModifier::simple(ModifierType::Liquescent));
                        i += 1;
                    }
                    '@' => {
                        modifiers.push(NoteModifier::simple(ModifierType::Fusion));
                        i += 1;
                    }
                    _ => break,
                }
            }

            let note_end = Position::new(
                note_start.line,
                note_start.character + (i - note_start_idx),
            );
            notes.push(Note {
                pitch,
                shape,
                modifiers,
                range: Range::new(note_start, note_end),
                fusion: false,
            });
        } else {
            i += 1;
        }
    }

    let nabc_parsed = if !nabc.is_empty() {
        Some(parse_nabc_snippets(nabc, Some(start)))
    } else {
        None
    };

    Some(NoteGroup {
        gabc: gabc.to_string(),
        nabc: if nabc.is_empty() {
            None
        } else {
            Some(nabc.to_vec())
        },
        nabc_parsed,
        range: Range::new(start, end),
        notes,
        custos,
        attributes: if attributes.is_empty() {
            None
        } else {
            Some(attributes)
        },
    })
}
