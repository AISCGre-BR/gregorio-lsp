//! `gregolint` CLI: lint one or more GABC files (or stdin) and print diagnostics.

use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use gregorio_lsp::lint::{lint_gabc_text, LintOptions, LintSeverity};
use gregorio_lsp::parser::types::Severity;

fn print_help() {
    eprintln!(
        "gregolint {} — Gregorio GABC/NABC linter\n\
\n\
USAGE:\n    gregolint [OPTIONS] [FILES...]\n\
\n\
OPTIONS:\n    -s, --severity <error|warning|info>  Minimum severity to report (default: info)\n    -i, --ignore <code>                   Ignore a diagnostic code (repeatable)\n    -h, --help                            Print help\n    -V, --version                         Print version\n\
\n\
If no FILES are given, reads GABC from stdin.",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut files: Vec<PathBuf> = Vec::new();
    let mut min_severity: Option<LintSeverity> = None;
    let mut ignore_codes: Vec<String> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("gregolint {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
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
            other if other.starts_with('-') => {
                eprintln!("error: unknown option '{other}'");
                return ExitCode::from(2);
            }
            other => files.push(PathBuf::from(other)),
        }
    }

    let opts = LintOptions {
        min_severity,
        ignore_codes,
    };

    let mut had_error = false;
    if files.is_empty() {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            eprintln!("error reading stdin: {e}");
            return ExitCode::from(2);
        }
        if report("<stdin>", &buf, &opts) {
            had_error = true;
        }
    } else {
        for file in &files {
            match std::fs::read_to_string(file) {
                Ok(text) => {
                    if report(&file.display().to_string(), &text, &opts) {
                        had_error = true;
                    }
                }
                Err(e) => {
                    eprintln!("error reading {}: {e}", file.display());
                    had_error = true;
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

fn report(label: &str, text: &str, opts: &LintOptions) -> bool {
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
