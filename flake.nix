{
  description = "vtopssh terminal system monitor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;
        runtimePath = lib.makeBinPath [
          pkgs.openssh
          pkgs.iputils
          pkgs.util-linux
        ];
        package = pkgs.rustPlatform.buildRustPackage {
          pname = "vtopssh";
          version = "0.1.3";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.makeWrapper ];

          postInstall = ''
            wrapProgram $out/bin/vtopssh \
              --prefix PATH : ${runtimePath}
          '';

          meta = with lib; {
            description = "Terminal UI system monitor for Linux with remote host support over SSH";
            homepage = "https://github.com/vrubelroman/vtopssh";
            license = licenses.mit;
            mainProgram = "vtopssh";
            platforms = platforms.linux;
          };
        };
      in {
        packages.default = package;

        apps.default = {
          type = "app";
          program = "${package}/bin/vtopssh";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            pkg-config
            rust-analyzer
            rustc
            rustfmt
          ];
        };
      });
}
