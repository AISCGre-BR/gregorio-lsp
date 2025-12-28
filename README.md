# Gregorio LSP

Complete Language Server Protocol implementation for Gregorio GABC/NABC notation files. Provides semantic analysis, error detection, warnings, cross-reference checking, auto-completion, hover information, and real-time diagnostics for Gregorian chant notation.

## Features

### ✅ Dual Parser Architecture
- **Primary Parser**: Tree-sitter-gregorio integration for fast, accurate parsing
- **Fallback Parser**: Pure TypeScript implementation for maximum compatibility
- Automatic fallback when tree-sitter is unavailable

### 🔍 Comprehensive Error Detection

The LSP implements all errors and warnings specified in the Gregorio compiler documentation:

#### Compilation Errors (Blocking)
- Missing `%%` separator between headers and notation
- Line breaks on the first syllable
- Clef changes on the first syllable  
- Score initial in elision
- NABC pipes (`|`) without `nabc-lines` header
- Invalid syntax constructions
- Invalid number of staff lines (must be 2-5)
- Style tags (bold, italic, etc.) opened/closed incorrectly
- Forced center within elision
- Translation centering errors

#### Warnings (Non-blocking)
- Missing `name` header field
- Duplicate header definitions
- Musical warnings:
  - Quilisma followed by equal/lower pitch note
  - Quilisma-pes preceded by equal/higher pitch note
  - Virga strata followed by equal/higher pitch note

#### Informational Messages
- Missing connector `!` in quilismatic sequences (3+ notes)

### 🚀 LSP Capabilities

- **Diagnostics**: Real-time error and warning detection
- **Hover**: Show information about nodes and syntax elements
- **Completion**: Auto-completion for clefs, bars, headers, and notation
- **Document Symbols**: Navigate through headers and structure
- **Incremental Updates**: Fast updates on document changes

## Installation

### From npm (when published)

```bash
npm install -g gregorio-lsp
```

### From source

```bash
cd gregorio-lsp
npm install
npm run build
```

### With tree-sitter-gregorio integration

To enable the tree-sitter parser (recommended):

```bash
# Build tree-sitter-gregorio first
cd ../tree-sitter-gregorio
npm install

# Then install gregorio-lsp
cd ../gregorio-lsp
npm install
npm run build
```

## Usage

### With VS Code

Create a VS Code extension or use an existing one that can connect to language servers. Example configuration:

```json
{
  "languageServer": {
    "module": "/path/to/gregorio-lsp/dist/server.js",
    "transport": "stdio",
    "languages": ["gabc"]
  }
}
```

### Command Line

Run the server directly:

```bash
node dist/server.js --stdio
```

### Programmatic Usage

```typescript
import { GabcParser } from 'gregorio-lsp/parser/gabc-parser';
import { DocumentValidator } from 'gregorio-lsp/validation/validator';

const text = `name: Example;
%%
(c4) Ky(f)ri(g)e(h)`;

const parser = new GabcParser(text);
const doc = parser.parse();

const validator = new DocumentValidator();
const errors = validator.validate(doc);

console.log(errors);
```

## Configuration

The LSP can be configured through workspace settings (if supported by the client):

```json
{
  "gregorioLsp.validation.enabledRules": [
    "name-header",
    "first-syllable-line-break",
    "quilisma-lower-pitch",
    "nabc-without-header"
  ],
  "gregorioLsp.parser.preferTreeSitter": true
}
```

### Available Validation Rules

- `name-header`: Warn if name header is missing
- `duplicate-headers`: Warn about duplicate headers
- `first-syllable-line-break`: Error on line break in first syllable
- `first-syllable-clef-change`: Error on clef change in first syllable
- `nabc-without-header`: Error when NABC used without header
- `quilisma-lower-pitch`: Warn about quilisma followed by lower pitch
- `quilisma-pes-higher-pitch`: Warn about quilisma-pes preceded by higher pitch
- `virga-strata-higher-pitch`: Warn about virga strata followed by higher pitch
- `staff-lines`: Error on invalid staff line count

## Architecture

```
gregorio-lsp/
├── src/
│   ├── server.ts                    # Main LSP server
│   ├── parser/
│   │   ├── types.ts                 # Type definitions
│   │   ├── gabc-parser.ts           # TypeScript fallback parser
│   │   └── tree-sitter-integration.ts # Tree-sitter integration
│   ├── validation/
│   │   ├── rules.ts                 # Validation rules
│   │   └── validator.ts             # Validator orchestration
│   └── __tests__/
│       ├── gabc-parser.test.ts      # Parser tests
│       └── validation-rules.test.ts # Validation tests
├── docs/                            # Documentation
├── package.json
├── tsconfig.json
└── README.md
```

## Development

### Build

```bash
npm run build
```

### Watch Mode

```bash
npm run watch
```

### Run Tests

```bash
npm test
```

### Lint

```bash
npm run lint
```

## Documentation References

This implementation is based on:

- **GABC Syntax Specification** (`docs/GABC_SYNTAX_SPECIFICATION.md`)
- **NABC Syntax Specification** (`docs/NABC_SYNTAX_SPECIFICATION.md`)
- **Gregorio Compiler Errors and Warnings** (`docs/GREGORIO_COMPILER_ERRORS_AND_WARNINGS.md`)
- **Errors and Warnings Summary** (`docs/ERRORS_AND_WARNINGS_SUMMARY.md`)

## Integration with tree-sitter-gregorio

The LSP integrates seamlessly with the [tree-sitter-gregorio](../tree-sitter-gregorio) grammar:

1. **Automatic Detection**: Tries to load tree-sitter-gregorio at startup
2. **Graceful Fallback**: Falls back to TypeScript parser if unavailable
3. **Shared Types**: Uses compatible AST representations
4. **Performance**: Tree-sitter provides ~10-100x faster parsing

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## Known Issues

### Pitch Descriptor Validation in NABC

The validation rule for balanced pitch descriptors in fused NABC glyphs (`validateBalancedPitchDescriptorsInFusedGlyphs`) may produce false positive warnings when a NABC snippet contains two or more complex neume descriptors. This occurs because the parser currently validates each individual complex descriptor separately, but does not account for the scope boundaries between adjacent complex descriptors in the same NABC snippet.

**Example that triggers false positive:**
```gabc
(g|vihk pe) % Two separate complex descriptors: 'vihk' and 'pe'
```

The warning states that pitch descriptors must be balanced in fused glyphs, but this applies only when the `!` fusion operator is used *within* a single complex descriptor (e.g., `vihk!tahk`). Adjacent complex descriptors without `!` should not trigger this validation.

**Workaround:** This is a cosmetic issue that does not affect the correctness of your GABC notation. The warning can be safely ignored in cases where your NABC snippet contains multiple space-separated complex descriptors without fusion operators.

**Status:** This limitation is scheduled for resolution in a future update that will improve the validation rule's scope detection.

## License

MIT License - see LICENSE file for details

## Authors

AISC Gre-BR

## Related Projects

- [tree-sitter-gregorio](../tree-sitter-gregorio) - Tree-sitter grammar for Gregorio notation
- [Gregorio](https://github.com/gregorio-project/gregorio) - Gregorian chant score engraving system

## Changelog

### 0.1.0 (Initial Release)

- ✅ Complete GABC parser with TypeScript fallback
- ✅ Tree-sitter-gregorio integration
- ✅ Full error and warning detection per Gregorio compiler spec
- ✅ LSP implementation with diagnostics, hover, and completion
- ✅ Comprehensive test suite
- ✅ Musical validation rules (quilisma, virga strata, etc.)
- ✅ NABC syntax support and validation
