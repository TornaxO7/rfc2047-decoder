{
  description = "Rust library for decoding RFC 2047 MIME Message Headers.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems.url = "github:nix-systems/default";
  };

  outputs = { nixpkgs, rust-overlay, systems, ... }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      devShells = eachSystem (system:
        let
          pkgs = import nixpkgs {
            inherit system;

            overlays = [ rust-overlay.overlays.default ];
          };

          rust-toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [ cargo-release ] ++ [ rust-toolchain ];
          };
        });
    };
}
