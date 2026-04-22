{
  description = "vbotssh terminal system monitor";

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
          pname = "vbotssh";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.makeWrapper ];

          postInstall = ''
            wrapProgram $out/bin/vbotssh \
              --prefix PATH : ${runtimePath}
          '';

          meta = with lib; {
            description = "Terminal UI system monitor for Linux with remote host support over SSH";
            homepage = "https://github.com/vrubelroman/vbotssh";
            license = licenses.mit;
            mainProgram = "vbotssh";
            platforms = platforms.linux;
          };
        };
      in {
        packages.default = package;

        apps.default = {
          type = "app";
          program = "${package}/bin/vbotssh";
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
