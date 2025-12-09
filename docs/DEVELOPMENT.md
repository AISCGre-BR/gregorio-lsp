# Gregorio LSP - Development Guide

## Project Structure

```
gregorio-lsp/
├── src/
│   ├── server.ts                     # Main LSP server entry point
│   ├── parser/
│   │   ├── types.ts                  # Core type definitions
│   │   ├── gabc-parser.ts            # Fallback TypeScript parser
│   │   └── tree-sitter-integration.ts # Tree-sitter wrapper
│   ├── validation/
│   │   ├── rules.ts                  # Individual validation rules
│   │   └── validator.ts              # Validation orchestrator
│   └── __tests__/                    # Test files
├── docs/                             # Documentation files
├── dist/                             # Compiled output (generated)
└── node_modules/                     # Dependencies (generated)
```

## Building from Source

### Prerequisites

- Node.js >= 16.0.0
- npm >= 7.0.0
- TypeScript >= 5.3.0

### Steps

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd gregorio-lsp
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Build tree-sitter-gregorio (optional, for full functionality)**
   ```bash
   cd ../tree-sitter-gregorio
   npm install
   npm run build
   cd ../gregorio-lsp
   ```

4. **Build the LSP server**
   ```bash
   npm run build
   ```

5. **Run tests**
   ```bash
   npm test
   ```

## Development Workflow

### Watch Mode

For active development, use watch mode to automatically recompile on changes:

```bash
npm run watch
```

### Running Tests

```bash
# Run all tests
npm test

# Run tests in watch mode
npm test -- --watch

# Run tests with coverage
npm test -- --coverage
```

### Linting

```bash
# Check for lint errors
npm run lint

# Auto-fix lint errors
npm run lint -- --fix
```

## Adding New Validation Rules

To add a new validation rule:

1. **Create the rule in `src/validation/rules.ts`**

```typescript
export const validateMyNewRule: ValidationRule = {
  name: 'my-new-rule',
  severity: 'warning',
  validate: (doc: ParsedDocument): ParseError[] => {
    const errors: ParseError[] = [];
    
    // Your validation logic here
    
    return errors;
  }
};
```

2. **Add to the rules array**

```typescript
export const allValidationRules: ValidationRule[] = [
  // ... existing rules
  validateMyNewRule
];
```

3. **Write tests in `src/__tests__/validation-rules.test.ts`**

```typescript
describe('validateMyNewRule', () => {
  it('should detect the error condition', () => {
    const doc: ParsedDocument = {
      // Create test document
    };

    const errors = validateMyNewRule.validate(doc);
    expect(errors.length).toBe(1);
  });
});
```

## Parser Extension

### Extending the TypeScript Fallback Parser

The fallback parser is in `src/parser/gabc-parser.ts`. To add new syntax support:

1. **Update types in `src/parser/types.ts`** if needed
2. **Add parsing logic in the appropriate method**
3. **Write comprehensive tests**

### Extending Tree-sitter Integration

Tree-sitter integration is in `src/parser/tree-sitter-integration.ts`. To add new node types:

1. **Add node type detection methods**
2. **Add extraction methods for new syntax elements**
3. **Update the main parser to use new methods**

## LSP Capabilities

### Current Capabilities

- ✅ `textDocument/didOpen`
- ✅ `textDocument/didChange`
- ✅ `textDocument/publishDiagnostics`
- ✅ `textDocument/hover`
- ✅ `textDocument/completion`
- ✅ `textDocument/documentSymbol`

### Adding New Capabilities

To add a new LSP capability:

1. **Update `onInitialize` in `src/server.ts`** to advertise the capability
2. **Implement the handler** (e.g., `connection.onDefinition`)
3. **Add tests** for the new feature
4. **Update documentation**

Example:

```typescript
// In onInitialize result
capabilities: {
  definitionProvider: true
}

// Handler
connection.onDefinition((params) => {
  // Implementation
});
```

## Debugging

### VS Code Launch Configuration

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "node",
      "request": "launch",
      "name": "Debug LSP Server",
      "program": "${workspaceFolder}/src/server.ts",
      "preLaunchTask": "npm: build",
      "outFiles": ["${workspaceFolder}/dist/**/*.js"],
      "sourceMaps": true
    }
  ]
}
```

### Logging

The server logs to the LSP client console. Enable verbose logging:

```typescript
connection.console.log('Debug message');
connection.console.warn('Warning message');
connection.console.error('Error message');
```

## Testing Strategy

### Unit Tests

- **Parser tests**: Verify correct parsing of GABC syntax
- **Validation tests**: Verify each validation rule works correctly
- **Integration tests**: Test full document validation flow

### Test Coverage Goals

- Minimum 80% code coverage
- 100% coverage for validation rules
- Test both success and failure cases

### Running Specific Tests

```bash
# Run tests matching a pattern
npm test -- --testNamePattern="validateNameHeader"

# Run tests in a specific file
npm test -- gabc-parser.test.ts
```

## Performance Considerations

### Parser Performance

- Tree-sitter parser: ~0.1-1ms for typical documents
- Fallback parser: ~1-10ms for typical documents
- Use tree-sitter when available for best performance

### Validation Performance

- Run validation on document changes (incremental)
- Debounce validation for rapid typing
- Cache parse results when possible

### Memory Usage

- Clear diagnostics for closed documents
- Limit stored document history
- Profile with `node --inspect`

## Release Process

1. **Update version in `package.json`**
2. **Update CHANGELOG.md** with changes
3. **Run full test suite**: `npm test`
4. **Build production**: `npm run build`
5. **Tag release**: `git tag v0.1.0`
6. **Publish**: `npm publish` (when ready)

## Contributing Guidelines

### Code Style

- Use TypeScript strict mode
- Follow existing code formatting
- Use ESLint rules defined in `.eslintrc.json`
- Write descriptive commit messages

### Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes with tests
4. Ensure tests pass: `npm test`
5. Ensure lint passes: `npm run lint`
6. Submit pull request with description

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Example:
```
feat(validation): add virga strata validation rule

Implements warning for virga strata followed by equal or higher pitch
notes, as specified in the Gregorio compiler documentation.

Closes #42
```

## Resources

- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [Gregorio Documentation](http://gregorio-project.github.io/gregorio/)
- [GABC Syntax Guide](./docs/GABC_SYNTAX_SPECIFICATION.md)
