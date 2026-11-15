# Security Policy

## Supported versions

Only the latest release of `gregorio-lsp` (the `main` branch) receives
security fixes.

## Reporting a vulnerability

Please **do not** open a public issue for security problems.

Report vulnerabilities privately through GitHub:
[Security → Report a vulnerability](https://github.com/AISCGre-BR/gregorio-lsp/security/advisories/new).

Include the affected version, a description of the issue and, if possible, a
minimal `.gabc` file or steps that reproduce it. You should receive a response
within 7 days.

## Automated checks

- **CodeQL** (`.github/workflows/codeql.yml`) — static analysis of the Rust
  sources and of the GitHub Actions workflows, on every push, pull request and
  weekly.
- **Dependency Review** (`.github/workflows/dependency-review.yml`) — blocks
  pull requests that add dependencies with known vulnerabilities or
  disallowed licenses.
- **cargo-deny** (`.github/workflows/security-audit.yml`, `deny.toml`) —
  checks dependencies against the RustSec advisory database daily, and
  enforces the license, source and duplicate-crate policy.
- **Dependabot** (`.github/dependabot.yml`) — weekly update PRs for Cargo
  dependencies and GitHub Actions.
