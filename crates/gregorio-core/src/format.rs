//! GABC source formatter.
//!
//! Entry point: [`format_gabc_text`].
//!
//! The formatter:
//! 1. Preserves the header section verbatim (normalizes trailing whitespace per line).
//! 2. Tokenizes the notation body into a flat sequence of [`Token`]s.
//! 3. Packs tokens into lines with a greedy algorithm respecting [`FormatOptions::max_line_width`].
//! 4. Optionally inserts a blank line after each clef token ([`FormatOptions::break_after_clef`]).
//! 5. Optionally inserts a blank line after each bar token ([`FormatOptions::break_after_bar`]).
//! 6. Optionally places each word on its own line ([`FormatOptions::word_per_line`]),
//!    which is useful for long GABC+NABC code (e.g. Graduale Triplex with two NABC lines).

use std::str::FromStr;

/// Controls the "one word per line" layout mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordPerLine {
    /// Enable automatically when the `nabc-lines` header is 2 or greater, or when
    /// at least [`LONG_WORD_RATIO`] of the words are wider than half of
    /// [`FormatOptions::max_line_width`].
    #[default]
    Auto,
    /// Always place each word on its own line.
    Always,
    /// Never use word-per-line mode (greedy packing only).
    Never,
}

impl FromStr for WordPerLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            other => Err(format!(
                "invalid word-per-line mode '{other}' (expected auto, always or never)"
            )),
        }
    }
}

/// Minimum fraction of long words (wider than half the maximum line width)
/// that activates word-per-line mode under [`WordPerLine::Auto`].
pub const LONG_WORD_RATIO: f64 = 0.25;

/// Options controlling how the formatter behaves.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Maximum output line width in characters. Default: 80.
    pub max_line_width: usize,
    /// Insert a blank line after each clef token. Default: true.
    pub break_after_clef: bool,
    /// Insert a blank line after each bar token. Default: true.
    pub break_after_bar: bool,
    /// Place each word on its own line. Default: [`WordPerLine::Auto`].
    pub word_per_line: WordPerLine,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            max_line_width: 80,
            break_after_clef: true,
            break_after_bar: true,
            word_per_line: WordPerLine::Auto,
        }
    }
}

// ── Token types ───────────────────────────────────────────────────────────────

/// A single formatting unit in the GABC notation body.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    /// A syllable + note group, e.g. `KY(fgh)`.
    Syllable { text: String, notes: String },
    /// A standalone note group with no preceding syllable text, e.g. `(c4)` or `(,)`.
    StandaloneGroup { notes: String },
    /// A styled-text span, e.g. `<i>bis</i>` or `<sp>V/</sp>`.
    StyledText { raw: String },
    /// Special markers: `*`, `**`, `+`.
    Special { raw: String },
    /// A line comment `% ...` (content includes the `%`).
    Comment { text: String },
}

impl Token {
    /// Returns the exact string that should appear in the output for this token.
    fn display(&self) -> String {
        match self {
            Token::Syllable { text, notes } => format!("{text}({notes})"),
            Token::StandaloneGroup { notes } => format!("({notes})"),
            Token::StyledText { raw } | Token::Special { raw } => raw.clone(),
            Token::Comment { text } => text.clone(),
        }
    }

    /// Returns `true` when this token represents a clef change, i.e.
    /// `(c1)`…`(cb4)`, `(f1)`…`(f4)`.
    fn is_clef(&self) -> bool {
        let notes = match self {
            Token::StandaloneGroup { notes } => notes.as_str(),
            _ => return false,
        };
        is_clef_notes(notes)
    }

    /// Returns `true` when this token represents a bar (divisio).
    fn is_bar(&self) -> bool {
        let notes = match self {
            Token::StandaloneGroup { notes } => notes.as_str(),
            _ => return false,
        };
        is_bar_notes(notes)
    }

    /// Returns `true` for the `*` / `**` markers, which are kept on the line of
    /// the preceding word in word-per-line mode.
    fn is_asterisk(&self) -> bool {
        matches!(self, Token::Special { raw } if raw == "*" || raw == "**")
    }
}

