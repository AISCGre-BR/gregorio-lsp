# Changelog

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
