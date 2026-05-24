//! Integration tests for `format_gabc_text`.

use gregorio_lsp::format::{format_gabc_text, FormatOptions};

fn default_opts() -> FormatOptions {
    FormatOptions::default()
}

fn opts_width(w: usize) -> FormatOptions {
    FormatOptions {
        max_line_width: w,
        ..Default::default()
    }
}

// ── Header section ────────────────────────────────────────────────────────────

#[test]
fn header_section_preserved_verbatim() {
    let input = "name: Kyrie XVI;\nmode: 8;\nbook: GR 1961;\n%%\nA(g)\n";
    let result = format_gabc_text(input, &default_opts());
    assert!(
        result.starts_with("name: Kyrie XVI;\nmode: 8;\nbook: GR 1961;\n%%\n"),
        "header not preserved:\n{result}"
    );
}

#[test]
fn header_trailing_whitespace_stripped() {
    let input = "name: Test;   \nmode: 8;  \n%%\nA(g)\n";
    let result = format_gabc_text(input, &default_opts());
    assert!(result.starts_with("name: Test;\nmode: 8;\n%%\n"));
}

#[test]
fn no_header_section_formats_notation_only() {
    // Input with no %% — formatter should still produce output.
    let result = format_gabc_text("A(g) B(h)\n", &default_opts());
    assert_eq!(result.trim(), "A(g) B(h)");
}

// ── Line wrapping ─────────────────────────────────────────────────────────────

#[test]
fn tokens_fit_on_one_line_stay_on_one_line() {
    let input = "%%\nA(g) B(h) C(i)\n";
    let result = format_gabc_text(input, &opts_width(80));
    let notation: Vec<&str> = result.trim().split('\n').skip(1).collect();
    assert_eq!(notation, vec!["A(g) B(h) C(i)"]);
}

#[test]
fn long_notation_wraps_at_limit() {
    // 8 tokens, each "XX(gh)" = 6 chars; with spaces that's up to ~55 chars on one line.
    // Setting width = 20 forces wrapping.
    let tokens = (0..8u32)
        .map(|i| format!("S{i}(gh)"))
        .collect::<Vec<_>>()
        .join(" ");
    let input = format!("%%\n{tokens}\n");
    let result = format_gabc_text(&input, &opts_width(20));
    let notation_lines: Vec<&str> = result
        .trim()
        .split('\n')
        .skip(1)
        .filter(|l| !l.is_empty())
        .collect();
    assert!(notation_lines.len() > 1, "expected wrapping:\n{result}");
    for line in &notation_lines {
        assert!(line.chars().count() <= 20, "line exceeds limit: {line:?}");
    }
}

#[test]
fn reflows_fragmented_source() {
    // Source has one token per line; should be joined into a single line (fits within 80).
    let input = "%%\nA(g)\nB(h)\nC(i)\nD(j)\n";
    let result = format_gabc_text(input, &opts_width(80));
    let notation_lines: Vec<&str> = result
        .trim()
        .split('\n')
        .skip(1)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(notation_lines.len(), 1);
    assert_eq!(notation_lines[0], "A(g) B(h) C(i) D(j)");
}

#[test]
fn single_token_longer_than_limit_is_not_broken() {
    // A token wider than the limit must still appear on its own line untouched.
    let input = "%%\nAVeryLongSyllable(fghijklmnopqrst)\n";
    let result = format_gabc_text(input, &opts_width(10));
    assert!(result.contains("AVeryLongSyllable(fghijklmnopqrst)"));
}

// ── break_after_clef ──────────────────────────────────────────────────────────

#[test]
fn break_after_clef_inserts_blank_line() {
    let input = "%%\n(c4) KY(f)ri(gh)e(h)\n";
    let result = format_gabc_text(
        input,
        &FormatOptions {
            break_after_clef: true,
            ..Default::default()
        },
    );
    // Clef on its own line followed by a blank line before the music.
    assert!(
        result.contains("(c4)\n\nKY(f)"),
        "expected blank line after clef:\n{result}"
    );
}

#[test]
fn break_after_clef_all_clef_variants() {
    for clef in &["c1", "c2", "c3", "c4", "cb3", "cb4", "f1", "f3", "f4"] {
        let input = format!("%%\n({clef}) A(g)\n");
        let result = format_gabc_text(
            &input,
            &FormatOptions {
                break_after_clef: true,
                ..Default::default()
            },
        );
        assert!(
            result.contains(&format!("({clef})\n\nA(g)")),
            "no blank line after clef {clef}:\n{result}"
        );
    }
}

