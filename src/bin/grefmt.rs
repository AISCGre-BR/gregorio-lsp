//! `grefmt` CLI: format one or more GABC files (or stdin).

use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use gregorio_lsp::format::{format_gabc_text, FormatOptions};

fn print_help() {
    eprintln!(
        "grefmt {} — Gregorio GABC/NABC formatter\n\
\n\
USAGE:\n    grefmt [OPTIONS] [FILES...]\n\
\n\
OPTIONS:\n    -w, --width <n>       Maximum line width (default: 80)\n        --break-after-clef  Insert a blank line after each clef token\n        --break-after-bar   Insert a blank line after each bar token\n    -c, --check           Exit 1 if any file would change; do not write output\n    -i, --in-place        Write formatted output back to each file\n    -h, --help            Print help\n    -V, --version         Print version\n\
\n\
If no FILES are given, reads from stdin and writes to stdout.\n\
Exit codes: 0 = all files already formatted | 1 = --check found a diff | 2 = argument or I/O error",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut files: Vec<PathBuf> = Vec::new();
    let mut max_line_width: usize = 80;
    let mut break_after_clef = false;
    let mut break_after_bar = false;
    let mut check_mode = false;
    let mut in_place = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("grefmt {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "-c" | "--check" => check_mode = true,
            "-i" | "--in-place" => in_place = true,
            "--break-after-clef" => break_after_clef = true,
            "--break-after-bar" => break_after_bar = true,
            "-w" | "--width" => match args.next() {
                Some(s) => match s.parse::<usize>() {
                    Ok(n) if n > 0 => max_line_width = n,
                    _ => {
                        eprintln!("error: --width requires a positive integer, got '{s}'");
                        return ExitCode::from(2);
                    }
                },
                None => {
                    eprintln!("error: --width requires a value");
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

    if check_mode && in_place {
        eprintln!("error: --check and --in-place are mutually exclusive");
        return ExitCode::from(2);
    }

    let opts = FormatOptions {
        max_line_width,
        break_after_clef,
        break_after_bar,
    };

    if files.is_empty() {
        return run_stdin(&opts, check_mode);
    }

    run_files(&files, &opts, check_mode, in_place)
}

// ── stdin ─────────────────────────────────────────────────────────────────────

fn run_stdin(opts: &FormatOptions, check_mode: bool) -> ExitCode {
    let mut buf = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut buf) {
        eprintln!("error reading stdin: {e}");
        return ExitCode::from(2);
    }
    let formatted = format_gabc_text(&buf, opts);
    if check_mode {
        if formatted != buf {
            return ExitCode::from(1);
        }
        return ExitCode::SUCCESS;
    }
    print!("{formatted}");
    ExitCode::SUCCESS
}

// ── file mode ─────────────────────────────────────────────────────────────────

fn run_files(
    files: &[PathBuf],
    opts: &FormatOptions,
    check_mode: bool,
    in_place: bool,
) -> ExitCode {
    let mut any_diff = false;

    for file in files {
        let text = match std::fs::read_to_string(file) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error reading {}: {e}", file.display());
                return ExitCode::from(2);
            }
        };

        let formatted = format_gabc_text(&text, opts);

        if check_mode {
            if formatted != text {
                eprintln!("would reformat: {}", file.display());
                any_diff = true;
            }
            continue;
        }

        if in_place {
            if formatted != text {
                if let Err(e) = std::fs::write(file, &formatted) {
                    eprintln!("error writing {}: {e}", file.display());
                    return ExitCode::from(2);
                }
                eprintln!("reformatted: {}", file.display());
            }
        } else {
            print!("{formatted}");
        }
    }

    if check_mode && any_diff {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
