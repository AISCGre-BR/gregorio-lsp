# Configuração

O servidor `gregorio-lsp` aceita configuração via mensagem LSP
`workspace/didChangeConfiguration`. As chaves são todas opcionais.

```jsonc
{
  "linting": {
    // Liga ou desliga totalmente o lint. Default: true.
    "enabled": true,

    // Severidade mínima publicada como diagnóstico:
    // "error" | "warning" | "info". Default: "info".
    "severity": "warning",

    // Quando true, o lint só é executado em didSave (não em didChange).
    // Default: false.
    "onSave": false,

    // Códigos de regra a ignorar (ex.: "quilisma-missing-connector").
    "ignoreRules": []
  }
}
```

Códigos de regras conhecidos estão definidos em
[`src/validation/rules.rs`](src/validation/rules.rs) e
[`src/validation/semantic.rs`](src/validation/semantic.rs).
