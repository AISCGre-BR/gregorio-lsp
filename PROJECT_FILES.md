# Gregorio LSP - Project File Structure

Complete list of files created for the Gregorio LSP project.

## Root Files

```
gregorio-lsp/
├── package.json                    # NPM package configuration
├── tsconfig.json                   # TypeScript compiler configuration
├── jest.config.js                  # Jest test configuration
├── .eslintrc.json                  # ESLint configuration
├── .gitignore                      # Git ignore rules
├── README.md                       # Main documentation
├── LICENSE                         # MIT License (existing)
├── CHANGELOG.md                    # Version history
└── CONTRIBUTING.md                 # Contribution guidelines
```

## Source Files (`src/`)

```
src/
├── server.ts                       # Main LSP server implementation
│
├── parser/
│   ├── types.ts                    # Type definitions for parser
│   ├── gabc-parser.ts              # TypeScript fallback parser
│   └── tree-sitter-integration.ts  # Tree-sitter integration wrapper
│
├── validation/
│   ├── rules.ts                    # Individual validation rules
│   └── validator.ts                # Validation orchestrator
│
└── __tests__/
    ├── gabc-parser.test.ts         # Parser unit tests
    └── validation-rules.test.ts    # Validation rules tests
```

## Documentation (`docs/`)

```
docs/
├── ERRORS_AND_WARNINGS_SUMMARY.md      # Compiler errors summary (existing)
├── GABC_SYNTAX_SPECIFICATION.md        # GABC syntax spec (existing)
├── GREGORIO_COMPILER_ERRORS_AND_WARNINGS.md  # Compiler errors (existing)
├── NABC_SYNTAX_SPECIFICATION.md        # NABC syntax spec (existing)
├── API.md                              # API documentation (new)
└── DEVELOPMENT.md                      # Development guide (new)
```

## Examples (`examples/`)

```
examples/
├── README.md                       # Examples documentation
├── kyrie-xvi.gabc                  # Valid GABC example
├── nabc-example.gabc               # NABC example
└── errors-example.gabc             # Error demonstration
```

## Scripts (`scripts/`)

```
scripts/
├── postinstall.js                  # Post-install verification script
└── build.sh                        # Build script
```

## Generated Files (not in repository)

These files are generated during build/test and should not be committed:

```
dist/                               # Compiled JavaScript output
├── server.js
├── server.d.ts
├── server.js.map
├── parser/
│   ├── types.js
│   ├── types.d.ts
│   ├── gabc-parser.js
│   ├── gabc-parser.d.ts
│   └── tree-sitter-integration.js
└── validation/
    ├── rules.js
    ├── validator.js
    └── ...

node_modules/                       # NPM dependencies
coverage/                           # Test coverage reports
*.log                               # Log files
.vscode/                            # VS Code settings (user-specific)
```

## File Purposes

### Configuration Files

- **package.json**: Defines project metadata, dependencies, and scripts
- **tsconfig.json**: TypeScript compiler options (strict mode, ES2020 target)
- **jest.config.js**: Test framework configuration
- **.eslintrc.json**: Code quality and style rules
- **.gitignore**: Files to exclude from version control

### Core Implementation

- **server.ts**: LSP server entry point, handles all LSP protocol messages
- **parser/types.ts**: Type definitions for AST and document structure
- **parser/gabc-parser.ts**: Full GABC parser implementation in TypeScript
- **parser/tree-sitter-integration.ts**: Wrapper for tree-sitter-gregorio
- **validation/rules.ts**: All validation rules (errors, warnings, info)
- **validation/validator.ts**: Coordinates validation rule execution

### Testing

- **__tests__/gabc-parser.test.ts**: Tests for parser functionality
- **__tests__/validation-rules.test.ts**: Tests for validation rules

### Documentation

- **README.md**: User-facing documentation, features, installation
- **CHANGELOG.md**: Version history and release notes
- **CONTRIBUTING.md**: Guidelines for contributors
- **docs/API.md**: Complete API reference
- **docs/DEVELOPMENT.md**: Development workflow and architecture

### Examples

- **examples/kyrie-xvi.gabc**: Real-world valid GABC file
- **examples/nabc-example.gabc**: NABC integration demonstration
- **examples/errors-example.gabc**: Error detection demonstration
- **examples/README.md**: Examples documentation

### Scripts

- **scripts/postinstall.js**: Verifies installation and dependencies
- **scripts/build.sh**: Custom build script with additional processing

## Total Files Created

- **Root**: 9 files
- **src/**: 7 files (3 modules + 4 test/parser/validation files)
- **docs/**: 2 new files (+ 4 existing)
- **examples/**: 4 files
- **scripts/**: 2 files

**Total new files**: ~24 files
**Lines of code**: ~3,500+ lines (excluding tests and docs)

## Architecture Overview

```
┌─────────────────────────────────────────┐
│           LSP Client (Editor)           │
└──────────────────┬──────────────────────┘
                   │ LSP Protocol
                   │
┌──────────────────▼──────────────────────┐
│          server.ts (LSP Server)         │
│  • textDocument/didOpen                 │
│  • textDocument/didChange               │
│  • textDocument/hover                   │
│  • textDocument/completion              │
│  • publishDiagnostics                   │
└──────┬──────────────────────┬───────────┘
       │                      │
       │                      │
┌──────▼─────────┐   ┌────────▼──────────┐
│ Parser Module  │   │ Validation Module │
│                │   │                   │
│ • Tree-sitter  │   │ • Rules           │
│ • TypeScript   │   │ • Validator       │
│   Fallback     │   │                   │
└────────────────┘   └───────────────────┘
```

## Integration Points

### With tree-sitter-gregorio

- Located at: `../tree-sitter-gregorio`
- Imported as: `require('tree-sitter-gregorio')`
- Used by: `src/parser/tree-sitter-integration.ts`
- Fallback: TypeScript parser if unavailable

### With VS Code / Editors

- Communication: stdin/stdout via LSP protocol
- Document sync: Incremental
- Capabilities advertised in `onInitialize`

## Next Steps for Usage

1. Install dependencies: `npm install`
2. Build project: `npm run build`
3. Run tests: `npm test`
4. Start server: `node dist/server.js --stdio`

For editor integration, see README.md and docs/DEVELOPMENT.md.
