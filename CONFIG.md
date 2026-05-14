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

Known rule codes (use any of these in `ignoreRules`):

| Code | Severity | Description |
|---|---|---|
| `name-header` | Warning | Missing or empty `name` header |
| `first-syllable-line-break` | Error | Line break on the first syllable |
| `first-syllable-clef-change` | Error | Clef change on the first syllable |
| `nabc-without-header` | Error | NABC pipe `\|` without `nabc-lines` header |
| `quilisma-lower-pitch` | Warning | Quilisma followed by equal or lower pitch |
| `quilisma-pes-higher-pitch` | Warning | Quilisma-pes preceded by equal or higher pitch |
| `virga-strata-higher-pitch` | Warning | Virga strata followed by equal or higher pitch |
| `staff-lines` | Error | `staff-lines` value outside the 2–5 range |
| `balanced-pitch-descriptors-fused-glyphs` | Warning | NABC fused glyphs with unbalanced pitch count |
| `modifiers-in-fused-glyphs` | Warning | Modifiers only allowed on the last glyph in a fusion chain |
| `multi-word-syllable` | Warning | Multiple space-separated words share a single note group; auto-fixable to `word1() word2(notes)` |
| `pes-quadratum-missing-note` | Warning | `q` modifier requires a subsequent note |
| `quilisma-missing-note` | Warning | Quilisma requires a subsequent note |
| `oriscus-scapus-isolated` | Warning | `O` requires both a preceding and a subsequent note |
| `oriscus-scapus-missing-preceding` | Warning | `O` requires a preceding note |
| `oriscus-scapus-missing-subsequent` | Warning | `O` requires a subsequent note |
| `quilisma-equal-or-lower` | Warning | Quilisma followed by lower or equal pitch |
| `quilisma-pes-preceded-by-higher` | Warning | Quilisma-pes preceded by higher or equal pitch |
| `virga-strata-equal-or-higher` | Warning | Virga strata followed by higher or equal pitch |
| `pes-stratus-equal-or-higher` | Warning | Pes stratus ending with a higher or equal following note |
| `nabc-conflicting-liquescence` | Warning | Both `>` and `~` on the same NABC descriptor |
| `nabc-invalid-pitch` | Warning | Invalid NABC pitch letter |
| `quilisma-missing-connector` | Info | Suggests `@` or `!` before a quilisma in a 3+ note group |

Full source: [`src/validation/rules.rs`](src/validation/rules.rs) and
[`src/validation/semantic.rs`](src/validation/semantic.rs).
