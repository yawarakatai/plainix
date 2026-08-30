{
  project =
    inputs: projectFile:
    let
      importedSpec = import projectFile;

      nixpkgs =
        if builtins.hasAttr "nixpkgs" inputs then
          inputs.nixpkgs
        else
          throw "Plainix: the flake must provide an input named `nixpkgs`";

      lib = nixpkgs.lib;
      typeOf = builtins.typeOf;
      join = lib.concatStringsSep ", ";

      validateAttrs =
        label: allowed: value:
        if !builtins.isAttrs value then
          throw "Plainix: `${label}` must be an attribute set, but got ${typeOf value}"
        else
          let
            unknown = builtins.filter (name: !(builtins.elem name allowed)) (builtins.attrNames value);
          in
          if unknown == [ ] then
            value
          else
            throw "Plainix: `${label}` contains unknown field(s): ${join unknown}";

      spec = validateAttrs "plainix.nix" [
        "develop"
        "systems"
      ] importedSpec;

      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      rawSystems = spec.systems or [ "x86_64-linux" ];
      systems =
        if !builtins.isList rawSystems then
          throw "Plainix: `systems` must be a list of strings, but got ${typeOf rawSystems}"
        else if rawSystems == [ ] then
          throw "Plainix: `systems` must contain at least one system"
        else if !(builtins.all builtins.isString rawSystems) then
          throw "Plainix: every value in `systems` must be a string"
        else if lib.length (lib.unique rawSystems) != lib.length rawSystems then
          throw "Plainix: `systems` must not contain duplicates"
        else
          let
            unsupported = builtins.filter (system: !(builtins.elem system supportedSystems)) rawSystems;
          in
          if unsupported == [ ] then
            rawSystems
          else
            throw "Plainix: unsupported system(s): ${join unsupported}; supported systems: ${join supportedSystems}";

      forSystems = lib.genAttrs systems;

      rawDevelop = validateAttrs "develop" [
        "env"
        "packages"
        "shellHook"
      ] (spec.develop or { });

      getPath =
        label: attrs: path:
        lib.attrByPath (lib.splitString "." path) (throw "Plainix: ${label} `${path}` does not exist")
          attrs;

      packagesFor =
        system:
        if builtins.hasAttr system nixpkgs.legacyPackages then
          nixpkgs.legacyPackages.${system}
        else
          throw "Plainix: nixpkgs does not provide packages for system `${system}`";

      evaluate = context: value: if builtins.isFunction value then value context else value;

      validatePackages =
        value:
        if !builtins.isList value then
          throw "Plainix: `develop.packages` must evaluate to a list, but got ${typeOf value}"
        else
          lib.imap0 (
            index: package:
            if builtins.isString package || lib.isDerivation package then
              package
            else
              throw "Plainix: `develop.packages[${toString index}]` must be a package name or package, but got ${typeOf package}"
          ) value;

      validateEnv =
        value:
        if !builtins.isAttrs value then
          throw "Plainix: `develop.env` must evaluate to an attribute set, but got ${typeOf value}"
        else
          lib.mapAttrs (
            name: envValue:
            if
              builtins.isString envValue
              || builtins.isBool envValue
              || builtins.isInt envValue
              || lib.isDerivation envValue
            then
              envValue
            else
              throw "Plainix: `develop.env.${name}` must be a string, boolean, integer, or package, but got ${typeOf envValue}"
          ) value;

      validateString =
        label: value:
        if builtins.isString value then
          value
        else
          throw "Plainix: `${label}` must evaluate to a string, but got ${typeOf value}";

      projectFor =
        system:
        let
          pkgs = packagesFor system;
          resolvePackage = value: if builtins.isString value then getPath "package" pkgs value else value;
          resolvePackages = map resolvePackage;
          context = {
            package = resolvePackage;
            libraryPath = values: pkgs.lib.makeLibraryPath (resolvePackages values);
          };
        in
        {
          inherit pkgs;
          packages = resolvePackages (validatePackages (evaluate context (rawDevelop.packages or [ ])));
          env = validateEnv (evaluate context (rawDevelop.env or { }));
          shellHook = validateString "develop.shellHook" (evaluate context (rawDevelop.shellHook or ""));
        };

      mkDevShell =
        system:
        let
          resolved = projectFor system;
        in
        {
          default = resolved.pkgs.mkShell {
            packages = resolved.packages;
            shellHook = resolved.shellHook;
            env = resolved.env;
          };
        };
    in
    {
      devShells = forSystems mkDevShell;
    };
}
