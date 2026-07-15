{
  description = "gregorio-lsp — Language Server and tools for Gregorio GABC/NABC notation";

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

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in
      {
        packages.default = rustPlatform.buildRustPackage {
          pname = "gregorio-lsp";
          version = "0.11.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              # tree-sitter-gregorio is an optional git dep (feature = "tree-sitter").
              # Nix requires its hash even when the feature is not built.
              "tree-sitter-gregorio-0.5.2" = "sha256-olYGpGIKSUp5IV+8jaNwuRDMB6pL6ITeCywfqBuVAp0=";
            };
          };
        };

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
