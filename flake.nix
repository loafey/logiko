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
        ];

        fetchy = (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        }).buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [ ] ++ min-pkgs;
        };

        dockerImage = pkgs.dockerTools.buildImage
          {
            name = "hello";
            tag = "latest";
            created = "now";
            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              paths = [ pkgs.hello ];
              pathsToLink = [ "/bin" ];
            };

            architecture = "arm64";

            config.Cmd = [ "/bin/hello" ];
          };

        min-pkgs = with pkgs; [
          pkg-config
          openssl
          gcc
          nodejs_22
          geckodriver
        ];
      in
      {
        defaultPackage = fetchy;

        packages = {
          fetchy = fetchy;
          docker = dockerImage;
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
