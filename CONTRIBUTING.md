# Contributing

Contribuições são bem-vindas. Antes de abrir um PR:

1. Garanta que `cargo build` e `cargo test` passam localmente.
2. Rode `cargo fmt` e `cargo clippy --all-targets` (warnings devem ser
   tratados ou justificados).
3. Para alterações no parser ou nas regras de validação, adicione testes em
   `tests/` cobrindo o novo comportamento.
4. Para mudanças que afetem o protocolo LSP, descreva o impacto em editores
   típicos (Helix, Neovim, VS Code) na descrição do PR.

## Estilo de commits

Usamos mensagens curtas no imperativo; quando útil, prefixos
[Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`,
`refactor:`, `docs:`, `test:`).

## Assinatura

Commits assinados com GPG são preferidos, especialmente para mudanças em
código de validação ou de servidor.

## Estrutura

Veja [README.md](README.md) para visão geral da estrutura de módulos.
