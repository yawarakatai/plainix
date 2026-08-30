{
  description = "Plainix - start a reproducible Nix development environment without learning flakes first";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forSystems = nixpkgs.lib.genAttrs systems;
      packageFor = system: self.packages.${system}.plainix;
      plainixLib = import ./lib;
      development = plainixLib.project { inherit nixpkgs; } ./plainix.nix;
    in
    {
      lib = plainixLib;

      packages = forSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          source = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./cli/Cargo.toml
              ./cli/src
            ];
          };
          plainix = pkgs.rustPlatform.buildRustPackage {
            pname = "plainix";
            version = "0.1.0";
            src = source;
            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "Scaffold reproducible Nix development environments";
              homepage = "https://github.com/yawarakatai/plainix";
              license = pkgs.lib.licenses.mit;
              mainProgram = "plainix";
            };
          };
        in
        {
          inherit plainix;
          default = plainix;
        }
      );

      apps = forSystems (system: {
        plainix = {
          type = "app";
          program = "${packageFor system}/bin/plainix";
          meta.description = "Initialize a Plainix development environment";
        };
        default = self.apps.${system}.plainix;
      });

      devShells = development.devShells;

      checks = forSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          minimal = self.lib.project { inherit nixpkgs; } ./tests/minimal/plainix.nix;
          presets = self.lib.project { inherit nixpkgs; } ./tests/presets/plainix.nix;
          invalidProjects =
            map
              (
                projectFile:
                builtins.tryEval (builtins.deepSeq (self.lib.project { inherit nixpkgs; } projectFile) true)
              )
              [
                ./tests/invalid/plainix.nix
                ./tests/invalid-package/plainix.nix
                ./tests/invalid-system/plainix.nix
              ];
          examples = map (projectFile: self.lib.project { inherit nixpkgs; } projectFile) [
            ./examples/minimal/plainix.nix
            ./examples/rust/plainix.nix
            ./examples/sqlite-c/plainix.nix
          ];
        in
        {
          cli = packageFor system;

          library =
            assert builtins.isString minimal.devShells.${system}.default.drvPath;
            assert minimal.devShells.${system}.default.RUST_BACKTRACE == "1";
            assert builtins.isString presets.devShells.${system}.default.drvPath;
            assert builtins.all (result: !result.success) invalidProjects;
            assert builtins.all (
              example: builtins.isString example.devShells.${system}.default.drvPath
            ) examples;
            pkgs.runCommand "plainix-library-tests" { } ''
              touch $out
            '';
        }
      );
    };
}
