# gregorio-lsp

> [!NOTE]
> **AI-Assisted Development**
>
> This project is developed, fully or in part, with LLM/generative AI assistance.
> All LLM-proposed changes go through human review and local testing before being committed.
>
> If you object to software built with generative AI assistance, we respect your opinion
> and regret we cannot meet your expectations. In that case, we kindly recommend exploring
> alternative solutions. If you know of any, feel free to open an issue — we'll be happy
> to include them in our documentation.

Language Server, linter, and formatter for GABC (Gregorio) files and NABC notation,
written in Rust.

This is the Rust rewrite of the original TypeScript project, providing the same
feature set with native binaries:

- `gregorio-lsp` — LSP server over stdio (based on [tower-lsp](https://github.com/ebkalderon/tower-lsp));
- `grelint` — CLI linter for GABC files and stdin input;
- `grefmt` — CLI formatter for GABC files and stdin input.

## Features

- Full GABC parser (headers, syllables, clefs, bars, custos, attributes, modifiers, and line breaks);
- NABC parser (St. Gall, Laon, modifiers, subpunctis/prepunctis, significant letters, and `!` fusions);
- 10 structural validation rules;
- Semantic analyzer with musical construction checks (quilisma, oriscus scapus, pes quadratum, virga strata, etc.);
- Source formatter with configurable line-width wrapping and optional blank lines after clef/bar tokens;
- Optional [tree-sitter-gregorio](../tree-sitter-gregorio) integration via the `tree-sitter` feature flag;
- Integration test suite with 29 cases covering parser, NABC, and validation.

## Installation

### Prebuilt binaries

Every release from v0.13.0 on ships prebuilt `gregorio-lsp`, `grelint` and
`grefmt` executables on the
[Releases page](https://github.com/AISCGre-BR/gregorio-lsp/releases):

| Platform | Asset |
|---|---|
| Linux x86-64 (static, musl) | `gregorio-lsp-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 (static, musl) | `gregorio-lsp-aarch64-unknown-linux-musl.tar.gz` |
| macOS Intel | `gregorio-lsp-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `gregorio-lsp-aarch64-apple-darwin.tar.gz` |
| Windows x86-64 | `gregorio-lsp-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `gregorio-lsp-aarch64-pc-windows-msvc.zip` |

Extract the archive and put the executables on your `PATH`. `SHA256SUMS`
lists the checksums of all assets. The
[Zed extension](https://github.com/lbssousa/zed-gregorio) downloads these
binaries automatically.

### From source

```bash
cargo install --git https://github.com/AISCGre-BR/gregorio-lsp \
  --tag v0.13.0 gregorio-server --bin gregorio-lsp
# CLI tools (optional)
cargo install --git https://github.com/AISCGre-BR/gregorio-lsp \
  --tag v0.13.0 gregorio-cli
```

## Building

Requires `cargo` and `rustc` (2021 edition).

```bash
cargo build --release
cargo test
cargo build --release --features tree-sitter   # enables tree-sitter integration
```

Binaries are placed in `target/release/`:

- `gregorio-lsp` — LSP server.
- `grelint` — CLI linter.
- `grefmt` — CLI formatter.

## LSP Server Usage

The `gregorio-lsp` binary reads and writes LSP messages over `stdio`. Configure your
editor to launch it as the language server for the `gabc` language.

Example JSON configuration sent via `workspace/didChangeConfiguration`:

```json
{
  "linting": {
    "enabled": true,
    "severity": "warning",
    "onSave": false,
    "ignoreRules": []
  },
  "formatting": {
    "maxLineWidth": 80,
    "breakAfterClef": false,
    "breakAfterBar": false,
    "wordPerLine": "auto"
  }
}
```

## `grelint` CLI Usage

```bash
grelint examples/kyrie-xvi.gabc
grelint -s warning -i quilisma-missing-connector file.gabc
cat file.gabc | grelint -
```

Output format: `file:line:column: severity [code] message`. The process exits with
`1` if at least one error-severity diagnostic is found.

## `grefmt` CLI Usage

```bash
# Format a file to stdout
grefmt file.gabc

# Format multiple files in-place
grefmt -i *.gabc

# Check whether files are already formatted (useful in CI)
grefmt --check *.gabc   # exits 1 if any file would change

# Pipe through stdin / stdout
cat file.gabc | grefmt

# Narrow line width and enable blank-line rules
grefmt -w 72 --break-after-clef --break-after-bar file.gabc
```

### Options

| Flag | Default | Description |
|---|---|---|
| `-w`, `--width <n>` | `80` | Maximum output line width in characters |
| `--break-after-clef` | off | Insert a blank line after each clef token, e.g. `(c4)` |
| `--break-after-bar` | off | Insert a blank line after each bar token (`,`, `;`, `:`, `::`, …) |
| `--word-per-line <mode>` | `auto` | One word per line: `auto`, `always` or `never` (see below) |
| `-c`, `--check` | off | Verify formatting without writing; exits `1` if any file would change |
| `-i`, `--in-place` | off | Write formatted output back to each input file |
| `-h`, `--help` | — | Print help |
| `-V`, `--version` | — | Print version |

`--check` and `--in-place` are mutually exclusive. When no files are given, `grefmt`
reads from stdin and writes to stdout.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | All files already formatted (or stdin formatted successfully) |
| `1` | `--check` found at least one file that would change |
| `2` | Argument parsing error or unreadable/unwritable file |

### What `grefmt` changes

The formatter operates on the **notation body** (everything after `%%`). It:

1. Collapses all inter-syllable whitespace and re-flows the notation onto lines no
   wider than `--width`.
2. Optionally inserts a blank line after each clef token (`--break-after-clef`):
   ```
   (c4)

   KY(f)ri(gh)e(h)
   ```
3. Optionally inserts a blank line after each bar token (`--break-after-bar`):
   ```
   KY(f)ri(gh)e(h) (,)

   e(f)lé(gh)i(h)son(h.)
   ```
4. Places each word on its own line (`--word-per-line`). In `auto` mode (the
   default) this happens when the `nabc-lines` header is 2 or greater, or when at
   least 25% of the words are wider than half of `--width` — typical of long
   GABC+NABC code such as the Graduale Triplex. Bars and `*` / `**` stay at the end
   of the preceding word's line:
   ```
   DI(g|un|ta)le(g|un|ta)xí(hig|to|to)sti(gf|clG|cl-) *()
   ju(g|un|vi)stí(gh|pe|pe)ti(j|un|ta)am,(j|un|ta) (;)
   ```

The **header section** (before `%%`) is preserved verbatim except that trailing
whitespace on each header line is stripped. Content inside parentheses is never
modified.

## Structure

```
src/
  lib.rs                  # public API
  lint.rs                 # reusable lint pipeline
  format.rs               # reusable format pipeline
  parser/
    types.rs              # AST types
    gabc.rs               # GABC parser
    nabc.rs               # NABC parser
  validation/
    rules.rs              # structural rules
    semantic.rs           # semantic analyzer
    validator.rs          # orchestrator
  tree_sitter_integration.rs   # optional, `tree-sitter` feature
  bin/
    server.rs             # gregorio-lsp binary
    lint.rs               # grelint binary
    grefmt.rs             # grefmt binary
tests/
  gabc_parser.rs
  nabc_parser.rs
  validation.rs
  format.rs
```

## License

MIT — Copyright (c) 2026 AISCGre Brasil. See [LICENSE](LICENSE).
