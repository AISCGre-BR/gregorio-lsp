# Quickstart

## Prerequisites

- `rustc` and `cargo` (2021 edition, MSRV 1.70+).
- Optional: `tree-sitter-gregorio` at `../tree-sitter-gregorio` for the
  `tree-sitter` feature.

## Build

```bash
cargo build --release
```

Binaries are placed in `target/release/`:

- `gregorio-lsp` — LSP server over stdio.
- `grelint` — command-line linter.
- `grefmt` — command-line formatter.

To enable the tree-sitter integration:

```bash
cargo build --release --features tree-sitter
```

## CLI Lint

```bash
grelint examples/kyrie-xvi.gabc
grelint -s warning -i quilisma-missing-connector file.gabc
cat file.gabc | grelint -
```

Exit codes:

- `0` — no `error`-severity diagnostics.
- `1` — at least one error.

## CLI Format

```bash
# Format a file to stdout
grefmt file.gabc

# Format multiple files in-place
grefmt -i *.gabc

# Check whether files need reformatting (CI-friendly)
grefmt --check *.gabc          # exits 1 if any file would change

# Read from stdin, write to stdout
cat file.gabc | grefmt

# Customise line width and enable blank-line rules
grefmt -w 72 --break-after-clef --break-after-bar file.gabc
```

| Flag | Default | Description |
|---|---|---|
| `-w`, `--width <n>` | `80` | Maximum output line width |
| `--break-after-clef` | off | Blank line after each clef token (e.g. `(c4)`) |
| `--break-after-bar` | off | Blank line after each bar token (`,`, `;`, `:`, `::`, …) |
| `-c`, `--check` | off | Exits `1` if any file would change; writes nothing |
| `-i`, `--in-place` | off | Writes formatted output back to each file |

`--check` and `--in-place` are mutually exclusive. With no files, `grefmt` reads
stdin and writes to stdout.

Exit codes:

- `0` — all files already formatted.
- `1` — `--check` found at least one file that would change.
- `2` — bad argument, unreadable file, or write error.

## LSP in your editor

Configure the `gregorio-lsp` language server for the `gabc` language
(`.gabc` extension). Example (Helix):

```toml
[language-server.gregorio-lsp]
command = "gregorio-lsp"

[[language]]
name = "gabc"
file-types = ["gabc"]
language-servers = ["gregorio-lsp"]
```

Lint and formatting configuration is sent via `workspace/didChangeConfiguration`:

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
    "breakAfterBar": false
  }
}
```

The server advertises `textDocument/formatting`, so editors that support
format-on-save (Helix `:fmt`, Neovim `vim.lsp.buf.format()`, VS Code
**Format Document**) will call `grefmt`'s logic automatically.

## Tests

```bash
cargo test
```