fn is_clef_notes(notes: &str) -> bool {
    // Matches: c1–c4, cb1–cb4, f1–f4
    let s = notes.trim();
    let s = s
        .strip_prefix('c')
        .or_else(|| s.strip_prefix('f'))
        .unwrap_or("");
    let s = s.strip_prefix('b').unwrap_or(s);
    matches!(s, "1" | "2" | "3" | "4")
}

fn is_bar_notes(notes: &str) -> bool {
    // Bar types defined in GABC spec (BarType enum)
    matches!(
        notes.trim(),
        "`" | "," | ";" | ":" | "::" | ":?:" | "'" | "!bar!" | ",0" | ",1"
    )
}

// ── Tokenizer ─────────────────────────────────────────────────────────────────

/// Tokenize the notation body (everything after `%%`), recording for each
/// token whether it was immediately preceded by whitespace in the source.
///
/// The boolean in each pair is `true` when there was at least one whitespace
/// character between the previous token and this one (or when this is the first
/// token). The packer uses this to decide whether a space should be placed
/// between two adjacent tokens: tokens that had no whitespace between them in
/// the source are kept adjacent in the output.
fn tokenize_with_spacing(notation: &str) -> Vec<(Token, bool)> {
    let mut entries: Vec<(Token, bool)> = Vec::new();
    let chars: Vec<char> = notation.chars().collect();
    let len = chars.len();
    let mut i = 0;
    // True when whitespace has been seen since the last emitted token.
    // Starts as `false`; the first token therefore carries `preceded = false`,
    // which is irrelevant because the packer treats an empty `current` as a
    // no-space case regardless.
    let mut had_space = false;

    while i < len {
        if chars[i].is_ascii_whitespace() {
            had_space = true;
            i += 1;
            continue;
        }

        // Capture and reset the spacing flag before consuming each token.
        let preceded = had_space;
        had_space = false;

        // Line comment: % … until end-of-line.
        if chars[i] == '%' {
            let start = i;
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            entries.push((
                Token::Comment {
                    text: chars[start..i].iter().collect(),
                },
                preceded,
            ));
            continue;
        }

        // Styled text or XML-like tag: <…>…</…>
        // If immediately followed by `(notes)` it is a syllable whose text is
        // the styled span (e.g. `<i>bis</i>(::)`).
        if chars[i] == '<' {
            let raw = consume_styled_text(&chars, &mut i);
            if i < len && chars[i] == '(' {
                let notes = consume_parens(&chars, &mut i);
                entries.push((Token::Syllable { text: raw, notes }, preceded));
            } else {
                entries.push((Token::StyledText { raw }, preceded));
            }
            continue;
        }

        // Special single-character markers: * ** +
        if chars[i] == '*' {
            if i + 1 < len && chars[i + 1] == '*' {
                entries.push((Token::Special { raw: "**".into() }, preceded));
                i += 2;
            } else {
                entries.push((Token::Special { raw: "*".into() }, preceded));
                i += 1;
            }
            continue;
        }
        if chars[i] == '+' {
            entries.push((Token::Special { raw: "+".into() }, preceded));
            i += 1;
            continue;
        }

        // Standalone note group: starts directly with `(`
        if chars[i] == '(' {
            let notes = consume_parens(&chars, &mut i);
            entries.push((Token::StandaloneGroup { notes }, preceded));
            continue;
        }

        // Syllable text followed by `(notes)`
        let text = consume_syllable_text(&chars, &mut i);
        if text.is_empty() {
            // Safety: skip unrecognised byte.
            i += 1;
            continue;
        }
        if i < len && chars[i] == '(' {
            let notes = consume_parens(&chars, &mut i);
            entries.push((Token::Syllable { text, notes }, preceded));
        } else {
            // Text without a following note group — treat as special/unknown token.
            entries.push((Token::Special { raw: text }, preceded));
        }
    }

    entries
}

/// Tokenize the notation body, discarding whitespace-spacing information.
/// Available only in test builds; use `tokenize_with_spacing` in production code.
#[cfg(test)]
fn tokenize(notation: &str) -> Vec<Token> {
    tokenize_with_spacing(notation)
        .into_iter()
        .map(|(token, _)| token)
        .collect()
}

