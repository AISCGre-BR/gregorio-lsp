//! `grelint` CLI: lint one or more GABC files (or stdin) and print diagnostics.

use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use gregorio_core::lint::{lint_gabc_text, LintOptions, LintSeverity};
use gregorio_core::parser::types::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

fn print_help() {
    eprintln!(
        "grelint {} — Gregorio GABC/NABC linter\n\
\n\
USAGE:\n    grelint [OPTIONS] [FILES...]\n\
\n\
OPTIONS:\n    -s, --severity <error|warning|info>  Minimum severity to report (default: info)\n    -i, --ignore <code>                   Ignore a diagnostic code (repeatable)\n    -f, --format <text|json>              Output format (default: text)\n        --fix                             Apply auto-fixable diagnostics in-place\n    -h, --help                            Print help\n    -V, --version                         Print version\n\
\n\
If no FILES are given, reads GABC from stdin.\n\
When --fix is used with stdin, the fixed output is written to stdout.",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut files: Vec<PathBuf> = Vec::new();
    let mut min_severity: Option<LintSeverity> = None;
    let mut ignore_codes: Vec<String> = Vec::new();
    let mut format = OutputFormat::Text;
    let mut fix_mode = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("grelint {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--fix" => fix_mode = true,
            "-s" | "--severity" => match args.next() {
                Some(s) => match LintSeverity::parse(&s) {
                    Some(sev) => min_severity = Some(sev),
                    None => {
                        eprintln!("error: invalid severity '{s}'");
                        return ExitCode::from(2);
                    }
                },
                None => {
                    eprintln!("error: --severity requires a value");
                    return ExitCode::from(2);
                }
            },
            "-i" | "--ignore" => match args.next() {
                Some(c) => ignore_codes.push(c),
                None => {
                    eprintln!("error: --ignore requires a value");
                    return ExitCode::from(2);
                }
            },
            "-f" | "--format" => match args.next() {
                Some(f) => match f.as_str() {
                    "text" => format = OutputFormat::Text,
                    "json" => format = OutputFormat::Json,
                    other => {
                        eprintln!("error: invalid format '{other}' (expected 'text' or 'json')");
                        return ExitCode::from(2);
                    }
                },
                None => {
                    eprintln!("error: --format requires a value");
                    return ExitCode::from(2);
                }
            },
            other if other.starts_with('-') => {
                eprintln!("error: unknown option '{other}'");
                return ExitCode::from(2);
            }
            other => files.push(PathBuf::from(other)),
        }
    }

    if fix_mode {
        return run_fix(&files);
    }

    let opts = LintOptions {
        min_severity,
        ignore_codes,
    };

    match format {
        OutputFormat::Text => run_text(&files, &opts),
        OutputFormat::Json => run_json(&files, &opts),
    }
}

// ── Text output ───────────────────────────────────────────────────────────────

