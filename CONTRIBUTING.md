# Contributing

Contributions are welcome. Before opening a PR:

1. Make sure `cargo build` and `cargo test` pass locally.
2. Run `cargo fmt` and `cargo clippy --all-targets` (warnings must be
   addressed or justified).
3. For changes to the parser or validation rules, add tests in `tests/`
   covering the new behavior.
4. For changes that affect the LSP protocol, describe the impact on
   typical editors (Helix, Neovim, VS Code) in the PR description.

## Commit style

We use short imperative messages; where helpful, use
[Conventional Commits](https://www.conventionalcommits.org/) prefixes
(`feat:`, `fix:`, `refactor:`, `docs:`, `test:`).

## Signing

GPG-signed commits are preferred, especially for changes to validation
or server code.

## Structure

See [README.md](README.md) for an overview of the module structure.