/// Consume characters that belong to a syllable text (everything that is not
/// whitespace, `(`, `<`, `*`, `+`, `%`).
fn consume_syllable_text(chars: &[char], i: &mut usize) -> String {
    let mut out = String::new();
    while *i < chars.len() {
        let ch = chars[*i];
        if ch.is_ascii_whitespace() || ch == '(' || ch == '<' || ch == '*' || ch == '+' || ch == '%'
        {
            break;
        }
        out.push(ch);
        *i += 1;
    }
    out
}

/// Consume a balanced parenthesised expression `(…)`.  Returns the content
/// between the outer `(` and `)` (the delimiters themselves are consumed but
/// not included in the return value).
fn consume_parens(chars: &[char], i: &mut usize) -> String {
    debug_assert_eq!(chars[*i], '(');
    *i += 1; // skip opening '('
    let mut depth = 1u32;
    let mut out = String::new();
    while *i < chars.len() {
        let ch = chars[*i];
        *i += 1;
        match ch {
            '(' => {
                depth += 1;
                out.push(ch);
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Consume a styled-text span starting with `<`.  Handles simple tags like
/// `<i>text</i>` and self-closing `<br/>`.  Returns the full raw text.
fn consume_styled_text(chars: &[char], i: &mut usize) -> String {
    let start = *i;
    // Read the opening tag name so we know when to stop.
    debug_assert_eq!(chars[*i], '<');
    *i += 1;
    // Collect tag name (letters, digits, /, space).
    let mut tag_name = String::new();
    while *i < chars.len() && chars[*i] != '>' && !chars[*i].is_ascii_whitespace() {
        if chars[*i] == '/' {
            // Self-closing or closing tag.
        } else {
            tag_name.push(chars[*i]);
        }
        *i += 1;
    }
    // Consume rest of opening tag up to `>`.
    while *i < chars.len() && chars[*i] != '>' {
        *i += 1;
    }
    if *i < chars.len() {
        *i += 1; // consume `>`
    }

    if tag_name.starts_with('/') || tag_name.is_empty() {
        // Closing or empty tag — nothing more to consume.
        return chars[start..*i].iter().collect();
    }

    // Consume until the matching closing tag `</tag_name>`.
    let close = format!("</{tag_name}>");
    let close_chars: Vec<char> = close.chars().collect();
    while *i < chars.len() {
        if chars[*i..].starts_with(&close_chars) {
            *i += close_chars.len();
            break;
        }
        *i += 1;
    }
    chars[start..*i].iter().collect()
}

// ── Line packer ───────────────────────────────────────────────────────────────

/// Returns the total char count of the token at `idx` plus all immediately
/// following adjacent (not preceded-by-space) tokens.  Used for lookahead so
/// the wrap decision accounts for the full indivisible group.
fn adjacent_group_chars(entries: &[(Token, bool)], idx: usize) -> usize {
    let mut total = entries[idx].0.display().chars().count();
    let mut j = idx + 1;
    while j < entries.len() && !entries[j].1 {
        total += entries[j].0.display().chars().count();
        j += 1;
    }
    total
}

/// Pack a token stream into output lines respecting `max_line_width`.
///
/// Rules applied here:
/// - Space-separated tokens are placed on the current line or wrapped to the
///   next line.  The wrap decision uses the **full adjacent group** starting at
///   the token (the token itself plus all immediately following non-spaced
///   tokens) so that adjacent syllables are never split across a wrap boundary.
/// - Adjacent tokens (no whitespace in the source) are always kept together;
///   they are appended unconditionally after their leading space-separated
///   anchor has been placed.
/// - `break_after_clef` / `break_after_bar`: after the respective token a blank
///   line is inserted.  If placing the bar/clef on the current line would exceed
///   the width limit it wraps to the next line like any other token.
/// - Comment tokens always start on their own line.
/// - `word_mode`: every space-separated word starts a new line, except bars and
///   `*` / `**` markers, which stay at the end of the preceding word's line.
fn pack(entries: &[(Token, bool)], opts: &FormatOptions, word_mode: bool) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    // Inline emit to avoid borrow-checker friction with an explicit closure.
    macro_rules! emit {
        () => {{
            lines.push(current.trim_end().to_string());
            current.clear();
        }};
    }

    let mut i = 0;
    while i < entries.len() {
        let (token, preceded_by_space) = &entries[i];

        // Comments always go on their own line.
        if let Token::Comment { .. } = token {
            if !current.trim().is_empty() {
                emit!();
            }
            lines.push(token.display());
            i += 1;
            continue;
        }

        let display = token.display();

        if current.trim().is_empty() {
            // First token on this line: just place it, no leading space.
            current.push_str(&display);
        } else if *preceded_by_space && word_mode && !token.is_bar() && !token.is_asterisk() {
            // Word-per-line mode: each word starts its own line.
            emit!();
            current.push_str(&display);
        } else if *preceded_by_space && word_mode {
            // Word-per-line mode: bars and `*` / `**` stay attached to the
            // preceding word, even when the line exceeds the width limit.
            current.push(' ');
            current.push_str(&display);
        } else if *preceded_by_space {
            // Space-separated token: check whether the full adjacent group
            // (this token + all immediately following non-spaced tokens) fits
            // on the current line before committing.
            let group = adjacent_group_chars(entries, i);

            // Lookahead: if the next space-separated token after this group is a
            // bar, extend the measure to include it.  This prevents a bare bar
            // from being pushed to the start of the next line — the anchor token
            // wraps with it instead.  Only extend when anchor+bar could actually
            // fit together on a fresh line (otherwise fall back to plain `group`).
            let mut j = i + 1;
            while j < entries.len() && !entries[j].1 {
                j += 1;
            }
            let effective = if j < entries.len() && entries[j].0.is_bar() {
                let bar_len = entries[j].0.display().chars().count();
                let extended = group + 1 + bar_len;
                if extended <= opts.max_line_width {
                    extended
                } else {
                    group
                }
            } else {
                group
            };

            if current.chars().count() + 1 + effective > opts.max_line_width {
                emit!();
                current.push_str(&display);
            } else {
                current.push(' ');
                current.push_str(&display);
            }
        } else {
            // Adjacent token: keep together with the preceding token.
            current.push_str(&display);
        }

        // Apply break-after rules *after* placing the token.
        let needs_blank =
            (opts.break_after_clef && token.is_clef()) || (opts.break_after_bar && token.is_bar());
        if needs_blank {
            emit!();
            lines.push(String::new()); // blank separator line
        }

        i += 1;
    }

    if !current.trim().is_empty() {
        lines.push(current.trim_end().to_string());
    }

    lines
}

// ── Word-per-line detection ───────────────────────────────────────────────────

/// Returns the value of the `nabc-lines` header (case-insensitive), or 0 when
/// it is absent or invalid.
fn header_nabc_lines(header: &str) -> usize {
    header
        .lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with('%'))
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            if !key.trim().eq_ignore_ascii_case("nabc-lines") {
                return None;
            }
            let value = value.split(';').next().unwrap_or("").trim();
            value.parse::<usize>().ok()
        })
        .unwrap_or(0)
}

