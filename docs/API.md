# Gregorio LSP - API Documentation

## Parser API

### GabcParser

The fallback TypeScript parser for GABC files.

```typescript
import { GabcParser } from 'gregorio-lsp/parser/gabc-parser';

const parser = new GabcParser(text: string);
const document = parser.parse();
```

#### Methods

##### `parse(): ParsedDocument`

Parses the input text and returns a parsed document.

**Returns**: `ParsedDocument` containing headers, notation, comments, and errors

**Example**:
```typescript
const text = `name: Example;
%%
(c4) Ky(f)ri(g)e(h)`;

const parser = new GabcParser(text);
const doc = parser.parse();

console.log(doc.headers.get('name')); // "Example"
console.log(doc.notation.syllables.length); // Number of syllables
```

##### `addError(message: string, range: Range, severity?: 'error' | 'warning' | 'info'): void`

Manually add an error to the parser's error list.

**Parameters**:
- `message`: Error message
- `range`: Location of the error
- `severity`: Error severity (default: 'error')

---

### TreeSitterParser

Wrapper for tree-sitter-gregorio integration.

```typescript
import { treeSitterParser } from 'gregorio-lsp/parser/tree-sitter-integration';

const tree = treeSitterParser.parse(text);
```

#### Methods

##### `isTreeSitterAvailable(): boolean`

Check if tree-sitter-gregorio is available.

**Returns**: `true` if tree-sitter can be used, `false` otherwise

##### `parse(text: string): Parser.Tree | null`

Parse text using tree-sitter.

**Parameters**:
- `text`: GABC source code

**Returns**: Parse tree or `null` on error

##### `extractErrors(tree: Parser.Tree): ParseError[]`

Extract syntax errors from a parse tree.

**Parameters**:
- `tree`: Tree-sitter parse tree

**Returns**: Array of parse errors

##### `findNodeAt(tree: Parser.Tree, position: Position): Parser.SyntaxNode | null`

Find the syntax node at a specific position.

**Parameters**:
- `tree`: Parse tree
- `position`: Document position

**Returns**: Syntax node at position or `null`

##### `extractHeaders(tree: Parser.Tree, text: string): Map<string, string>`

Extract headers from a parse tree.

**Parameters**:
- `tree`: Parse tree
- `text`: Source text

**Returns**: Map of header names to values

---

## Validation API

### DocumentValidator

Orchestrates validation rules and produces diagnostics.

```typescript
import { DocumentValidator } from 'gregorio-lsp/validation/validator';

const validator = new DocumentValidator();
const errors = validator.validate(document);
```

#### Methods

##### `validate(doc: ParsedDocument): ParseError[]`

Validate a parsed document.

**Parameters**:
- `doc`: Parsed document to validate

**Returns**: Array of validation errors and warnings

**Example**:
```typescript
const validator = new DocumentValidator();
const errors = validator.validate(parsedDoc);

errors.forEach(error => {
  console.log(`${error.severity}: ${error.message}`);
});
```

##### `setRuleEnabled(ruleName: string, enabled: boolean): void`

Enable or disable a specific validation rule.

**Parameters**:
- `ruleName`: Name of the rule
- `enabled`: Whether to enable the rule

**Example**:
```typescript
validator.setRuleEnabled('quilisma-lower-pitch', false);
```

##### `getAvailableRules(): string[]`

Get list of all available validation rules.

**Returns**: Array of rule names

---

### Validation Rules

Individual validation rules exported from `validation/rules.ts`.

#### ValidationRule Interface

```typescript
interface ValidationRule {
  name: string;
  severity: 'error' | 'warning' | 'info';
  validate: (doc: ParsedDocument) => ParseError[];
}
```

#### Available Rules

##### `validateNameHeader`
- **Name**: `name-header`
- **Severity**: warning
- **Description**: Warns if the `name` header is missing

##### `validateFirstSyllableLineBreak`
- **Name**: `first-syllable-line-break`
- **Severity**: error
- **Description**: Errors if first syllable has a line break

##### `validateFirstSyllableClefChange`
- **Name**: `first-syllable-clef-change`
- **Severity**: error
- **Description**: Errors if first syllable has a clef change

##### `validateNabcWithoutHeader`
- **Name**: `nabc-without-header`
- **Severity**: error
- **Description**: Errors if NABC content exists without `nabc-lines` header

##### `validateQuilismaFollowedByLowerPitch`
- **Name**: `quilisma-lower-pitch`
- **Severity**: warning
- **Description**: Warns when quilisma is followed by equal or lower pitch

##### `validateQuilismaPesPrecededByHigherPitch`
- **Name**: `quilisma-pes-higher-pitch`
- **Severity**: warning
- **Description**: Warns when quilisma-pes is preceded by equal or higher pitch

##### `validateVirgaStrataFollowedByHigherPitch`
- **Name**: `virga-strata-higher-pitch`
- **Severity**: warning
- **Description**: Warns when virga strata is followed by equal or higher pitch

##### `validateStaffLines`
- **Name**: `staff-lines`
- **Severity**: error
- **Description**: Errors if staff lines count is invalid (must be 2-5)

---

## Type Definitions

