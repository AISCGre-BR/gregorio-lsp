//! GABC pitch transposition.
//!
//! Shifts pitch letters (`a`–`n`, `p` and their uppercase equivalents) inside
//! `(…)` note groups in the music section of a GABC document.
//!
//! ## Pitch cycle (ascending)
//!
//! ```text
//! a  b  c  d  e  f  g  h  i  j  k  l  m  n  p  →  a  (wraps)
//! ```
//!
//! Descending is the exact reverse.
//!
//! ## Multi-NABC support
//!
//! When the document declares `nabc-lines: N;` in its headers, each `(…)` group
//! may contain multiple `|`-separated segments that cycle between GABC and NABC
//! with period `N+1`.  Segment index `k` is GABC when `k mod (N+1) == 0`; all
//! others are NABC and are left unchanged.
//!
//! Example with `nabc-lines: 2`:
//! ```text
//! (fgh | pu | ta | ij | vi | pe)
//!  ^^^   NABC  NABC  ^^   NABC NABC
//!  GABC               GABC
//! ```

/// Direction of pitch shift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftDirection {
    /// Shift one step upward  (… n → p → a …).
    Up,
    /// Shift one step downward (… a → p → n …).
    Down,
}

// Pitch cycle ascending (lowercase).  'o' is intentionally absent.
const CYCLE_LOWER: [char; 15] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'p',
];

// Pitch cycle ascending (uppercase).  'O' is intentionally absent.
const CYCLE_UPPER: [char; 15] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'P',
];

/// Returns `true` if `c` is a valid GABC pitch letter.
pub fn is_gabc_pitch(c: char) -> bool {
    matches!(c, 'a'..='n' | 'A'..='N' | 'p' | 'P')
}

/// Shifts a single GABC pitch letter one step in `dir`.
///
/// Non-pitch characters are returned unchanged.
pub fn shift_pitch(c: char, dir: ShiftDirection) -> char {
    let cycle: &[char] = if c.is_ascii_uppercase() {
        &CYCLE_UPPER
    } else {
        &CYCLE_LOWER
    };
    match cycle.iter().position(|&x| x == c) {
        Some(pos) => match dir {
            ShiftDirection::Up => cycle[(pos + 1) % cycle.len()],
            ShiftDirection::Down => cycle[(pos + cycle.len() - 1) % cycle.len()],
        },
        None => c,
    }
}

/// Applies a pitch shift to the music section of a GABC document.
///
/// Only pitch letters inside `(…)` groups after the `%%` separator are
/// modified.  Clef patterns at the start of a group, NABC segments (as
/// determined by the `nabc-lines` header), lyric text, and headers are left
/// unchanged.
///
/// If `byte_range` is `Some(start..end)`, only pitch letters whose byte
/// offset in `text` falls within that range are shifted (selection mode).
/// If `None`, all pitch letters in the music section are shifted.
pub fn shift_notes(
    text: &str,
    dir: ShiftDirection,
    byte_range: Option<std::ops::Range<usize>>,
) -> String {
    let nabc_lines = parse_nabc_lines(text);
    let mut result = String::with_capacity(text.len());
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_music = false;

    while i < n {
        let (_, ch) = chars[i];

        // Detect the %% section separator.
        if !in_music && ch == '%' && i + 1 < n && chars[i + 1].1 == '%' {
            result.push_str("%%");
            i += 2;
            in_music = true;
            continue;
        }

        // Headers (before %%) pass through unchanged.
        if !in_music {
            result.push(ch);
            i += 1;
            continue;
        }

        // Comments in the music section (% … newline) pass through unchanged.
        if ch == '%' {
            while i < n && chars[i].1 != '\n' {
                result.push(chars[i].1);
                i += 1;
            }
            continue;
        }

        // Lyric text (outside parentheses) passes through unchanged.
        if ch != '(' {
            result.push(ch);
            i += 1;
            continue;
        }

        // Opening parenthesis: process the group.
        result.push('(');
        i += 1;

        shift_group(&chars, &mut i, &mut result, dir, byte_range.as_ref(), nabc_lines);

        // Closing parenthesis.
        if i < n && chars[i].1 == ')' {
            result.push(')');
            i += 1;
        }
    }

    result
}

/// Processes the content of one `(…)` group (starting at the character
/// immediately after `(`).
///
/// Clef groups are copied verbatim.  In note groups, each `|`-separated
/// segment is classified as GABC or NABC according to `nabc_lines`, and only
/// GABC pitch letters are shifted.
fn shift_group(
    chars: &[(usize, char)],
    i: &mut usize,
    result: &mut String,
    dir: ShiftDirection,
    byte_range: Option<&std::ops::Range<usize>>,
    nabc_lines: usize,
) {
    // Clef groups are passed through unchanged.
    if is_clef_group(chars, *i) {
        while *i < chars.len() && chars[*i].1 != ')' {
            result.push(chars[*i].1);
            *i += 1;
        }
        return;
    }

    // Segment index `seg` increments at every `|`.
    //
    // When nabc_lines == 0 (no NABC declared), the first `|` permanently
    // switches to NABC (seg > 0 → NABC).
    //
    // When nabc_lines > 0, segments cycle with period (nabc_lines + 1):
    //   seg % (nabc_lines + 1) == 0  →  GABC
    //   otherwise                    →  NABC
    let period = nabc_lines + 1; // always >= 1; for nabc_lines==0, period==1 but guarded below
    let mut seg: usize = 0;

    while *i < chars.len() && chars[*i].1 != ')' {
        let (byte_pos, c) = chars[*i];

        if c == '|' {
            seg += 1;
            result.push(c);
            *i += 1;
            continue;
        }

        // With nabc_lines == 0: only seg 0 is GABC.
        // With nabc_lines >  0: seg is GABC iff seg % period == 0.
        let is_gabc = if nabc_lines == 0 {
            seg == 0
        } else {
            seg % period == 0
        };

        if is_gabc && is_gabc_pitch(c) {
            let in_range = byte_range.map_or(true, |r| r.contains(&byte_pos));
            result.push(if in_range { shift_pitch(c, dir) } else { c });
        } else {
            result.push(c);
        }

        *i += 1;
    }
}

/// Parses the `nabc-lines` value from the document headers (before `%%`).
///
/// Returns `0` if the header is absent or unparseable.
pub fn parse_nabc_lines(text: &str) -> usize {
    let sep = text.find("%%").unwrap_or(text.len());
    for line in text[..sep].lines() {
        let lower = line.trim().to_lowercase();
        if let Some(rest) = lower.strip_prefix("nabc-lines:") {
            if let Ok(n) = rest.trim().trim_end_matches(';').trim().parse::<usize>() {
                return n;
            }
        }
    }
    0
}

/// Returns `true` if the group content starting at `chars[start]` (the
/// character immediately after `(`) begins with a clef pattern:
/// `(c|f) b? [1-4]`.
fn is_clef_group(chars: &[(usize, char)], start: usize) -> bool {
    let mut j = start;
    // Skip leading whitespace inside the group.
    while j < chars.len() && chars[j].1 != ')' && chars[j].1.is_whitespace() {
        j += 1;
    }
    if j >= chars.len() || !matches!(chars[j].1, 'c' | 'f') {
        return false;
    }
    let mut k = j + 1;
    if k < chars.len() && chars[k].1 == 'b' {
        k += 1;
    }
    k < chars.len() && matches!(chars[k].1, '1'..='4')
}
