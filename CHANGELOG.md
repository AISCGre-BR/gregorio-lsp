# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.1] - 2026-05-16

### Added
- New validation rule `punctuation-after-note-group` (Warning, auto-fixable):
  detects punctuation typed after a syllable's note-group parentheses
  (for example `foo(), bar(); baz():`), which makes GregorioTeX hyphenate the
  rendered text incorrectly. The auto-fix moves the punctuation before the
  previous `(...)` group (`foo,() bar;() baz:()`).

### Changed
- Merged redundant `oriscus-equal-or-higher` (semantic) into the structural
  `oriscus-higher-pitch` rule: the two checks were logically identical; only the
  structural rule is retained. The diagnostic code remains `oriscus-higher-pitch`.
- Corrected the warning message for `oriscus-higher-pitch`: the issue is not a
  "rendering problem" but a violation of the Gregorian semiological rule — the
  oriscus must always lead to a lower note. The new message reads:
  *"Oriscus on '\{pitch\}' followed by a note of equal or higher pitch '\{next\}':
  violates Gregorian semiological rule (oriscus must always lead to a lower note)"*.

## [0.7.0] - 2026-05-16

### Removed
- Validation rule `multi-word-syllable` (Warning): flagged syllable text containing
  multiple space-separated words sharing a single note group (e.g. `foo bar baz()`)
  and offered an auto-fix. The Gregorio maintainers confirmed that the GABC
  specification is silent on this pattern and that it works as expected in practice,
  so the diagnostic was incorrect. The rule and its auto-fix have been removed
  entirely.

