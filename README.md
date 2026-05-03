# gregorio-lsp

Language Server e linter para arquivos GABC (Gregorio) e notação NABC, escritos em Rust.

Esta é a reescrita Rust do projeto original em TypeScript, fornecendo o mesmo
conjunto de funcionalidades com binários nativos:

- `gregorio-lsp` — servidor LSP via stdio (baseado em [tower-lsp](https://github.com/ebkalderon/tower-lsp));
- `gregolint` — CLI de lint para arquivos GABC e leitura via `stdin`.

## Funcionalidades

- Parser GABC completo (cabeçalhos, sílabas, claves, barras, custos, atributos, modificadores e quebras de linha);
- Parser NABC (St. Gall, Laon, modificadores, subpunctis/prepunctis, letras significativas e fusões `!`);
- 10 regras de validação estrutural;
- Analisador semântico com verificações musicais (quilisma, oriscus scapus, pes quadratum, virga strata etc.);
- Suporte opcional a [tree-sitter-gregorio](../tree-sitter-gregorio) via feature `tree-sitter`;
- Suíte de testes de integração com 29 casos cobrindo parser, NABC e validação.

## Compilação

Requer `cargo` e `rustc` (edição 2021).

```bash
cargo build --release
cargo test
cargo build --release --features tree-sitter   # ativa integração tree-sitter
```

Os binários ficam em `target/release/gregorio-lsp` e `target/release/gregolint`.

## Uso do servidor LSP

O binário `gregorio-lsp` lê e escreve mensagens LSP em `stdio`. Configure seu
editor para iniciá-lo como language server para a linguagem `gabc`.

Exemplo de configuração JSON enviada via `workspace/didChangeConfiguration`:

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

## Uso do CLI `gregolint`

```bash
gregolint examples/kyrie-xvi.gabc
gregolint -s warning -i quilisma-missing-connector arquivo.gabc
cat arquivo.gabc | gregolint -
```

Saída no formato `arquivo:linha:coluna: severidade [código] mensagem`. O
processo retorna `1` se houver pelo menos um erro.

## Estrutura

```
src/
  lib.rs                  # API pública
  lint.rs                 # pipeline de lint reutilizável
  parser/
    types.rs              # tipos do AST
    gabc.rs               # parser GABC
    nabc.rs               # parser NABC
  validation/
    rules.rs              # regras estruturais
    semantic.rs           # analisador semântico
    validator.rs          # orquestrador
  tree_sitter_integration.rs   # opcional, feature `tree-sitter`
  bin/
    server.rs             # binário gregorio-lsp
    lint.rs               # binário gregolint
tests/
  gabc_parser.rs
  nabc_parser.rs
  validation.rs
```

## Licença

MIT — Copyright (c) 2026 AISCGre Brasil. Veja [LICENSE](LICENSE).
