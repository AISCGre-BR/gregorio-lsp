# Contributing to Gregorio LSP

Thank you for your interest in contributing to Gregorio LSP! This document provides guidelines and information for contributors.

## Code of Conduct

Be respectful and inclusive. We welcome contributions from everyone.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- GABC sample file that demonstrates the issue
- Your environment (OS, Node version, etc.)

### Suggesting Features

Feature suggestions are welcome! Please create an issue describing:
- The feature you'd like to see
- Why it would be useful
- How it might work
- Any examples from other tools

### Pull Requests

1. **Fork the repository** and create a branch from `main`
2. **Make your changes** with clear, focused commits
3. **Add tests** for new functionality
4. **Update documentation** as needed
5. **Ensure tests pass**: `npm test`
6. **Ensure lint passes**: `npm run lint`
7. **Submit the pull request**

### Development Setup

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for detailed setup instructions.

Quick start:
```bash
git clone <your-fork>
cd gregorio-lsp
npm install
npm run build
npm test
```

## Coding Standards

### TypeScript Style

- Use TypeScript strict mode
- Prefer `const` over `let`, avoid `var`
- Use meaningful variable names
- Add JSDoc comments for public APIs
- Keep functions focused and small

### Code Organization

- One class/interface per file when possible
- Group related functionality in directories
- Export public API from index files
- Keep implementation details private

### Testing

- Write tests for all new features
- Test both success and failure cases
- Aim for 80%+ code coverage
- Use descriptive test names

Example:
```typescript
describe('validateQuilismaFollowedByLowerPitch', () => {
  it('should warn when quilisma is followed by lower pitch', () => {
    // Test implementation
  });
  
  it('should not warn when quilisma is followed by higher pitch', () => {
    // Test implementation
  });
});
```

### Commit Messages

Use conventional commit format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(validation): add virga strata validation rule

Implements warning for virga strata followed by equal or higher 
pitch notes, as specified in Gregorio compiler documentation.

Closes #42
```

```
fix(parser): handle multiline headers correctly

Previously, multiline headers were not parsed correctly when they
contained semicolons. This fixes the parsing logic to properly
handle the ;; terminator.

Fixes #38
```

## Project Structure

```
gregorio-lsp/
├── src/
│   ├── server.ts                 # Main LSP server
│   ├── parser/                   # Parsing logic
│   │   ├── types.ts
│   │   ├── gabc-parser.ts
│   │   └── tree-sitter-integration.ts
│   ├── validation/               # Validation rules
│   │   ├── rules.ts
│   │   └── validator.ts
│   └── __tests__/                # Tests
├── docs/                         # Documentation
├── examples/                     # Example GABC files
└── dist/                         # Compiled output
```

## Adding New Features

### Adding a Validation Rule

1. **Define the rule** in `src/validation/rules.ts`:
```typescript
export const validateMyRule: ValidationRule = {
  name: 'my-rule',
  severity: 'warning',
  validate: (doc: ParsedDocument): ParseError[] => {
    // Implementation
    return errors;
  }
};
```

2. **Add to rules array**:
```typescript
export const allValidationRules: ValidationRule[] = [
  // ... existing rules
  validateMyRule
];
```

3. **Write tests** in `src/__tests__/validation-rules.test.ts`

4. **Update documentation** in README and API docs

### Adding Parser Support

1. **Update types** in `src/parser/types.ts` if needed
2. **Implement parsing** in `src/parser/gabc-parser.ts`
3. **Add tests** in `src/__tests__/gabc-parser.test.ts`
4. **Update tree-sitter integration** if applicable

### Adding LSP Capability

1. **Advertise capability** in `onInitialize`
2. **Implement handler** (e.g., `connection.onDefinition`)
3. **Add tests**
4. **Document in API docs**

## Testing Guidelines

### Running Tests

```bash
# All tests
npm test

# Watch mode
npm test -- --watch

# With coverage
npm test -- --coverage

# Specific test
npm test -- --testNamePattern="validateNameHeader"
```

### Test Structure

```typescript
describe('FeatureName', () => {
  describe('specific functionality', () => {
    it('should do something', () => {
      // Arrange
      const input = createTestInput();
      
      // Act
      const result = functionUnderTest(input);
      
      // Assert
      expect(result).toBe(expectedValue);
    });
  });
});
```

## Documentation

### Code Documentation

Use JSDoc for public APIs:

```typescript
/**
 * Parse GABC text into a document structure.
 * 
 * @param text - The GABC source code to parse
 * @returns Parsed document with headers, notation, and errors
 * 
 * @example
 * ```typescript
 * const parser = new GabcParser(gabcText);
 * const doc = parser.parse();
 * console.log(doc.headers.get('name'));
 * ```
 */
parse(text: string): ParsedDocument {
  // Implementation
}
```

### User Documentation

- Keep README.md up to date with features
- Update API.md for API changes
- Add examples for new features
- Update CHANGELOG.md

## Release Process

Releases are managed by maintainers:

1. Update version in `package.json`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v0.x.0`
4. Push tag: `git push origin v0.x.0`
5. Publish to npm (when applicable)

## Getting Help

- **Issues**: Use GitHub issues for bugs and features
- **Discussions**: For questions and general discussion
- **Documentation**: Check docs/ directory first

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Gregorio LSP! 🎵