/// Decide whether word-per-line mode applies to this document.
fn use_word_per_line(header: &str, entries: &[(Token, bool)], opts: &FormatOptions) -> bool {
    match opts.word_per_line {
        WordPerLine::Always => return true,
        WordPerLine::Never => return false,
        WordPerLine::Auto => {}
    }
    if header_nabc_lines(header) >= 2 {
        return true;
    }

    let threshold = opts.max_line_width / 2;
    let mut words = 0usize;
    let mut long_words = 0usize;
    for (idx, (token, preceded)) in entries.iter().enumerate() {
        // A word starts at the first token or at any space-separated token.
        if idx > 0 && !*preceded {
            continue;
        }
        if token.is_bar() || token.is_clef() || matches!(token, Token::Comment { .. }) {
            continue;
        }
        words += 1;
        if adjacent_group_chars(entries, idx) > threshold {
            long_words += 1;
        }
    }
    words > 0 && (long_words as f64) / (words as f64) >= LONG_WORD_RATIO
}

// ── Header normalizer ─────────────────────────────────────────────────────────

/// Preserve the header section with only trailing-whitespace normalization.
/// Returns (normalized_header_text, rest_after_separator).
///
/// The `%%` separator line is included in the returned header text.
fn split_and_normalize_header(text: &str) -> (String, &str) {
    // Find `%%` on its own (possibly with surrounding whitespace).
    if let Some(pos) = find_separator(text) {
        let (header_raw, rest) = text.split_at(pos);
        // rest starts with `%%`; consume the separator line.
        let after_sep = &rest[2..]; // skip `%%`
        let after_sep = after_sep.strip_prefix('\n').unwrap_or(after_sep);

        let header_normalized = normalize_trailing_whitespace(header_raw);
        let header_out = format!("{header_normalized}%%");
        (header_out, after_sep)
    } else {
        // No `%%` found — treat the entire text as notation (no headers).
        (String::new(), text)
    }
}