### Core Types

#### `Position`
```typescript
interface Position {
  line: number;
  character: number;
}
```

#### `Range`
```typescript
interface Range {
  start: Position;
  end: Position;
}
```

#### `ParsedDocument`
```typescript
interface ParsedDocument {
  headers: Map<string, string>;
  notation: NotationSection;
  comments: Comment[];
  errors: ParseError[];
}
```

#### `ParseError`
```typescript
interface ParseError {
  message: string;
  range: Range;
  severity: 'error' | 'warning' | 'info';
}
```

#### `NotationSection`
```typescript
interface NotationSection {
  syllables: Syllable[];
  range: Range;
}
```

#### `Syllable`
```typescript
interface Syllable {
  text: string;
  notes: NoteGroup[];
  range: Range;
  clef?: Clef;
  bar?: Bar;
  lineBreak?: LineBreak;
}
```

#### `NoteGroup`
```typescript
interface NoteGroup {
  gabc: string;
  nabc?: string[];
  range: Range;
  notes: Note[];
}
```

#### `Note`
```typescript
interface Note {
  pitch: string;
  shape: NoteShape;
  modifiers: NoteModifier[];
  range: Range;
}
```

#### `NoteShape` Enum
```typescript
enum NoteShape {
  Punctum = 'punctum',
  PunctumInclinatum = 'punctum_inclinatum',
  Virga = 'virga',
  VirgaReversa = 'virga_reversa',
  Oriscus = 'oriscus',
  Quilisma = 'quilisma',
  Stropha = 'stropha',
  Liquescent = 'liquescent',
  Cavum = 'cavum',
  Linea = 'linea',
  Flat = 'flat',
  Sharp = 'sharp',
  Natural = 'natural'
}
```

#### `NoteModifier`
```typescript
interface NoteModifier {
  type: ModifierType;
  value?: string;
}
```

#### `ModifierType` Enum
```typescript
enum ModifierType {
  InitioDebilis = 'initio_debilis',
  PunctumMora = 'punctum_mora',
  HorizontalEpisema = 'horizontal_episema',
  VerticalEpisema = 'vertical_episema',
  Liquescent = 'liquescent',
  Oriscus = 'oriscus',
  Quilisma = 'quilisma',
  Fusion = 'fusion',
  Cavum = 'cavum',
  Strata = 'strata'
}
```

---

## Usage Examples

### Basic Parsing

```typescript
import { GabcParser } from 'gregorio-lsp/parser/gabc-parser';

const gabcText = `
name: Kyrie;
mode: 8;
%%
(c4) Ky(f)ri(g)e(h) *() e(h)le(g)i(f)son.(f.) (::)
`;

const parser = new GabcParser(gabcText);
const doc = parser.parse();

console.log('Name:', doc.headers.get('name'));
console.log('Mode:', doc.headers.get('mode'));
console.log('Syllables:', doc.notation.syllables.length);
```

### Validation

```typescript
import { GabcParser } from 'gregorio-lsp/parser/gabc-parser';
import { DocumentValidator } from 'gregorio-lsp/validation/validator';

const parser = new GabcParser(gabcText);
const doc = parser.parse();

const validator = new DocumentValidator();
const errors = validator.validate(doc);

errors.forEach(error => {
  console.log(`[${error.severity}] ${error.message}`);
  console.log(`  at line ${error.range.start.line}`);
});
```

### Custom Validation Rules

```typescript
import { ValidationRule, ParseError, ParsedDocument } from 'gregorio-lsp';

const customRule: ValidationRule = {
  name: 'my-custom-rule',
  severity: 'warning',
  validate: (doc: ParsedDocument): ParseError[] => {
    const errors: ParseError[] = [];
    
    // Custom validation logic
    if (!doc.headers.has('author')) {
      errors.push({
        message: 'Consider adding an author header',
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
        severity: 'info'
      });
    }
    
    return errors;
  }
};

const validator = new DocumentValidator([customRule]);
```

### Tree-sitter Integration

```typescript
import { treeSitterParser } from 'gregorio-lsp/parser/tree-sitter-integration';

if (treeSitterParser.isTreeSitterAvailable()) {
  const tree = treeSitterParser.parse(gabcText);
  
  if (tree) {
    const headers = treeSitterParser.extractHeaders(tree, gabcText);
    const errors = treeSitterParser.extractErrors(tree);
    
    console.log('Parsed with tree-sitter');
    console.log('Headers:', headers);
    console.log('Errors:', errors);
  }
} else {
  console.log('Tree-sitter not available, using fallback');
}
```

---

## Error Handling

All parsing and validation methods are designed to be non-throwing. Errors are collected and returned as part of the result.

```typescript
try {
  const parser = new GabcParser(text);
  const doc = parser.parse();
  
  // Check for parse errors
  if (doc.errors.length > 0) {
    console.error('Parse errors:', doc.errors);
  }
  
  // Validate
  const validator = new DocumentValidator();
  const errors = validator.validate(doc);
  
  if (errors.length > 0) {
    console.error('Validation errors:', errors);
  }
} catch (error) {
  // Should not happen under normal circumstances
  console.error('Unexpected error:', error);
}
```
