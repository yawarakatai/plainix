# Configuration reference

`plainix.nix` describes a development shell. Most projects only need
`systems` and `develop.packages`.

## Complete shape

```nix
{
  systems = [ "x86_64-linux" ];

  develop = {
    packages = [ "git" ];
    env.EXAMPLE = "value";
    shellHook = "echo ready";
  };
}
```

Unknown fields are errors.

## `systems`

A non-empty list of Nix system names. Duplicate values are rejected.

```nix
systems = [
  "x86_64-linux"
  "aarch64-linux"
  "aarch64-darwin"
];
```

When omitted, it defaults to `x86_64-linux`. A system must be available from
the selected nixpkgs revision.

## `develop.packages`

Packages placed on `PATH` inside `nix develop`.

```nix
develop.packages = [
  "git"
  "python3"
  "python3Packages.pytest"
];
```

Strings are resolved as nixpkgs attribute paths, including dotted names.

## `develop.env`

Environment variables available in the shell.

```nix
develop.env = {
  RUST_BACKTRACE = "1";
  APP_ENV = "development";
};
```

Values may be strings, booleans, integers, or Nix packages. They are passed via
the structured `env` argument to `pkgs.mkShell`, so they cannot silently replace
Plainix's `packages` or `shellHook` settings.

## `develop.shellHook`

Shell code run when entering the environment.

```nix
develop.shellHook = ''
  echo "Development environment ready"
'';
```

Review a repository's shell hook before running `nix develop` or `direnv allow`.

## System-specific values

The three `develop` fields may be functions. A function receives helpers for
resolving a package or constructing a library path:

```nix
{
  develop.env = nix: {
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

Most users do not need to edit `flake.nix`.