/// Find the byte offset of `%%` in `text`.
fn find_separator(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'%' && bytes[i + 1] == b'%' {
            // Must not be inside a `%single-comment` (those have only one `%`).
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Strip trailing whitespace from each line, preserving line endings.
fn normalize_trailing_whitespace(text: &str) -> String {
    // `str::lines()` correctly omits the trailing empty "line" that `split('\n')`
    // would produce for strings ending with '\n'.
    let body: String = text
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    if text.ends_with('\n') {
        body + "\n"
    } else {
        body
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Format a complete GABC document string according to `opts`.
///
/// # Guarantees
/// - Header section (before `%%`) is preserved verbatim except trailing whitespace per line.
/// - The `%%` separator is always emitted on its own line.
/// - The notation body is re-flowed: existing inter-syllable whitespace (including
///   source newlines) is replaced by the formatter's output.
/// - The output always ends with exactly one newline character.
pub fn format_gabc_text(text: &str, opts: &FormatOptions) -> String {
    let (header, notation) = split_and_normalize_header(text);

    let tokens = tokenize_with_spacing(notation);
    let word_mode = use_word_per_line(&header, &tokens, opts);
    let packed = pack(&tokens, opts, word_mode);

    let mut out = String::new();

    if !header.is_empty() {
        out.push_str(&header);
        out.push('\n');
    }

    for (idx, line) in packed.iter().enumerate() {
        out.push_str(line);
        if idx + 1 < packed.len() || !line.is_empty() {
            out.push('\n');
        }
    }

    // Ensure exactly one trailing newline.
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }

    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(max: usize) -> FormatOptions {
        FormatOptions {
            max_line_width: max,
            ..Default::default()
        }
    }

    // ── Tokenizer ──────────────────────────────────────────────────────────

    #[test]
    fn tokenize_syllable() {
        let tokens = tokenize("KY(f)ri(gh)e(h)");
        assert_eq!(
            tokens,
            vec![
                Token::Syllable {
                    text: "KY".into(),
                    notes: "f".into()
                },
                Token::Syllable {
                    text: "ri".into(),
                    notes: "gh".into()
                },
                Token::Syllable {
                    text: "e".into(),
                    notes: "h".into()
                },
            ]
        );
    }

    #[test]
    fn tokenize_standalone_clef_and_bar() {
        let tokens = tokenize("(c4) Foo(g) (,)");
        assert_eq!(tokens.len(), 3);
        assert!(tokens[0].is_clef());
        assert!(tokens[2].is_bar());
    }

    #[test]
    fn tokenize_styled_text() {
        // Styled span immediately followed by `(notes)` → Syllable
        let tokens = tokenize("<i>bis</i>(::)");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Syllable { text, notes }
                if text == "<i>bis</i>" && notes == "::"));
        // Styled span with no following note group → StyledText
        let tokens2 = tokenize("<i>bis</i>");
        assert_eq!(tokens2.len(), 1);
        assert!(matches!(&tokens2[0], Token::StyledText { raw } if raw == "<i>bis</i>"));
    }

    #[test]
    fn tokenize_special_markers() {
        let tokens = tokenize("*(,) **(::) +(g)");
        assert_eq!(tokens.len(), 6);
        assert!(matches!(&tokens[0], Token::Special { raw } if raw == "*"));
        assert!(matches!(&tokens[2], Token::Special { raw } if raw == "**"));
        assert!(matches!(&tokens[4], Token::Special { raw } if raw == "+"));
    }

    // ── is_clef / is_bar helpers ───────────────────────────────────────────

    #[test]
    fn clef_detection() {
        assert!(is_clef_notes("c4"));
        assert!(is_clef_notes("cb3"));
        assert!(is_clef_notes("f1"));
        assert!(!is_clef_notes(","));
        assert!(!is_clef_notes("::"));
    }

    #[test]
    fn bar_detection() {
        assert!(is_bar_notes(","));
        assert!(is_bar_notes(";"));
        assert!(is_bar_notes(":"));
        assert!(is_bar_notes("::"));
        assert!(!is_bar_notes("c4"));
        assert!(!is_bar_notes("gh"));
    }

    // ── Line packing ───────────────────────────────────────────────────────

    #[test]
    fn pack_fits_on_one_line() {
        let result = format_gabc_text("%%\nA(g) B(h) C(i)\n", &opts(80));
        assert_eq!(result.trim(), "%%\nA(g) B(h) C(i)".trim());
    }

    #[test]
    fn pack_wraps_at_limit() {
        // Each token is ~5 chars; limit of 10 means at most 2 per line.
        let input = "%%\nAB(gh) CD(hi) EF(ij)\n";
        let result = format_gabc_text(input, &opts(12));
        let notation: Vec<&str> = result
            .trim()
            .split('\n')
            .skip(1) // skip %%
            .collect();
        assert!(notation.len() >= 2, "expected wrapping: {result:?}");
        for line in &notation {
            assert!(line.chars().count() <= 12, "line too long: {line:?}");
        }
    }

    #[test]
    fn reflows_existing_linebreaks() {
        // Input has artificial linebreaks; formatter should re-pack.
        let input = "%%\nA(g)\nB(h)\nC(i)\n";
        let result = format_gabc_text(input, &opts(80));
        let notation_lines: Vec<&str> = result
            .trim()
            .split('\n')
            .skip(1)
            .filter(|l| !l.is_empty())
            .collect();
        assert_eq!(notation_lines.len(), 1, "should be on one line: {result:?}");
    }

    // ── break_after_clef ──────────────────────────────────────────────────

    #[test]
    fn break_after_clef_inserts_blank_line() {
        let input = "%%\n(c4) Foo(g) Bar(h)\n";
        let result = format_gabc_text(
            input,
            &FormatOptions {
                max_line_width: 80,
                break_after_clef: true,
                break_after_bar: false,
                ..Default::default()
            },
        );
        // Clef must be on its own line followed by a blank line before the music.
        assert!(
            result.contains("(c4)\n\nFoo(g)"),
            "expected blank line after clef:\n{result}"
        );
    }

    // ── break_after_bar ───────────────────────────────────────────────────

    #[test]
    fn break_after_bar_inserts_blank_line() {
        let input = "%%\nFoo(g) (,) Bar(h)\n";
        let result = format_gabc_text(
            input,
            &FormatOptions {
                max_line_width: 80,
                break_after_clef: false,
                break_after_bar: true,
                ..Default::default()
            },
        );
        assert!(
            result.contains("(,)\n\nBar(h)"),
            "expected blank line after bar:\n{result}"
        );
    }

    // ── Whitespace-preserving packer ──────────────────────────────────────

    #[test]
    fn adjacent_syllables_no_space_preserved() {
        // Syllables with no whitespace between them in the source must remain
        // adjacent in the output — the formatter must not insert a space.
        let result = format_gabc_text("%%\nfoo()bar()\n", &opts(80));
        assert!(
            result.contains("foo()bar()"),
            "formatter must not add space between adjacent syllables:\n{result}"
        );
    }

    #[test]
    fn spaced_syllables_keep_space() {
        // Syllables separated by whitespace must remain separated.
        let result = format_gabc_text("%%\nfoo(g) bar(h)\n", &opts(80));
        assert!(
            result.contains("foo(g) bar(h)"),
            "formatter must preserve space between space-separated syllables:\n{result}"
        );
    }

    #[test]
    fn bar_triggers_blank_line_and_respects_width_limit() {
        // With a very tight width limit, the bar must trigger a blank-line break
        // AND no output line must exceed the limit.  When the bar doesn't fit on
        // the preceding line it may legitimately be placed at the start of the
        // next line — the important invariant is (;)\n\n (blank line after bar).
        let input = "%%\nA(g) B(h) C(i) D(j) (;) E(f) F(g)\n";
        let result = format_gabc_text(
            input,
            &FormatOptions {
                max_line_width: 20,
                break_after_clef: false,
                break_after_bar: true,
                ..Default::default()
            },
        );
        assert!(
            result.contains("(;)\n\n"),
            "bar must be followed by blank line:\n{result}"
        );
        for line in result.lines() {
            assert!(
                line.chars().count() <= 20,
                "line exceeds max_line_width=20: {line:?}"
            );
        }
    }

    #[test]
    fn wrap_before_anchor_when_bar_would_start_next_line() {
        // When placing a token would push the following bar to the start of the
        // next line, the token itself should wrap instead so the bar stays at
        // the end of the line with its anchor.
        //   width=20: "A(g) B(h) C(i) D(j) (;)" — D(j) + (;) = 4+1+3=8; placing
        //   D(j) on "A(g) B(h) C(i)" (14) → 14+1+8=23 > 20, so D(j) wraps.
        //   Result must contain "D(j) (;)" on one line, not "(;)" alone.
        let input = "%%\nA(g) B(h) C(i) D(j) (;) E(f)\n";
        let result = format_gabc_text(
            input,
            &FormatOptions {
                max_line_width: 20,
                break_after_clef: false,
                break_after_bar: true,
                ..Default::default()
            },
        );
        assert!(
            result.contains("D(j) (;)\n\n"),
            "anchor must wrap with bar, not leave bar at start of line:\n{result}"
        );
        for line in result.lines() {
            assert!(
                line.chars().count() <= 20,
                "line exceeds max_line_width=20: {line:?}"
            );
        }
    }

    // ── Header preservation ───────────────────────────────────────────────

    #[test]
    fn header_preserved() {
        let input = "name: Test;\nmode: 8;\n%%\nA(g)\n";
        let result = format_gabc_text(input, &opts(80));
        assert!(result.starts_with("name: Test;\nmode: 8;\n%%\n"));
    }

    #[test]
    fn trailing_whitespace_stripped_from_header() {
        let input = "name: Test;   \n%%\nA(g)\n";
        let result = format_gabc_text(input, &opts(80));
        assert!(result.starts_with("name: Test;\n%%\n"));
    }

    // ── Trailing newline ──────────────────────────────────────────────────

    #[test]
    fn output_ends_with_single_newline() {
        let result = format_gabc_text("%%\nA(g)\n", &opts(80));
        assert!(result.ends_with('\n'));
        assert!(!result.ends_with("\n\n"));
    }

    // ── word_per_line ─────────────────────────────────────────────────────

    fn word_opts(mode: WordPerLine) -> FormatOptions {
        FormatOptions {
            word_per_line: mode,
            ..Default::default()
        }
    }

    #[test]
    fn word_per_line_mode_parses_from_str() {
        assert_eq!("auto".parse::<WordPerLine>(), Ok(WordPerLine::Auto));
        assert_eq!("Always".parse::<WordPerLine>(), Ok(WordPerLine::Always));
        assert_eq!(" never ".parse::<WordPerLine>(), Ok(WordPerLine::Never));
        assert!("sometimes".parse::<WordPerLine>().is_err());
    }

    #[test]
    fn word_per_line_always_one_word_per_line_with_bar_attached() {
        let result = format_gabc_text("%%\nA(g) B(h) (,) C(i)\n", &word_opts(WordPerLine::Always));
        assert_eq!(result, "%%\nA(g)\nB(h) (,)\n\nC(i)\n");
    }

    #[test]
    fn word_per_line_keeps_asterisk_and_adjacent_syllables() {
        let result = format_gabc_text(
            "%%\nDi(g)le(g)xi(h) *() ju(g)sti(h)\n",
            &word_opts(WordPerLine::Always),
        );
        assert_eq!(result, "%%\nDi(g)le(g)xi(h) *()\nju(g)sti(h)\n");
    }

    #[test]
    fn word_per_line_keeps_comments_on_own_line() {
        let result = format_gabc_text("%%\nA(g) % note\nB(h)\n", &word_opts(WordPerLine::Always));
        assert_eq!(result, "%%\nA(g)\n% note\nB(h)\n");
    }

    #[test]
    fn word_per_line_bar_stays_attached_beyond_width() {
        let opts = FormatOptions {
            max_line_width: 10,
            word_per_line: WordPerLine::Always,
            ..Default::default()
        };
        let result = format_gabc_text("%%\nLongword(ghijk) (;) B(h)\n", &opts);
        assert!(
            result.contains("Longword(ghijk) (;)\n\nB(h)"),
            "bar must stay attached:\n{result}"
        );
    }

    #[test]
    fn word_per_line_never_ignores_nabc_header() {
        let input = "nabc-lines:2;\n%%\nA(g|vi|vi) B(h|vi|vi)\n";
        let result = format_gabc_text(input, &word_opts(WordPerLine::Never));
        assert!(result.ends_with("%%\nA(g|vi|vi) B(h|vi|vi)\n"), "{result}");
    }

    #[test]
    fn word_per_line_auto_activates_with_two_nabc_lines() {
        let input = "name:X;\nnabc-lines:2;\n%%\nA(g|vi|vi) B(h|vi|vi)\n";
        let result = format_gabc_text(input, &word_opts(WordPerLine::Auto));
        assert!(result.ends_with("%%\nA(g|vi|vi)\nB(h|vi|vi)\n"), "{result}");
    }

    #[test]
    fn word_per_line_auto_stays_off_with_one_nabc_line_and_short_words() {
        let input = "nabc-lines:1;\n%%\nA(g|vi) B(h|vi)\n";
        let result = format_gabc_text(input, &word_opts(WordPerLine::Auto));
        assert!(result.ends_with("%%\nA(g|vi) B(h|vi)\n"), "{result}");
    }

    #[test]
    fn word_per_line_auto_activates_on_long_words() {
        // width 20 → threshold 10; 1 of 4 words is longer than 10 chars (25%).
        let opts = FormatOptions {
            max_line_width: 20,
            word_per_line: WordPerLine::Auto,
            ..Default::default()
        };
        let result = format_gabc_text("%%\nA(g) B(h) C(i) Long(ghijkl)\n", &opts);
        assert!(
            result.ends_with("%%\nA(g)\nB(h)\nC(i)\nLong(ghijkl)\n"),
            "{result}"
        );
    }

    #[test]
    fn word_per_line_auto_stays_off_for_short_words() {
        let result = format_gabc_text(
            "%%\n(c4) A(g) B(h) C(i) (,) D(j)\n",
            &word_opts(WordPerLine::Auto),
        );
        assert!(result.contains("A(g) B(h) C(i) (,)"), "{result}");
    }

    #[test]
    fn word_per_line_is_idempotent() {
        let input =
            "nabc-lines:2;\n%%\n(c4) DI(g|un|ta)le(g|un|ta) *() ju(g|un|vi) (;) et(j|un|ta)\n";
        let opts = word_opts(WordPerLine::Auto);
        let once = format_gabc_text(input, &opts);
        let twice = format_gabc_text(&once, &opts);
        assert_eq!(once, twice);
    }

    #[test]
    fn nabc_lines_header_detection() {
        assert_eq!(header_nabc_lines("name:X;\nnabc-lines:2;\n%%"), 2);
        assert_eq!(header_nabc_lines("NABC-Lines : 3 ;\n"), 3);
        assert_eq!(header_nabc_lines("% nabc-lines:2;\nname:X;\n"), 0);
        assert_eq!(header_nabc_lines("name:X;\n"), 0);
    }
}
