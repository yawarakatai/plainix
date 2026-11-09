{
  description = "Plainix - share a reproducible project environment without learning flakes first";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

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
      environment = plainixLib.project { inherit nixpkgs; } ./plainix.nix;
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
              ./tests/presets.json
            ];
          };
          plainix = pkgs.rustPlatform.buildRustPackage {
            pname = "plainix";
            version = "0.4.0";
            src = source;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [
              pkgs.gitMinimal
              pkgs.makeWrapper
            ];
            postInstall = ''
              wrapProgram $out/bin/plainix \
                --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.gitMinimal ]}
            '';

            meta = {
              description = "Scaffold reproducible project environments with Nix";
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
          meta.description = "Initialize a Plainix project environment";
        };
        default = self.apps.${system}.plainix;
      });

      devShells = environment.devShells;

      checks = forSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          minimal = self.lib.project { inherit nixpkgs; } ./tests/minimal/plainix.nix;
          allowUnfree = self.lib.project { inherit nixpkgs; } ./tests/allow-unfree/plainix.nix;
          systemSpecific = self.lib.project { inherit nixpkgs; } ./tests/system-specific/plainix.nix;
          # Evaluates every preset for every system it claims, so a host only
          # needs to run this once to catch breakage on the other platforms.
          unavailablePresetPackages = pkgs.lib.concatMap (
            preset:
            pkgs.lib.concatMap (
              presetSystem:
              map (package: "${preset.id}: ${package} on ${presetSystem}") (
                builtins.filter (
                  package:
                  !(builtins.tryEval
                    (pkgs.lib.attrByPath (pkgs.lib.splitString "." package) (throw "missing")
                      nixpkgs.legacyPackages.${presetSystem}
                    ).drvPath
                  ).success
                ) preset.packages
              )
            ) preset.systems
          ) (builtins.fromJSON (builtins.readFile ./tests/presets.json));
          invalidProjects =
            map
              (
                projectFile:
                builtins.tryEval
                  (self.lib.project { inherit nixpkgs; } projectFile).devShells.${system}.default.drvPath
              )
              [
                ./tests/invalid/plainix.nix
                ./tests/invalid-package/plainix.nix
                ./tests/invalid-package-set/plainix.nix
                ./tests/invalid-package-value/plainix.nix
                ./tests/invalid-unfree/plainix.nix
                ./tests/invalid-allow-unfree/plainix.nix
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
            assert builtins.isString allowUnfree.devShells.${system}.default.drvPath;
            assert systemSpecific.devShells.${system}.default.PLAINIX_SYSTEM == system;
            assert builtins.attrNames systemSpecific.devShells == builtins.sort builtins.lessThan systems;
            assert pkgs.lib.assertMsg (unavailablePresetPackages == [ ])
              "Plainix: presets offer unavailable packages: ${pkgs.lib.concatStringsSep ", " unavailablePresetPackages}";
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