fn run_text(files: &[PathBuf], opts: &LintOptions) -> ExitCode {
    let mut had_error = false;
    if files.is_empty() {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            eprintln!("error reading stdin: {e}");
            return ExitCode::from(2);
        }
        if report_text("<stdin>", &buf, opts) {
            had_error = true;
        }
    } else {
        for file in files {
            match std::fs::read_to_string(file) {
                Ok(text) => {
                    if report_text(&file.display().to_string(), &text, opts) {
                        had_error = true;
                    }
                }
                Err(e) => {
                    eprintln!("error reading {}: {e}", file.display());
                    return ExitCode::from(2);
                }
            }
        }
    }
    if had_error {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn report_text(label: &str, text: &str, opts: &LintOptions) -> bool {
    let diags = lint_gabc_text(text, opts);
    let mut had_error = false;
    for d in diags {
        if d.severity == Severity::Error {
            had_error = true;
        }
        let code = d.code.as_deref().unwrap_or("-");
        println!(
            "{}:{}:{}: {} [{}] {}",
            label,
            d.range.start.line + 1,
            d.range.start.character + 1,
            d.severity.as_str(),
            code,
            d.message
        );
    }
    had_error
}

// ── JSON output ───────────────────────────────────────────────────────────────

fn run_json(files: &[PathBuf], opts: &LintOptions) -> ExitCode {
    use serde_json::{json, Value};

    let mut all_diags: Vec<Value> = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();
    let mut had_error = false;
    let mut error_count: u32 = 0;
    let mut warning_count: u32 = 0;
    let mut info_count: u32 = 0;
    let mut file_count: u32 = 0;

    let inputs: Vec<(String, Result<String, String>)> = if files.is_empty() {
        let mut buf = String::new();
        let result = io::stdin()
            .read_to_string(&mut buf)
            .map(|_| buf)
            .map_err(|e| e.to_string());
        vec![("<stdin>".to_string(), result)]
    } else {
        files
            .iter()
            .map(|p| {
                (
                    p.display().to_string(),
                    std::fs::read_to_string(p).map_err(|e| e.to_string()),
                )
            })
            .collect()
    };

    for (label, result) in inputs {
        file_count += 1;
        match result {
            Err(reason) => {
                skipped.push(json!({ "file": label, "reason": reason }));
            }
            Ok(text) => {
                let diags = lint_gabc_text(&text, opts);
                for d in diags {
                    match d.severity {
                        Severity::Error => {
                            had_error = true;
                            error_count += 1;
                        }
                        Severity::Warning => warning_count += 1,
                        Severity::Info => info_count += 1,
                    }
                    all_diags.push(json!({
                        "file": label,
                        "severity": d.severity.as_str(),
                        "code": d.code,
                        "message": d.message,
                        "range": {
                            "start": {
                                "line": d.range.start.line,
                                "character": d.range.start.character
                            },
                            "end": {
                                "line": d.range.end.line,
                                "character": d.range.end.character
                            }
                        },
                        "source": "grelint"
                    }));
                }
            }
        }
    }

    let output = json!({
        "tool": "grelint",
        "diagnostics": all_diags,
        "skipped": skipped,
        "summary": {
            "files": file_count,
            "diagnostics": all_diags.len(),
            "errors": error_count,
            "warnings": warning_count,
            "info": info_count
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).unwrap_or_default()
    );

    if had_error {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

// ── Fix mode ──────────────────────────────────────────────────────────────────

fn run_fix(files: &[PathBuf]) -> ExitCode {
    let fix_opts = LintOptions {
        min_severity: Some(LintSeverity::Info),
        ignore_codes: Vec::new(),
    };

    if files.is_empty() {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            eprintln!("error reading stdin: {e}");
            return ExitCode::from(2);
        }
        let fixed = apply_fixes(&buf, &lint_gabc_text(&buf, &fix_opts));
        print!("{fixed}");
        return ExitCode::SUCCESS;
    }

    for file in files {
        let text = match std::fs::read_to_string(file) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error reading {}: {e}", file.display());
                return ExitCode::from(2);
            }
        };
        let diags = lint_gabc_text(&text, &fix_opts);
        let fixed = apply_fixes(&text, &diags);
        if fixed != text {
            if let Err(e) = std::fs::write(file, &fixed) {
                eprintln!("error writing {}: {e}", file.display());
                return ExitCode::from(2);
            }
            eprintln!("fixed: {}", file.display());
        }
    }
    ExitCode::SUCCESS
}

/// Convert a (line, character) position to a byte offset in `text`.
/// Both line and character are 0-based; character is counted in Unicode code points.
fn byte_offset(text: &str, line: usize, character: usize) -> Option<usize> {
    let mut cur_line = 0usize;
    let mut cur_char = 0usize;
    for (idx, ch) in text.char_indices() {
        if cur_line == line && cur_char == character {
            return Some(idx);
        }
        if ch == '\n' {
            cur_line += 1;
            cur_char = 0;
        } else if cur_line == line {
            cur_char += 1;
        }
    }
    if cur_line == line && cur_char == character {
        return Some(text.len());
    }
    None
}

/// Apply all auto-fixes from `diags` to `text`, returning the modified string.
/// Fixes are applied from end to start so that earlier positions remain valid.
fn apply_fixes(text: &str, diags: &[gregorio_core::parser::types::ParseError]) -> String {
    use gregorio_core::parser::types::Range;

    let mut fixes: Vec<(Range, &str)> = diags
        .iter()
        .filter_map(|d| d.fix.as_ref().map(|f| (f.range, f.new_text.as_str())))
        .collect();

    // Sort in reverse order (end-of-file first) so each replacement doesn't shift
    // the byte offsets of subsequent (earlier) replacements.
    fixes.sort_by(|(r1, _), (r2, _)| {
        r2.start
            .line
            .cmp(&r1.start.line)
            .then(r2.start.character.cmp(&r1.start.character))
    });

    let mut result = text.to_string();
    for (range, new_text) in fixes {
        let Some(start) = byte_offset(&result, range.start.line, range.start.character) else {
            continue;
        };
        let Some(end) = byte_offset(&result, range.end.line, range.end.character) else {
            continue;
        };
        result = format!("{}{}{}", &result[..start], new_text, &result[end..]);
    }
    result
}
