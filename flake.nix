{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { flake-utils, nixpkgs, naersk, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        toolchain = with fenix.packages.${system};  combine [
          minimal.cargo
          minimal.rustc
          latest.clippy
          latest.rust-src
          latest.rustfmt
          targets.wasm32-unknown-unknown.latest.rust-std
          targets.x86_64-unknown-linux-gnu.latest.rust-std
        ];

        fetchy = (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        }).buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [ ] ++ min-pkgs;
        };

        min-pkgs = with pkgs; [
          pkg-config
          openssl
          gcc
          nodejs_22
          geckodriver
          dioxus-cli
        ];
      in
      {
        defaultPackage = fetchy;

        packages = {
          fetchy = fetchy;
        };

        devShell = (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        }).buildPackage
          {
            src = ./.;
            nativeBuildInputs = with pkgs; [ ] ++ min-pkgs;

            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
                pkgs.lib.makeLibraryPath min-pkgs
              }"'';
          };
      });
}
