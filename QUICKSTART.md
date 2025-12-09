# Gregorio LSP - Quick Start Guide

Get started with the Gregorio LSP in 5 minutes!

## Prerequisites

- Node.js 16+ installed
- npm 7+ installed
- Basic knowledge of GABC notation

## Installation

### Step 1: Clone the Repository

```bash
cd ~/Documentos
git clone <repository-url> gregorio-lsp
cd gregorio-lsp
```

### Step 2: Install Dependencies

```bash
npm install
```

This will:
- Install all required npm packages
- Check for tree-sitter-gregorio (optional)
- Verify Node.js version

### Step 3: Build the Project

```bash
npm run build
```

This compiles TypeScript to JavaScript in the `dist/` directory.

### Step 4: Run Tests (Optional)

```bash
npm test
```

Verify everything works correctly.

## Basic Usage

### Command Line Testing

Test the parser directly:

```bash
node -e "
const { GabcParser } = require('./dist/parser/gabc-parser');
const text = 'name: Test;\\n%%\\n(c4) Te(f)st(g)';
const parser = new GabcParser(text);
const doc = parser.parse();
console.log('Headers:', Array.from(doc.headers.entries()));
console.log('Syllables:', doc.notation.syllables.length);
"
```

### Starting the LSP Server

```bash
node dist/server.js --stdio
```

The server will listen on stdin/stdout for LSP protocol messages.

## Testing with Example Files

### Valid GABC File

```bash
cat examples/kyrie-xvi.gabc
```

This is a complete, valid Kyrie chant from the Graduale Romanum.

### With NABC

```bash
cat examples/nabc-example.gabc
```

Shows NABC (adiastematic notation) integration.

### Error Detection

```bash
cat examples/errors-example.gabc
```

Intentional errors to demonstrate validation.

## Validating Files Programmatically

Create a file `test-validator.js`:

```javascript
const fs = require('fs');
const { GabcParser } = require('./dist/parser/gabc-parser');
const { DocumentValidator } = require('./dist/validation/validator');

// Read a GABC file
const text = fs.readFileSync('examples/kyrie-xvi.gabc', 'utf-8');

// Parse it
const parser = new GabcParser(text);
const doc = parser.parse();

// Validate it
const validator = new DocumentValidator();
const errors = validator.validate(doc);

// Print results
console.log(`Found ${errors.length} issues:`);
errors.forEach(error => {
  console.log(`[${error.severity}] Line ${error.range.start.line}: ${error.message}`);
});
```

Run it:
```bash
node test-validator.js
```

## Integration with VS Code

### Option 1: Use existing LSP client extension

If you have a generic LSP client extension installed:

1. Configure it to use `gregorio-lsp`
2. Point to `dist/server.js`
3. Set file pattern to `*.gabc`

### Option 2: Create custom extension

See `docs/DEVELOPMENT.md` for creating a VS Code extension wrapper.

## Common Tasks

### Watch Mode (Development)

```bash
npm run watch
```

Automatically recompiles on file changes.

### Running Specific Tests

```bash
npm test -- --testNamePattern="validateNameHeader"
```

### Linting

```bash
npm run lint
```

### Cleaning Build

```bash
npm run clean
npm run build
```

## Troubleshooting

### "tree-sitter-gregorio not found"

This is normal! The LSP will use the TypeScript fallback parser. For better performance:

```bash
cd ../tree-sitter-gregorio
npm install
cd ../gregorio-lsp
npm install
```

### "Cannot find module"

Make sure you've run:
```bash
npm install
npm run build
```

### Tests Failing

Check Node.js version:
```bash
node --version  # Should be 16+
```

Update dependencies:
```bash
rm -rf node_modules package-lock.json
npm install
```

### TypeScript Errors

Clean and rebuild:
```bash
npm run clean
npm run build
```

## Editor Configuration Examples

### VS Code `settings.json`

```json
{
  "gregorioLsp.validation.enabledRules": [
    "name-header",
    "quilisma-lower-pitch",
    "nabc-without-header"
  ],
  "gregorioLsp.parser.preferTreeSitter": true
}
```

### Neovim with `nvim-lspconfig`

```lua
require'lspconfig'.gregorio_lsp.setup{
  cmd = { "node", "/path/to/gregorio-lsp/dist/server.js", "--stdio" },
  filetypes = { "gabc" },
  root_dir = require'lspconfig'.util.root_pattern(".git", "*.gabc"),
}
```

## Learning Resources

### Documentation
- `README.md` - Features and installation
- `docs/API.md` - Complete API reference
- `docs/DEVELOPMENT.md` - Development guide
- `docs/GABC_SYNTAX_SPECIFICATION.md` - GABC syntax reference
- `docs/NABC_SYNTAX_SPECIFICATION.md` - NABC syntax reference

### Examples
- `examples/` - Example GABC files
- `src/__tests__/` - Test files show usage patterns

### External Resources
- [Gregorio Project](http://gregorio-project.github.io/)
- [GABC Notation Tutorial](http://gregorio-project.github.io/gabc/)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)

## Next Steps

1. **Explore examples**: Look at files in `examples/`
2. **Read API docs**: Check out `docs/API.md`
3. **Try validation**: Test with your own GABC files
4. **Integrate with editor**: Set up in VS Code or your preferred editor
5. **Contribute**: See `CONTRIBUTING.md` for guidelines

## Getting Help

- Check documentation in `docs/`
- Look at example files in `examples/`
- Review test files in `src/__tests__/`
- Open an issue on GitHub

## Quick Reference Card

```bash
# Installation
npm install                 # Install dependencies
npm run build              # Build project

# Development
npm run watch              # Watch mode
npm test                   # Run tests
npm run lint              # Check code quality

# Running
node dist/server.js --stdio   # Start LSP server

# Validation
const parser = new GabcParser(text);
const doc = parser.parse();
const validator = new DocumentValidator();
const errors = validator.validate(doc);
```

---

Happy chanting! 🎵
