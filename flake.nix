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
          # Everything here comes from the binary cache, so `nix run` compiles
          # nothing on the user's machine.
          plainix =
            pkgs.runCommandLocal "plainix"
              {
                meta = {
                  description = "Scaffold reproducible project environments with Nix";
                  homepage = "https://github.com/yawarakatai/plainix";
                  license = pkgs.lib.licenses.mit;
                  mainProgram = "plainix";
                };
              }
              ''
                install -Dm444 ${./cli/plainix.py} $out/share/plainix/plainix.py
                install -Dm444 ${./cli/presets.json} $out/share/plainix/presets.json
                mkdir -p $out/bin
                cat > $out/bin/plainix <<EOF
                #!${pkgs.runtimeShell}
                export PATH="${pkgs.fzf}/bin:\$PATH"
                exec ${pkgs.python3Minimal}/bin/python3 -I $out/share/plainix/plainix.py "\$@"
                EOF
                chmod +x $out/bin/plainix
              '';
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
          ) (builtins.fromJSON (builtins.readFile ./cli/presets.json));
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

          unit =
            pkgs.runCommandLocal "plainix-unit-tests"
              {
                nativeBuildInputs = [
                  pkgs.gitMinimal
                  pkgs.python3Minimal
                ];
              }
              ''
                cp ${./cli}/* .
                export HOME=$PWD
                python3 test_plainix.py
                touch $out
              '';

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