### Changed
- Validation rule `virga-strata-higher-pitch` renamed and expanded to
  `oriscus-higher-pitch` (structural, `rules.rs`). The rule now covers **any oriscus
  that is the last (or only) note in its note group** — i.e. both isolated oriscus and
  virga strata — not just the virga-strata-specific pattern. An oriscus that has a
  following note in the same group (salicus, pes-quassus) is excluded, as the higher
  note is intentional in those neumes. The check is now a cross-group boundary scan
  (the next note may be in the next syllable's group), correcting the previous
  implementation which was dead code (`ModifierType::Strata` was never set by the
  parser). **Replaces the `virga-strata-higher-pitch` code** — update any `ignoreRules`
  config entries accordingly.
- Semantic check `virga-strata-equal-or-higher` (previously dead code) replaced by
  `oriscus-equal-or-higher` (`semantic.rs`) with the same expanded scope and
  cross-group-boundary logic.

## [0.6.0] - 2026-05-14

### Changed
- CLI binary renamed from `gregolint` to `grelint`. All invocations, help text,
  JSON output fields (`"tool"`, `"source"`), and documentation updated accordingly.
  **Breaking change**: existing scripts or editor integrations that invoke `gregolint`
  must be updated to call `grelint` instead.

## [0.5.0] - 2026-05-14

### Added
- New validation rule `line-break-at-end-of-score` (Warning): detects forced
  line-break markers (`z`, `Z`, `z+`, `z-`, `Z+`, `Z-`) at the end of a score,
  which GregorioTeX silently ignores while emitting the warning
  "Package GregorioTeX Warning: Ignoring forced line break ('Z' or 'z') at end of
  score." The rule is auto-fixable: standalone `(z)` groups are removed entirely;
  when the marker is mixed with pitch notes (e.g. `(fgh z)`) only the marker is
  stripped, preserving the notes.
- New validation rule `duplicate-headers` (Warning): detects header keys defined
  more than once in the GABC preamble, mirroring GregorioTeX's warning
  "several %s definitions found". The `annotation` header is excluded from the
  check up to 2 entries (GregorioTeX allows exactly 2 annotations); 3 or more
  `annotation` headers trigger the warning. The `commentary` header is fully
  exempt: it is an `OTHER_HEADER` in GregorioTeX (not a first-class keyword),
  all entries are silently appended to the score's header linked list with no
  duplicate check and no warning at any count.
- New validation rule `duplicate-syllable-center` (Warning): detects a syllable
  text that contains two or more `{…}` forced-center markers; only the first is
  used by GregorioTeX (W-C05).
- New validation rule `center-after-protrusion` (Warning): detects a forced-center
  `{…}` marker that appears after a `<pr>` protrusion tag in syllable text; the
  center is silently ignored by GregorioTeX (W-C06).
- New validation rule `unmatched-center-close` (Warning, auto-fixable): detects a
  `}` appearing in syllable text without a preceding `{`; auto-fix removes the
  stray closing brace (W-C07).
- New validation rule `duplicate-protrusion` (Warning, auto-fixable): detects
  syllable text with more than one `<pr>` protrusion tag; auto-fix keeps only the
  first tag (W-C08).
- New validation rule `unclosed-center-before-protrusion` (Warning, auto-fixable):
  detects an open `{` forced-center that is never closed before a `<pr>` protrusion
  tag; auto-fix inserts the missing `}` immediately before the protrusion (W-C09).
- `Syllable.text_range`: new `Range` field on the `Syllable` AST node that covers
  only the text portion of a syllable (before the opening `(`), enabling precise
  diagnostic ranges for text-markup rules.
- `HeaderMap.duplicate_keys`: new `Vec<String>` field that records each key
  overwritten during header parsing, enabling the `duplicate-headers` rule.

## [0.4.0] - 2026-05-14

### Added
- GABC pitch transposition: `shift_notes` shifts all pitch letters
  (`a`–`n`, `p`) in `(…)` note groups one step up or down through the
  15-pitch cycle `a b c d e f g h i j k l m n p`.  Uppercase
  (PunctumInclinatum) letters follow an independent cycle.  Multi-NABC
  segments (`nabc-lines: N`) are detected and left unchanged.
  Selection-aware: when a byte range is supplied only pitches within that
  range are shifted.  Exposed as LSP code actions ("Shift all notes up/down",
  "Shift selected notes up/down") and workspace command
  `gregorio/shiftNotesUp` / `gregorio/shiftNotesDown`.
- GABC empty-group fill: `fill_empty_groups` replaces each empty `()` group
  with the last GABC pitch letter seen in a preceding non-empty, non-clef
  group — e.g. `(fgh) () ()` → `(fgh) (h) (h)`.  Clef groups are skipped
  and do not update the seed pitch.  Multi-NABC aware (uses only the GABC
  segments to extract the seed).  Selection-aware.  Exposed as LSP code
  action "Fill empty note groups / Fill empty note groups in selection" and
  workspace command `gregorio/fillEmptyGroups`.
- `note_ops` module (`src/note_ops.rs`) consolidating all note-level
  manipulation helpers: `shift_notes`, `fill_empty_groups`, `shift_pitch`,
  `is_gabc_pitch`, `parse_nabc_lines`.

### Changed
- Internal module `transpose` renamed to `note_ops` to reflect that it now
  hosts multiple unrelated note-manipulation operations.

## [0.3.0] - 2026-05-04

### Added
- `AGENTS.md`: comprehensive AI code generation guide covering architecture, parsers,
  validation rules, LSP server, CLI linter, tree-sitter integration, testing, and
  commit style.
- CLI `gregolint`: `--format json` (`-f json`) output mode — structured JSON with
  `tool`, `diagnostics[]`, `skipped[]`, and `summary` fields; ranges are 0-based
  (LSP convention), suitable for CI pipelines and editor integrations.
- Fallback GABC parser now correctly handles Gregorio 6.2.0 changes:
  - Empty GABC snippets before NABC content (`(|vi)`, `(||ta)`, `(g||ta)`) — pipe
    always starts the NABC side of the snippet list.
  - Lyric tie `~` in syllable text passes through as literal text (not a parse error).
- Four new regression tests covering the above parser changes.

### Changed
- `tree-sitter-gregorio` dependency changed from local path to published git tag
  `v0.5.2` (`https://github.com/aiscgre-br/tree-sitter-gregorio`).
- Copyright and authorship updated to **AISCGre Brasil** across `LICENSE`,
  `Cargo.toml`, and `README.md`.
- CLI `gregolint`: unreadable file now exits with code `2` (CLI execution failure)
  instead of `1` (lint errors found), matching documented exit code semantics.

## [0.2.0]

### Changed
- Full rewrite from TypeScript to Rust (edition 2021).
- LSP server reimplemented on [tower-lsp](https://github.com/ebkalderon/tower-lsp).
- CLI `gregolint` ported to Rust with the same interface (`-s`, `-i`, `-h`, `-V`).
- GABC and NABC parsers rewritten preserving behaviour and messages.
- Semantic analyser and ten validation rules ported.
- Integration with [tree-sitter-gregorio](https://github.com/aiscgre-br/tree-sitter-gregorio)
  made optional via `tree-sitter` feature flag.
- Test suite with 58 integration cases covering the GABC parser, NABC parser,
  validation, leaning indicators, oriscus, and example corpus.
- Removed Node toolchain artefacts (`package.json`, `tsconfig.json`,
  `jest.config.js`, `.eslintrc.json`, `scripts/`, `node_modules/`).

## [0.1.x]

- Initial TypeScript implementation. History preserved in Git commits.

---

[0.6.0]: https://github.com/aiscgre-br/gregorio-lsp/compare/v0.5.0...v0.6.0
[0.4.0]: https://github.com/aiscgre-br/gregorio-lsp/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/aiscgre-br/gregorio-lsp/releases/tag/v0.3.0
