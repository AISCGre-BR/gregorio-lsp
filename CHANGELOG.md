# Changelog

## 1.0.0-alpha.1 — 2026-05-02

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
  `v1.0.0-alpha.1` (`https://github.com/aiscgre-br/tree-sitter-gregorio`).
- Copyright and authorship updated to **AISCGre Brasil** across `LICENSE`,
  `Cargo.toml`, and `README.md`.
- CLI `gregolint`: unreadable file now exits with code `2` (CLI execution failure)
  instead of `1` (lint errors found), matching documented exit code semantics.

## 0.2.0 — Reescrita em Rust

- Migração completa do código TypeScript para Rust (edição 2021).
- Servidor LSP reimplementado sobre [tower-lsp](https://github.com/ebkalderon/tower-lsp).
- CLI `gregolint` portado para Rust com mesma interface (`-s`, `-i`, `-h`, `-V`).
- Parsers GABC e NABC reescritos preservando comportamento e mensagens.
- Analisador semântico e dez regras de validação portados.
- Integração com [tree-sitter-gregorio](../tree-sitter-gregorio) tornada opcional via feature `tree-sitter`.
- Suíte de testes nova com 58 casos de integração cobrindo parser GABC,
  parser NABC, validação, indicadores leaning, oriscus e corpus de exemplos.
- Removidos artefatos do toolchain Node (`package.json`, `tsconfig.json`,
  `jest.config.js`, `.eslintrc.json`, `scripts/`, `node_modules/`).

## 0.1.x

Histórico anterior (TypeScript) preservado nos commits de Git.
