# AGENTS.md — AI Code Generation Guide for gregorio-lsp

This document is the primary reference for AI agents (GitHub Copilot, Claude, etc.)
contributing code to this repository. Read it before making any changes.

> **Language policy**: All content in this repository **must be in English** — without
> exception. This applies to: source code identifiers (variables, functions, constants,
> types), code comments, documentation (`.md` files, docstrings), commit messages,
> test function names and descriptions, error message strings, and any other
> human-readable text added or modified in a contribution.

---

## 1. Project Overview

`gregorio-lsp` is a [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
(LSP) implementation in Rust for the **GABC** and **NABC** notation languages used by
[Gregorio](https://gregorio-project.github.io/), a software suite for typesetting
Gregorian chant in LaTeX.

The server provides real-time diagnostics, completions, hover, and document symbols for
`.gabc` files in any LSP-compatible editor (Helix, Neovim, VS Code, etc.).

It also ships a standalone CLI linter (`gregolint`) for use in CI pipelines and
pre-commit hooks.

**Companion project:**
[`tree-sitter-gregorio`](https://github.com/aiscgre-br/tree-sitter-gregorio) — optional
tree-sitter grammar integrated via the `tree-sitter` Cargo feature flag.

**Language specs** live in `docs/`:

| File | Contents |
|---|---|
| `docs/GABC_SYNTAX_SPECIFICATION.md` | Complete GABC notation reference |
| `docs/NABC_SYNTAX_SPECIFICATION.md` | Complete NABC (adiastematic) notation reference |
| `docs/GREGORIO_COMPILER_ERRORS_AND_WARNINGS.md` | Upstream compiler diagnostics |
| `docs/ERRORS_AND_WARNINGS_SUMMARY.md` | Summary table of errors/warnings |

---

## 2. Repository Structure

```
Cargo.toml              ← Package manifest; optional feature: tree-sitter
Cargo.lock              ← Locked dependency versions (commit this)
LICENSE                 ← MIT

src/
  lib.rs                ← Public crate API; re-exports lint, parser, validation modules
  lint.rs               ← lint_gabc_text() pipeline entry point
  tree_sitter_integration.rs ← Optional tree-sitter wrapper (#[cfg(feature="tree-sitter")])
  parser/
    mod.rs              ← Re-exports GabcParser, nabc::*, types
    types.rs            ← Core AST types (Position, Range, ParseError, Syllable, Note, …)
    gabc.rs             ← Hand-written GABC parser (~1000 lines)
    nabc.rs             ← Hand-written NABC parser (~600 lines)
  validation/
    mod.rs              ← Re-exports rules, semantic, DocumentValidator
    rules.rs            ← 10 structural validation rules (ValidationRule structs)
    semantic.rs         ← SemanticAnalyzer (musical constructions, NABC checks)
    validator.rs        ← DocumentValidator — orchestrates rules + parser errors
  bin/
    server.rs           ← LSP server binary (tower-lsp + tokio, ~500 lines)
    lint.rs             ← gregolint CLI binary (~200 lines)

tests/
  corpus.rs             ← Integration corpus tests
  gabc_parser.rs        ← GABC parser unit tests (15+ cases)
  gabc_advanced.rs      ← Advanced GABC parser tests
  leaning_indicators.rs ← Punctum inclinatum leaning tests
  nabc_parser.rs        ← NABC parser unit tests
  oriscus_leaning.rs    ← Oriscus direction tests
  validation.rs         ← Validation + semantic tests (10+ cases)

docs/                   ← GABC/NABC language specifications
examples/               ← Sample .gabc files (kyrie-xvi, errors-example, nabc-example)
```

---

## 3. Development Workflow

> **NixOS environment**: `cargo`, `rustc`, and `gcc` are not in PATH directly.
> Always use nix shells:

```bash
# Build (standard)
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#gcc -c cargo build

# Build with tree-sitter feature
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#gcc -c \
  cargo build --features tree-sitter

# Run all tests
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#gcc -c cargo test

# Run tests with tree-sitter feature
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#gcc -c \
  cargo test --features tree-sitter

# Run a specific test
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#gcc -c \
  cargo test --test gabc_parser test_function_name

# Code formatting
nix shell nixpkgs#cargo nixpkgs#rustc -c cargo fmt

# Lint
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#gcc -c cargo clippy
```

**The cycle for any code change:**

1. Edit source in `src/`
2. `cargo fmt` — keep formatting consistent
3. `cargo clippy` — fix any warnings before committing
4. Add or update tests in `tests/`
5. `cargo test` — all tests must pass
6. `cargo test --features tree-sitter` — also verify tree-sitter integration
7. Update `CHANGELOG.md` with a concise entry

---

## 4. Architecture

### 4.1 Lint Pipeline

`lint_gabc_text(text, options) -> Vec<ParseError>` in `src/lint.rs` is the single
public entry point for diagnostics:

1. **Parse** — `GabcParser::new(text).parse()` → `ParsedDocument`
2. **Validate** — `DocumentValidator::new().validate(&doc)` → structural errors
3. **Semantic** — `SemanticAnalyzer::new().analyze(&doc)` → musical construction warnings
4. **Tree-sitter** (optional) — `TreeSitterParser::extract_query_diagnostics()` → grammar-level errors
5. **Filter** — apply `LintOptions.min_severity` and `LintOptions.ignore_codes`
6. **Deduplicate** — remove exact-position duplicates

### 4.2 Feature Flag: `tree-sitter`

The `tree-sitter` Cargo feature is **opt-in** and **never enabled by default**. All code
gated on it must use `#[cfg(feature = "tree-sitter")]`. The feature adds:

- `src/tree_sitter_integration.rs` — `TreeSitterParser` struct
- Dependency on `tree-sitter = "~0.22"` and `tree-sitter-gregorio = "=1.0.0-alpha.1"`

Do NOT introduce any `tree-sitter` imports outside `tree_sitter_integration.rs`.

### 4.3 Binary vs. Library

| Target | Path | Purpose |
|---|---|---|
| `lib` (`gregorio_lsp`) | `src/lib.rs` | Public API for third-party consumers |
| `bin` (`gregorio-lsp`) | `src/bin/server.rs` | LSP stdio server |
| `bin` (`gregolint`) | `src/bin/lint.rs` | CLI linter |

The library re-exports everything a consumer needs. The binaries are thin wrappers
that call into the library.

---

## 5. Core Types (`src/parser/types.rs`)

All cross-module data structures live here. The most important ones:

| Type | Purpose |
|---|---|
| `Position { line, character }` | 0-based LSP position |
| `Range { start, end }` | Pair of `Position` |
| `Severity` | `Error \| Warning \| Info` |
| `ParseError { message, range, severity, code }` | Single diagnostic |
| `ParsedDocument` | Full parse result: headers, notation, comments, errors |
| `Syllable` | Text + notes + clef/bar/line-break override |
| `NoteGroup` | GABC snippet + optional NABC snippet, parsed notes, custos, attributes |
| `Note` | Single note: pitch, shape, modifiers, fusion |
| `NoteShape` | 13 variants: Punctum, Quilisma, Virga, Oriscus, … |
| `ModifierType` | 12 variants: InitioDebilis, PunctumMora, HorizontalEpisema, … |
| `Clef` | kind (C/F), line (1–4), has_flat |
| `Bar` | kind (Virgula … DivisioFinalis) |
| `GabcAttribute` | `[key:value]` or `[key]` attribute on a note |
| `NabcBasicGlyph` | 31 variants: vi, pu, ta, gr, cl, pe, po, … |
| `NabcGlyphDescriptor` | Full NABC descriptor with modifiers, pitch, subpunctis, fusion |

**Rules for modifying types:**
- `ParseError` fields are serialized to LSP `Diagnostic` in `src/bin/server.rs`.
  Any new field must also be handled there.
- `Position` and `Range` follow LSP (0-based, UTF-16 column) conventions throughout.
  Do NOT change to 1-based or UTF-8 without updating all callers.
- `NoteShape` and `ModifierType` enums are exhaustively matched in parser and
  validation code. Adding a variant requires updating every `match`.

---

## 6. Parser Implementation (`src/parser/`)

### 6.1 GABC Parser (`gabc.rs`)

`GabcParser<'a>` is a byte-oriented cursor with line/character tracking. It is
**not** a combinator library — it uses direct index manipulation and regex for headers.

**Key invariants:**
- All `Range` values stored in AST nodes are byte-based internally but
  `position_map` converts them to `Position` (line+character) for diagnostics.
- Column tracking is **code-point based** (matching original TypeScript behaviour),
  not byte-based.
- The parser never panics. Unrecognised input is skipped and produces a `ParseError`.

**Free parsing functions** (not methods on `GabcParser`) handle self-contained sub-problems:
- `parse_clef_with_position(content, base)` — matches `c1`–`c4`, `cb1`–`cb4`, `f1`–`f4`
- `parse_bar_with_position(content, base)` — matches all bar types
- `parse_attribute(text, start)` — parses `[key:value]` or `[key]` attributes
- `parse_note_group(content, base)` — full GABC parenthesis content

When adding new GABC syntax, prefer extending the appropriate free function over
adding state to `GabcParser`.

### 6.2 NABC Parser (`nabc.rs`)

**Entry points:**
- `parse_nabc_snippet(nabc, start)` — single `NabcGlyphDescriptor`
- `parse_nabc_descriptors(nabc, start)` — multiple from a snippet string
- `parse_nabc_snippets(snippets, start)` — batch (one string per snippet)

The parser handles spacing (`/`, `//`, `` ` ``, ` `` ``), glyph codes (2 chars),
modifiers, pitch (`h` + letter), subpunctis (`su`), prepunctis (`pp`), significant
letters (`ls`, `lt`), and fusion chains (`!`).

**Validation** is done inline during parse — `validate_nabc_descriptor()` checks for
invalid pitch letters and conflicting liquescence markers.

---

## 7. Validation System (`src/validation/`)

### 7.1 Structural Rules (`rules.rs`)

Each rule is a `ValidationRule` struct:

```rust
pub struct ValidationRule {
    pub name: &'static str,      // kebab-case code used in ignoreRules config
    pub severity: Severity,
    pub validate: fn(&ParsedDocument) -> Vec<ParseError>,
}
```

**Current rules (in execution order):**

| Code | Severity | Description |
|---|---|---|
| `name-header` | Warning | Missing or empty `name` header |
| `first-syllable-line-break` | Error | Line break on first syllable |
| `first-syllable-clef-change` | Error | Clef change on first syllable |
| `nabc-without-header` | Error | NABC pipe `\|` without `nabc-lines` header |
| `quilisma-lower-pitch` | Warning | Quilisma followed by equal/lower pitch |
| `quilisma-pes-higher-pitch` | Warning | Quilisma-pes preceded by equal/higher pitch |
| `virga-strata-higher-pitch` | Warning | Virga strata followed by equal/higher pitch |
| `staff-lines` | Error | `staff-lines` value outside 2–5 range |
| `balanced-pitch-descriptors-fused-glyphs` | Warning | NABC fused glyphs with unbalanced pitch count |
| `modifiers-in-fused-glyphs` | Warning | Modifiers only allowed on last glyph in fusion |

**Rules for adding new structural rules:**
- Add the `ValidationRule` constant in `rules.rs`.
- Register it in `DocumentValidator::new()` in `validator.rs`.
- Add a test case in `tests/validation.rs`.
- Document the new code in `CONFIG.md` under `ignoreRules`.

### 7.2 Semantic Analyzer (`semantic.rs`)

`SemanticAnalyzer` performs per-note musical construction checks that require
context beyond a single note (preceding/following notes, syllable boundaries).

**Current semantic checks:**

| Code | Severity | Description |
|---|---|---|
| `missing-name-header` | Warning | (duplicate of structural; kept for legacy) |
| `line-break-on-first-syllable` | Error | (duplicate of structural; kept for legacy) |
| `pipe-without-nabc-lines` | Error | (duplicate of structural; kept for legacy) |
| `pes-quadratum-missing-note` | Warning | `q` modifier needs a subsequent note |
| `quilisma-missing-note` | Warning | Quilisma needs a subsequent note |
| `oriscus-scapus-isolated` | Warning | `O` requires both preceding and subsequent notes |
| `oriscus-scapus-missing-preceding` | Warning | `O` needs a preceding note |
| `oriscus-scapus-missing-subsequent` | Warning | `O` needs a subsequent note |
| `quilisma-equal-or-lower` | Warning | Quilisma followed by lower/equal pitch |
| `quilisma-pes-preceded-by-higher` | Warning | Quilisma-pes preceded by higher/equal pitch |
| `virga-strata-equal-or-higher` | Warning | Virga strata followed by higher/equal pitch |
| `pes-stratus-equal-or-higher` | Warning | Pes stratus ending with higher/equal following note |
| `nabc-conflicting-liquescence` | Warning | Both `>` and `~` on same NABC descriptor |
| `nabc-invalid-pitch` | Warning | Invalid NABC pitch letter |
| `quilisma-missing-connector` | Info | Suggests `@` or `!` before quilisma (3+ notes) |

**Rules for adding new semantic checks:**
- Add a private method to `SemanticAnalyzer` and call it from `analyze()`.
- Use `SemanticError { code, message, range, severity, related_info }`.
- `related_info` (`Vec<RelatedInfo>`) can reference supporting positions.
- Add a test case in `tests/validation.rs`.

### 7.3 `DocumentValidator` (`validator.rs`)

Orchestrates rules and collects parser-level errors:

```rust
let validator = DocumentValidator::new();               // all 10 rules enabled
let validator = DocumentValidator::with_rules(vec![…]); // custom subset
validator.set_rule_enabled("rule-name", false);         // toggle at runtime
validator.available_rules()                             // list all rule codes
```

---

## 8. LSP Server (`src/bin/server.rs`)

### 8.1 Architecture

Built on [tower-lsp](https://github.com/ebkalderon/tower-lsp) with async
[tokio](https://tokio.rs/) runtime.

```
Backend {
    client: Client,
    documents: Mutex<HashMap<Url, String>>,          // in-memory text store
    config: Mutex<LintingConfig>,                    // workspace configuration
    ts_parser: Mutex<Option<TreeSitterParser>>,      // [cfg(feature="tree-sitter")]
    ts_trees: Mutex<HashMap<Url, Tree>>,             // [cfg(feature="tree-sitter")]
}
```

### 8.2 LSP Capabilities

| Capability | Status |
|---|---|
| Text document sync | `INCREMENTAL` |
| Publish diagnostics | Enabled (push model) |
| Completion | Triggered on `(`, `\|`, `<`, `n`, `a`, `b`, `c` |
| Hover | Stub (returns `None`) |
| Document symbols | Lists headers |
| Workspace configuration | `workspace/didChangeConfiguration` |
| Workspace folders | Enabled |

### 8.3 Text Synchronization

`did_change` applies LSP `TextDocumentContentChangeEvent` items sequentially using
`apply_lsp_change(text, range, replacement)`. The helper function:
- Converts LSP `Position` to byte offsets via `byte_offset(text, position)`.
- Applies the replacement, returning the updated full document string.

If the `tree-sitter` feature is enabled, `did_change` additionally calls
`TreeSitterParser::apply_incremental_edit()` to update the syntax tree
incrementally instead of re-parsing from scratch.

### 8.4 Configuration Schema

Received via `workspace/didChangeConfiguration` as JSON:

```json
{
  "linting": {
    "enabled": true,
    "severity": "warning",
    "onSave": false,
    "ignoreRules": ["quilisma-missing-connector"]
  }
}
```

- `severity`: `"error"` | `"warning"` | `"info"` (default `"info"`)
- `ignoreRules`: array of rule codes from §7.1 and §7.2
- `onSave`: if `true`, only validate on `didSave` (not on every `didChange`)

When adding a new rule, add its code to `CONFIG.md`.

---

## 9. CLI Linter (`src/bin/lint.rs`)

### 9.1 Interface

```
gregolint [OPTIONS] [FILE…]

Options:
  -s, --severity <error|warning|info>   Minimum severity (default: info)
  -i, --ignore <code>                   Ignore rule code (repeatable)
  -h, --help
  -V, --version
```

Reads from stdin if no files are given.

### 9.2 Output Format

```
filename:line:column: severity [code] message
```

Columns are 1-based in the CLI output (convert from 0-based `Position.character + 1`).

### 9.3 Exit Codes

| Code | Meaning |
|---|---|
| `0` | No errors (warnings/infos may be present) |
| `1` | At least one error-severity diagnostic |
| `2` | CLI argument parsing error |

Do NOT change exit code semantics — CI scripts depend on them.

---

## 10. Tree-Sitter Integration (`src/tree_sitter_integration.rs`)

### 10.1 `TreeSitterParser` Methods

| Method | Purpose |
|---|---|
| `new() -> Option<Self>` | Initializes parser; returns `None` if language unavailable |
| `parse(text) -> Option<Tree>` | Full parse from scratch |
| `parse_with_old(text, old_tree) -> Option<Tree>` | Incremental parse |
| `apply_incremental_edit(text, range, replacement)` | Computes `InputEdit`, returns new text + edit |
| `extract_errors(tree, text) -> Vec<ParseError>` | Finds `ERROR` nodes in the CST |
| `extract_query_diagnostics(tree, text)` | Runs `diagnostics.scm` queries |
| `find_node_at(tree, text, position) -> Option<Node>` | Hover/go-to-definition support |

### 10.2 Version Contract

`tree-sitter-gregorio` exposes `STABLE_NODE_KIND_CONTRACT_VERSION` in its Rust binding.
At startup, `TreeSitterParser::new()` checks this version. If it does not match the
version the server was compiled against, initialization fails gracefully (returns `None`).

When `tree-sitter-gregorio` bumps its version, update the expected value in
`tree_sitter_integration.rs` and bump the `tree-sitter-gregorio` version pin in
`Cargo.toml`.

### 10.3 Position Conversions

The tree-sitter API uses byte offsets and `Point { row, column }` (0-based,
byte columns). The LSP uses `Position { line, character }` (0-based, UTF-16 columns).
All conversions between these coordinate systems are centralized in
`tree_sitter_integration.rs`. Do NOT duplicate conversion logic elsewhere.

---

## 11. Testing

### 11.1 Test File Layout

Integration tests live in `tests/`. Each file covers one logical domain:

| File | Domain |
|---|---|
| `tests/gabc_parser.rs` | Basic GABC header and notation parsing |
| `tests/gabc_advanced.rs` | Advanced GABC features (attributes, alterations, custos) |
| `tests/nabc_parser.rs` | NABC descriptor parsing |
| `tests/leaning_indicators.rs` | Punctum inclinatum leaning suffixes (`0`/`1`/`2`) |
| `tests/oriscus_leaning.rs` | Oriscus direction (`o0`/`o1`/`O0`/`O1`) |
| `tests/validation.rs` | Structural rules + semantic checks |
| `tests/corpus.rs` | Integration corpus (full documents) |

### 11.2 Writing Tests

Use `pretty_assertions` for readable diffs:

```rust
use pretty_assertions::assert_eq;
```

For parser tests, parse a minimal `.gabc` string and assert specific fields:

```rust
let doc = GabcParser::new("name: Test;\n%%\nA(c4)").parse();
assert_eq!(doc.headers.get("name"), Some(&"Test".to_string()));
```

For validation tests, pass the document through `lint_gabc_text` and inspect
the returned `Vec<ParseError>` by code and severity.

For tree-sitter integration tests, gate them behind `#[cfg(feature = "tree-sitter")]`.

### 11.3 Coverage Expectations

- Every new `ValidationRule` must have at least one positive and one negative test.
- Every new `SemanticAnalyzer` check must have at least one test that triggers it
  and one that confirms the clean case produces no false positives.
- New parser constructs must have at least one test exercising the happy path and
  one for malformed input.

---

## 12. Common Pitfalls

1. **`LintOptions` severity filter**: `LintSeverity::Error` only shows errors;
   `LintSeverity::Info` shows everything. The filter is `diagnostic.severity >= min_severity`.
   Do not reverse the comparison direction.

2. **0-based vs. 1-based positions**: `Position` is 0-based (LSP convention).
   The CLI linter converts to 1-based for human-readable output. Do not mix the two
   inside `src/`.

3. **`ParseError.code` uniqueness**: Each rule and semantic check must have a unique
   kebab-case code. Duplicates cause `ignoreRules` to suppress unintended diagnostics.
   Check `rules.rs` and `semantic.rs` before adding a new code.

4. **Mutex deadlocks in the LSP server**: `Backend` fields are wrapped in `tokio::sync::Mutex`.
   Never hold two locks simultaneously. Acquire, use, and drop each lock separately.

5. **`apply_lsp_change` byte offset arithmetic**: LSP sends `Position` in UTF-16 columns.
   `byte_offset()` must iterate the line as UTF-8 while counting UTF-16 code units.
   Incrementally patching this function is fragile — add a test for non-ASCII input
   whenever you touch it.

6. **`parse_with_old` requires a valid `InputEdit`**: Passing a stale or incorrect
   `InputEdit` to tree-sitter will silently produce a corrupt tree. Always derive the
   edit from `apply_incremental_edit()` rather than constructing it manually.

7. **Feature-gated code in `lib.rs`**: `tree_sitter_integration` is re-exported from
   `lib.rs` only when the feature is enabled. Downstream consumers that do not enable
   the feature will get a compile error if they import it. Never move tree-sitter types
   into the unconditional public API.

8. **`NoteShape` and `ModifierType` are non-exhaustive in practice**: All `match`
   expressions on these enums must cover every variant. Clippy's `non_exhaustive_patterns`
   lint will catch missing arms, but only if `#[warn(clippy::wildcard_enum_match_arm)]`
   is enabled. Do not add `_ => unreachable!()` catch-alls to hide new variants.

9. **Parser error recovery**: `GabcParser` skips unrecognised bytes and emits a
   `ParseError` rather than panicking. New parser code must follow the same contract —
   never use `unwrap()` or `expect()` on user input.

10. **`HeaderMap` case-insensitive keys**: Headers are stored and looked up case-
    insensitively. Do not bypass `HeaderMap::get()` with a raw `HashMap` lookup, as
    that would miss alternate casing.

11. **`SemanticAnalyzer` cross-syllable context**: Some checks (oriscus scapus,
    quilisma connector) require inspecting the previous syllable's last note. The
    analyser tracks `prev_syllable: Option<&Syllable>`. Never assume the previous
    note is in the same syllable.

12. **Exit code 1 vs. 2 in `gregolint`**: Exit code `1` means linting found errors;
    exit code `2` means the CLI itself could not run (bad arguments, unreadable file).
    Do not confuse the two.

---

## 13. Companion Project: tree-sitter-gregorio

When `tree-sitter-gregorio` changes its grammar (node kinds, field names, or
`STABLE_NODE_KIND_CONTRACT_VERSION`), also update `gregorio-lsp`:

- `src/tree_sitter_integration.rs` — update node kind strings and field name strings
  used in `extract_errors()` and `extract_query_diagnostics()`.
- `Cargo.toml` — bump the `version = "=X.Y.Z"` pin on `tree-sitter-gregorio`.
- `Cargo.lock` — regenerate (`cargo update -p tree-sitter-gregorio`).

After updating, run the full test suite with the feature enabled:

```bash
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#gcc -c \
  cargo test --features tree-sitter
```

---

## 14. Commit Style

- Follow **Conventional Commits**: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- Scope (optional): `feat(server):`, `fix(parser):`, `test(validation):`, `docs(agents):`
- GPG-sign commits (preferred):
  ```bash
  git -c gpg.program=gpg -c commit.gpgsign=true commit \
    --gpg-sign=BAC0B1B569777A733E37447FB10712C404063D38 -m "feat(server): ..."
  ```
- Keep **all text in English**: commit messages, comments, documentation, identifiers,
  test names, and error strings.
- Reference upstream Gregorio issues when implementing syntax from a specific release:
  `feat(parser): parse lyric tie ~ in syllable text (gregorio#1684)`

---

## 15. Checklist for Adding a New Validation Rule

- [ ] Read the relevant section of `docs/GABC_SYNTAX_SPECIFICATION.md` or
      `docs/NABC_SYNTAX_SPECIFICATION.md`
- [ ] Decide: structural rule (`rules.rs`) or semantic check (`semantic.rs`)
- [ ] Choose a unique kebab-case code — verify it does not clash with existing codes
- [ ] Implement the rule/check with appropriate `Severity` and `ParseError.code`
- [ ] Register the rule in `DocumentValidator::new()` (for structural rules only)
- [ ] Add a positive test (triggers the diagnostic) in `tests/validation.rs`
- [ ] Add a negative test (clean input, no false positive) in `tests/validation.rs`
- [ ] Document the new code in `CONFIG.md` under `ignoreRules`
- [ ] Run `cargo test` and `cargo test --features tree-sitter`
- [ ] Update `CHANGELOG.md` with a concise entry
- [ ] GPG-sign the commit

## 16. Checklist for Adding a New Parser Feature

- [ ] Read the relevant section of the language specification in `docs/`
- [ ] Add or extend the appropriate type in `src/parser/types.rs`
- [ ] Implement parsing in `src/parser/gabc.rs` or `src/parser/nabc.rs`
- [ ] Ensure malformed input produces a `ParseError`, never a panic
- [ ] Add tests in `tests/gabc_parser.rs`, `tests/gabc_advanced.rs`, or `tests/nabc_parser.rs`
- [ ] Update any `match` expressions on affected enums in validation and server code
- [ ] Run `cargo test` and `cargo clippy`
- [ ] Update `CHANGELOG.md` with a concise entry
- [ ] GPG-sign the commit
