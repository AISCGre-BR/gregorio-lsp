# Configuration

The `gregorio-lsp` server accepts configuration via the LSP
`workspace/didChangeConfiguration` message. All keys are optional.

```jsonc
{
  "linting": {
    // Enables or disables linting entirely. Default: true.
    "enabled": true,

    // Minimum severity published as a diagnostic:
    // "error" | "warning" | "info". Default: "info".
    "severity": "warning",

    // When true, linting only runs on didSave (not on didChange).
    // Default: false.
    "onSave": false,

    // Rule codes to ignore (e.g. "quilisma-missing-connector").
    "ignoreRules": []
  }
}
```

Known rule codes are defined in
[`src/validation/rules.rs`](src/validation/rules.rs) and
[`src/validation/semantic.rs`](src/validation/semantic.rs).
