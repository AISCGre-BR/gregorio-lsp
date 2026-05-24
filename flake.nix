{
  description = "gregorio-lsp — dev shell with locally compiled binaries in PATH";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.gcc
          ];

          env.RUST_BACKTRACE = "1";

          shellHook = ''
            # Put the locally compiled release binaries at the front of PATH
            # so they shadow any system-installed version.  Zed's extension
            # uses worktree.which("gregorio-lsp") which respects this PATH,
            # giving the local build priority when working inside this project.
            export PATH="$PWD/target/release:$PATH"

            echo ""
            echo "  gregorio-lsp dev shell"
            echo "  ────────────────────────────────────────────────────────"
            echo "  cargo build --release   → rebuild and reload in Zed"
            echo "  cargo test              → run test suite"
            echo "  cargo clippy            → lint"
            echo "  cargo fmt --check       → check formatting"
            echo ""
            echo "  Active binary: $PWD/target/release/gregorio-lsp"
            echo "  ────────────────────────────────────────────────────────"
            echo ""
          '';
        };
      }
    );
}
