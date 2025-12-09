# Gregorio LSP - Examples

This directory contains example GABC files to demonstrate the LSP features and validation.

## Files

### `kyrie-xvi.gabc`
A complete, valid GABC file transcribing the Kyrie XVI from the Graduale Romanum. This demonstrates:
- Proper header structure
- Complex neume notation
- Style tags (`<i>`, `<sp>`)
- Bars and separators
- Musical notation syntax

### `nabc-example.gabc`
An example showing NABC (adiastematic notation) integration:
- `nabc-lines: 1;` header
- NABC snippets separated by `|`
- Mixed GABC/NABC notation

### `errors-example.gabc`
A file with intentional errors to demonstrate LSP error detection:
- Missing `name` header (warning)
- Line break on first syllable (error)
- NABC without header (error)
- Quilisma followed by lower pitch (warning)
- Clef change on first syllable (error)

## Testing the LSP

Open these files in an editor with the Gregorio LSP enabled to see:

1. **Real-time diagnostics** - Errors and warnings appear as you type
2. **Hover information** - Hover over notation elements to see details
3. **Auto-completion** - Type `(` to get clef suggestions
4. **Document symbols** - View headers in the outline/symbol view

## Running Validation Manually

You can also validate these files programmatically:

```typescript
import { readFileSync } from 'fs';
import { GabcParser } from '../src/parser/gabc-parser';
import { DocumentValidator } from '../src/validation/validator';

const text = readFileSync('examples/kyrie-xvi.gabc', 'utf-8');
const parser = new GabcParser(text);
const doc = parser.parse();

const validator = new DocumentValidator();
const errors = validator.validate(doc);

console.log(`Found ${errors.length} issues`);
errors.forEach(error => {
  console.log(`[${error.severity}] Line ${error.range.start.line}: ${error.message}`);
});
```

## Expected Validation Results

### `kyrie-xvi.gabc`
- ✅ No errors
- ✅ No warnings
- All headers present and valid
- All notation syntactically correct

### `nabc-example.gabc`
- ✅ No errors
- ✅ No warnings
- NABC properly declared and used

### `errors-example.gabc`
- ⚠️ 1 warning: Missing name header
- ❌ 3 errors:
  - Line break on first syllable
  - NABC pipe without nabc-lines header
  - Clef change on first syllable
- ⚠️ 1 warning: Quilisma followed by lower pitch

## Creating Your Own Examples

To create valid GABC files:

1. Always include the `name:` header
2. Separate headers from notation with `%%`
3. Enclose notation in parentheses: `(c4) Text(notes)`
4. If using NABC, declare `nabc-lines: 1;` in headers
5. Use proper clef notation: `c1`-`c4`, `f1`-`f4`, `cb1`-`cb4`

For more syntax details, see `docs/GABC_SYNTAX_SPECIFICATION.md`.
