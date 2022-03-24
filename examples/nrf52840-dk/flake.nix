{
  description = "Rust dev shell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/master";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustpkg = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in with pkgs; {
        devShell = mkShell {
          buildInputs =
            [ cargo-flash flip-link probe-rs-cli probe-run rustpkg ];
        };
      });
}
