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

        # Builds the whole workspace (gregorio-lsp, grelint, grefmt) in one
        # derivation. gregorio-wasm is dropped from the workspace since it
        # requires wasm-bindgen and only targets wasm32; keeping it would
        # pull that dependency into native host builds for no benefit.
        gregorio-lsp = rustPlatform.buildRustPackage {
          pname = "gregorio-lsp";
          version = "0.11.0";
          src = ./.;

          postPatch = ''
            sed -i '/gregorio-wasm/d' Cargo.toml
          '';

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              # tree-sitter-gregorio is an optional git dep (feature = "tree-sitter").
              # Nix requires its hash even when the feature is not built.
              "tree-sitter-gregorio-0.5.2" = "sha256-olYGpGIKSUp5IV+8jaNwuRDMB6pL6ITeCywfqBuVAp0=";
            };
          };

          meta = {
            description = "Language Server, linter and formatter for Gregorio GABC/NABC notation";
            homepage = "https://github.com/AISCGre-BR/gregorio-lsp";
            license = pkgs.lib.licenses.mit;
            mainProgram = "gregorio-lsp";
          };
        };
      in
      {
        packages = {
          inherit gregorio-lsp;

          # Same build as gregorio-lsp; these just point mainProgram at the
          # other workspace binaries so each tool is invokable on its own.
          grelint = gregorio-lsp.overrideAttrs (old: {
            meta = old.meta // {
              description = "grelint — Gregorio GABC/NABC linter";
              mainProgram = "grelint";
            };
          });

          grefmt = gregorio-lsp.overrideAttrs (old: {
            meta = old.meta // {
              description = "grefmt — Gregorio GABC/NABC formatter";
              mainProgram = "grefmt";
            };
          });

          default = gregorio-lsp;
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