#[test]
fn break_after_clef_disabled_no_blank_line() {
    let input = "%%\n(c4) KY(f)ri(gh)e(h)\n";
    let result = format_gabc_text(
        input,
        &FormatOptions {
            break_after_clef: false,
            ..Default::default()
        },
    );
    assert!(
        !result.contains("(c4)\n\n"),
        "unexpected blank line after clef:\n{result}"
    );
}

// ── break_after_bar ───────────────────────────────────────────────────────────

#[test]
fn break_after_bar_inserts_blank_line_for_all_bar_types() {
    for bar_notes in &[",", ";", ":", "::"] {
        let input = format!("%%\nFoo(g) ({bar_notes}) Bar(h)\n");
        let result = format_gabc_text(
            &input,
            &FormatOptions {
                break_after_bar: true,
                ..Default::default()
            },
        );
        assert!(
            result.contains(&format!("({bar_notes})\n\nBar(h)")),
            "no blank line after bar `{bar_notes}`:\n{result}"
        );
    }
}

#[test]
fn break_after_bar_disabled_no_blank_line() {
    let input = "%%\nFoo(g) (,) Bar(h)\n";
    let result = format_gabc_text(
        input,
        &FormatOptions {
            break_after_bar: false,
            ..Default::default()
        },
    );
    assert!(!result.contains("(,)\n\n"));
}

#[test]
fn break_after_clef_and_bar_combined() {
    let input = "%%\n(c4) Foo(g) (,) Bar(h)\n";
    let result = format_gabc_text(
        input,
        &FormatOptions {
            break_after_clef: true,
            break_after_bar: true,
            ..Default::default()
        },
    );
    assert!(
        result.contains("(c4)\n\nFoo(g)"),
        "no blank after clef:\n{result}"
    );
    assert!(
        result.contains("(,)\n\nBar(h)"),
        "no blank after bar:\n{result}"
    );
}

// ── Styled text and special markers ──────────────────────────────────────────

#[test]
fn styled_text_preserved() {
    let input = "%%\nA(g) <i>bis</i>(::) B(h)\n";
    let result = format_gabc_text(input, &default_opts());
    assert!(
        result.contains("<i>bis</i>(::)"),
        "styled text lost:\n{result}"
    );
}

#[test]
fn special_markers_preserved() {
    let input = "%%\nA(g) * B(h) ** (,) + C(i)\n";
    let result = format_gabc_text(input, &default_opts());
    assert!(result.contains("*"), "star lost:\n{result}");
    assert!(result.contains("**"), "double-star lost:\n{result}");
    assert!(result.contains("+"), "plus lost:\n{result}");
}

// ── Real-world kyrie example ──────────────────────────────────────────────────

#[test]
fn kyrie_xvi_formats_without_error() {
    let kyrie = "\
name: Kyrie XVI;\n\
office-part: Kyriale;\n\
mode: 8;\n\
book: Graduale Romanum, 1961, p. 46*;\n\
%%\n\
(c4) KY(f)ri(gh)e(h.) *() e(ixh_i_H'GhvF'E)lé(fgf')i(f)son.(f.) <i>bis</i>(::)\n\
Chri(ixf!gwh_GF'g)ste(fgf.) e(f)lé(gh)i(h)son.(h.) <i>bis</i>(::)\n\
Ký(f)ri(gh)e(ixhih.) <sp>V/</sp>.(::)\n\
e(ixh_i_H'Ghih)lé(gf~)i(gh/ih)son.(h.) (::)\n\
Ký(f)ri(h)e(ixhhi) **(,) (hg/hih) (,) e(f)lé(gh)i(h)son.(h.) (::)\n";

    let result = format_gabc_text(kyrie, &default_opts());
    // Result must contain the header verbatim.
    assert!(result.starts_with("name: Kyrie XVI;\n"));
    // Result must end with exactly one newline.
    assert!(result.ends_with('\n'));
    assert!(!result.ends_with("\n\n"));
    // All note-group content must be preserved.
    assert!(result.contains("ixh_i_H'GhvF'E"));
}

// ── Output invariants ─────────────────────────────────────────────────────────

#[test]
fn output_ends_with_single_newline() {
    for input in &["%%\nA(g)\n", "%%\nA(g)", "name: T;\n%%\nA(g)\n"] {
        let result = format_gabc_text(input, &default_opts());
        assert!(
            result.ends_with('\n'),
            "missing trailing newline for: {input:?}"
        );
        assert!(
            !result.ends_with("\n\n"),
            "double trailing newline for: {input:?}"
        );
    }
}

#[test]
fn idempotent_default_options() {
    let input = "name: Test;\n%%\nA(g) B(h) C(i)\n";
    let once = format_gabc_text(input, &default_opts());
    let twice = format_gabc_text(&once, &default_opts());
    assert_eq!(once, twice, "formatter is not idempotent");
}
