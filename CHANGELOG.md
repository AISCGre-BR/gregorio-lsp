# Changelog

All notable changes to the Gregorio LSP will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-12-09

### Added

#### Core Features
- Complete Language Server Protocol implementation for GABC files
- Dual parser architecture with tree-sitter-gregorio integration and TypeScript fallback
- Automatic parser selection based on availability
- Real-time diagnostics with error and warning detection
- Hover provider for syntax elements
- Completion provider for clefs, bars, and headers
- Document symbols provider

#### Parser
- Full GABC syntax parser implemented in TypeScript
- Support for all GABC notation elements:
  - Headers (name, mode, annotation, etc.)
  - Clefs (C-clefs, F-clefs, with/without flat)
  - Notes (all shapes: punctum, virga, oriscus, quilisma, etc.)
  - Note modifiers (episema, punctum mora, liquescence, etc.)
  - Bars (virgula, divisio minima/minor/maior/finalis)
  - Style tags (bold, italic, color, etc.)
  - Comments
- NABC (adiastematic notation) support
- Tree-sitter-gregorio integration for enhanced performance

#### Validation
- Complete implementation of Gregorio compiler error detection:
  - Missing required headers
  - Syntax errors (style tags, forced center, etc.)
  - Line breaks and clef changes on first syllable
  - NABC usage without proper headers
  - Invalid staff line counts
  
- Musical validation rules:
  - Quilisma followed by equal/lower pitch (warning)
  - Quilisma-pes preceded by equal/higher pitch (warning)
  - Virga strata followed by equal/higher pitch (warning)
  - Missing connectors in quilismatic sequences (info)

#### Documentation
- Comprehensive README with features, installation, and usage
- API documentation with all public interfaces
- Development guide with contribution guidelines
- Example files demonstrating various features
- Integration guide for tree-sitter-gregorio

#### Testing
- Unit tests for parser functionality
- Unit tests for all validation rules
- Test coverage configuration
- Example files for manual testing

#### Build & Development
- TypeScript configuration with strict mode
- ESLint configuration for code quality
- Jest configuration for testing
- NPM scripts for build, watch, test, and lint
- Git ignore configuration

### Technical Details

#### Dependencies
- `vscode-languageserver`: ^9.0.1
- `vscode-languageserver-textdocument`: ^1.0.11
- `tree-sitter`: ^0.21.0
- `tree-sitter-gregorio`: file:../tree-sitter-gregorio (optional)

#### Dev Dependencies
- TypeScript 5.3.2
- Jest 29.7.0
- ESLint 8.54.0

## [Unreleased]

### Planned Features
- Definition provider (go to header definitions)
- References provider (find all uses of macros)
- Code actions (quick fixes for common errors)
- Formatting provider (auto-format GABC files)
- Rename provider (rename throughout document)
- Signature help (parameter hints for complex syntax)
- Workspace symbols (search across all GABC files)
- Configuration options (enable/disable specific rules)
- Performance optimizations (incremental parsing)
- Enhanced NABC validation
- More comprehensive test suite
- VS Code extension wrapper

### Under Consideration
- Syntax highlighting improvements
- Snippet support
- Code folding
- Semantic tokens provider
- Call hierarchy provider
- Document links (hyperlinks to referenced files)
- Color provider (preview colors in style tags)

---

## Version History

- **0.1.0** (2024-12-09) - Initial release with core LSP features

[0.1.0]: https://github.com/yourusername/gregorio-lsp/releases/tag/v0.1.0
[Unreleased]: https://github.com/yourusername/gregorio-lsp/compare/v0.1.0...HEAD
