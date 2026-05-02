# Quickstart

## Pré-requisitos

- `rustc` e `cargo` (edição 2021, MSRV 1.70+).
- Opcional: `tree-sitter-gregorio` em `../tree-sitter-gregorio` para a feature
  `tree-sitter`.

## Build

```bash
cargo build --release
```

Os binários ficam em `target/release/`:

- `gregorio-lsp` — servidor LSP via stdio.
- `gregolint` — linter de linha de comando.

Para habilitar a integração tree-sitter:

```bash
cargo build --release --features tree-sitter
```

## Lint via CLI

```bash
gregolint examples/kyrie-xvi.gabc
gregolint -s warning -i quilisma-missing-connector arquivo.gabc
cat arquivo.gabc | gregolint -
```

Códigos de saída:

- `0` — nenhuma diagnóstico de severidade `error`.
- `1` — pelo menos um erro.

## LSP no editor

Configure o language server `gregorio-lsp` para a linguagem `gabc`
(extensão `.gabc`). Exemplo (Helix):

```toml
[language-server.gregorio-lsp]
command = "gregorio-lsp"

[[language]]
name = "gabc"
file-types = ["gabc"]
language-servers = ["gregorio-lsp"]
```

A configuração de lint é enviada via `workspace/didChangeConfiguration`:

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

## Testes

```bash
cargo test
```
