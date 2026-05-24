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

/// Options controlling how the formatter behaves.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Maximum output line width in characters. Default: 80.
    pub max_line_width: usize,
    /// Insert a blank line after each clef token. Default: true.
    pub break_after_clef: bool,
    /// Insert a blank line after each bar token. Default: true.
    pub break_after_bar: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            max_line_width: 80,
            break_after_clef: true,
            break_after_bar: true,
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
/// Used by unit tests that only need to inspect token identity.
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

/// Pack a token stream into output lines respecting `max_line_width`.
///
/// Rules applied here:
/// - Tokens that were preceded by whitespace in the source are separated by a
///   single space on the same line (or wrapped to the next line when the width
///   limit would be exceeded).
/// - Tokens that were **not** preceded by whitespace (adjacent in the source)
///   are kept adjacent in the output — no space is inserted between them, and
///   the line-width limit is not enforced across the boundary.
/// - `break_after_clef` / `break_after_bar`: after the respective token, the
///   current line is emitted and the next token starts on a new line
///   (single line break, not a blank line).
/// - Comment tokens always start on their own line and force a new line
///   afterwards.
fn pack(entries: &[(Token, bool)], opts: &FormatOptions) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    let emit = |lines: &mut Vec<String>, current: &mut String| {
        let trimmed = current.trim_end().to_string();
        lines.push(trimmed);
        current.clear();
    };

    for (token, preceded_by_space) in entries {
        // Comments always go on their own line.
        if let Token::Comment { .. } = token {
            if !current.trim().is_empty() {
                emit(&mut lines, &mut current);
            }
            lines.push(token.display());
            continue;
        }

        let display = token.display();

        if current.trim().is_empty() {
            // First token on this line: just place it, no leading space.
            current.push_str(&display);
        } else if *preceded_by_space {
            // Token was space-separated in the source: add a space, or wrap
            // if the line would exceed the width limit.
            let candidate = format!("{current} {display}");
            if candidate.chars().count() > opts.max_line_width {
                emit(&mut lines, &mut current);
                current.push_str(&display);
            } else {
                current = candidate;
            }
        } else {
            // Token was adjacent (no whitespace) in the source: keep adjacent.
            // The width limit is not enforced here to preserve source intent.
            current.push_str(&display);
        }

        // Apply break-after rules *after* placing the token.
        // Inserts a blank line (empty line) between the clef/bar and the
        // following music, separating score sections visually.
        let needs_blank =
            (opts.break_after_clef && token.is_clef()) || (opts.break_after_bar && token.is_bar());
        if needs_blank {
            emit(&mut lines, &mut current);
            lines.push(String::new()); // blank line
        }
    }

    if !current.trim().is_empty() {
        lines.push(current.trim_end().to_string());
    }

    lines
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
    let packed = pack(&tokens, opts);

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
}
