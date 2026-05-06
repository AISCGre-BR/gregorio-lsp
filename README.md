# gregorio-lsp

Language Server and linter for GABC (Gregorio) files and NABC notation, written in Rust.

This is the Rust rewrite of the original TypeScript project, providing the same
feature set with native binaries:

- `gregorio-lsp` — LSP server over stdio (based on [tower-lsp](https://github.com/ebkalderon/tower-lsp));
- `gregolint` — CLI linter for GABC files and stdin input.

## Features

- Full GABC parser (headers, syllables, clefs, bars, custos, attributes, modifiers, and line breaks);
- NABC parser (St. Gall, Laon, modifiers, subpunctis/prepunctis, significant letters, and `!` fusions);
- 10 structural validation rules;
- Semantic analyzer with musical construction checks (quilisma, oriscus scapus, pes quadratum, virga strata, etc.);
- Optional [tree-sitter-gregorio](../tree-sitter-gregorio) integration via the `tree-sitter` feature flag;
- Integration test suite with 29 cases covering parser, NABC, and validation.

## Building

Requires `cargo` and `rustc` (2021 edition).

```bash
cargo build --release
cargo test
cargo build --release --features tree-sitter   # enables tree-sitter integration
```

Binaries are placed in `target/release/gregorio-lsp` and `target/release/gregolint`.

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
  }
}
```

## `gregolint` CLI Usage

```bash
gregolint examples/kyrie-xvi.gabc
gregolint -s warning -i quilisma-missing-connector file.gabc
cat file.gabc | gregolint -
```

Output format: `file:line:column: severity [code] message`. The process exits with
`1` if at least one error-severity diagnostic is found.

## Structure

```
src/
  lib.rs                  # public API
  lint.rs                 # reusable lint pipeline
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
    lint.rs               # gregolint binary
tests/
  gabc_parser.rs
  nabc_parser.rs
  validation.rs
```

## License

MIT — Copyright (c) 2026 AISCGre Brasil. See [LICENSE](LICENSE).
