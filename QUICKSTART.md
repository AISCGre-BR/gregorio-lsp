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

Lint configuration is sent via `workspace/didChangeConfiguration`:

```json
{
  "linting": {
    "enabled": true,
    "severity": "warning",
    "onSave": false,
    "ignoreRules": []
  }
}
```

## Tests

```bash
cargo test
```
