# Configuration reference

`plainix.nix` describes the tools and environment shared by a project. Most
projects only need `systems` and `packages`.

## Complete shape

```nix
{
  systems = [ "x86_64-linux" ];
  packages = [ "git" ];
  env.EXAMPLE = "value";
  shellHook = "echo ready";
}
```

All fields are optional. Unknown fields are errors.

## `systems`

A non-empty list of supported system names. Duplicate values are rejected.

```nix
systems = [
  "x86_64-linux"
  "aarch64-linux"
  "aarch64-darwin"
];
```

Only these three systems are supported. When omitted, the list defaults to
`x86_64-linux`.

## `packages`

Packages placed on `PATH` inside the project environment.

```nix
packages = [
  "git"
  "python3"
  "python3Packages.pytest"
];
```

Strings are resolved as nixpkgs attribute paths, including dotted names. Each
entry must be a package name or a package returned by `nix.package`. Search for
package names at [search.nixos.org/packages](https://search.nixos.org/packages?channel=unstable).

An empty list is valid:

```nix
packages = [ ];
```

## `env`

Environment variables available after entering the environment.

```nix
env = {
  RUST_BACKTRACE = "1";
  PROJECT_ASSETS = "./assets";
};
```

Values may be strings, booleans, integers, or Nix packages. They are passed via
the structured `env` argument to `pkgs.mkShell`, so they cannot silently replace
Plainix's package list or shell hook.

## `shellHook`

Shell code run when entering the environment.

```nix
shellHook = ''
  echo "Project environment ready"
'';
```

Review a repository's shell hook before running `nix develop` or `direnv allow`.

## System-specific values

`packages`, `env`, and `shellHook` may be functions. A function receives helpers
for resolving a package or constructing a library path:

```nix
{
  env = nix: {
    SQLITE_INCLUDE_DIR = "${nix.package "sqlite.dev"}/include";
    SQLITE_LIBRARY_PATH = nix.libraryPath [ "sqlite" ];
  };
}
```

The context contains only:

- `nix.package "name"`
- `nix.libraryPath [ ... ]`

For requirements beyond these helpers, use ordinary Nix outside
`plainix.nix`. Plainix does not provide a package build DSL or arbitrary flake
outputs.

## Compatibility flake

Plainix generates this file so standard Nix commands can read `plainix.nix`:

```nix
{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.plainix = {
    url = "github:yawarakatai/plainix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs:
    inputs.plainix.lib.project inputs ./plainix.nix;
}
```

`flake.lock` pins the exact nixpkgs revision. Most users do not need to edit
`flake.nix`.
