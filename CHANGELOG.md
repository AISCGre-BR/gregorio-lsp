# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-05-04

### Added
- `AGENTS.md`: comprehensive AI code generation guide covering architecture, parsers,
  validation rules, LSP server, CLI linter, tree-sitter integration, testing, and
  commit style.
- CLI `gregolint`: `--format json` (`-f json`) output mode — structured JSON with
  `tool`, `diagnostics[]`, `skipped[]`, and `summary` fields; ranges are 0-based
  (LSP convention), suitable for CI pipelines and editor integrations.
- Fallback GABC parser now correctly handles Gregorio 6.2.0 changes:
  - Empty GABC snippets before NABC content (`(|vi)`, `(||ta)`, `(g||ta)`) — pipe
    always starts the NABC side of the snippet list.
  - Lyric tie `~` in syllable text passes through as literal text (not a parse error).
- Four new regression tests covering the above parser changes.

### Changed
- `tree-sitter-gregorio` dependency changed from local path to published git tag
  `v0.5.2` (`https://github.com/aiscgre-br/tree-sitter-gregorio`).
- Copyright and authorship updated to **AISCGre Brasil** across `LICENSE`,
  `Cargo.toml`, and `README.md`.
- CLI `gregolint`: unreadable file now exits with code `2` (CLI execution failure)
  instead of `1` (lint errors found), matching documented exit code semantics.

## [0.2.0]

### Changed
- Full rewrite from TypeScript to Rust (edition 2021).
- LSP server reimplemented on [tower-lsp](https://github.com/ebkalderon/tower-lsp).
- CLI `gregolint` ported to Rust with the same interface (`-s`, `-i`, `-h`, `-V`).
- GABC and NABC parsers rewritten preserving behaviour and messages.
- Semantic analyser and ten validation rules ported.
- Integration with [tree-sitter-gregorio](https://github.com/aiscgre-br/tree-sitter-gregorio)
  made optional via `tree-sitter` feature flag.
- Test suite with 58 integration cases covering the GABC parser, NABC parser,
  validation, leaning indicators, oriscus, and example corpus.
- Removed Node toolchain artefacts (`package.json`, `tsconfig.json`,
  `jest.config.js`, `.eslintrc.json`, `scripts/`, `node_modules/`).

## [0.1.x]

- Initial TypeScript implementation. History preserved in Git commits.

---

[0.3.0]: https://github.com/aiscgre-br/gregorio-lsp/releases/tag/v0.3.0
