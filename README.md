# Plainix

Plainix creates a reproducible Nix development environment for an existing
project. You do not need to understand flakes to get started.

```console
cd your-project
nix run github:yawarakatai/plainix -- init
nix develop
```

Plainix detects the project, writes a readable `plainix.nix`, generates the Nix
compatibility files, and pins their inputs. After that, the Plainix CLI is not
needed: use `nix develop` to enter the environment.

> Plainix currently manages development tools only. It does not build or run
> your application for you.

## Before you start

[Install Nix](https://nixos.org/download/) and enable its `nix-command` and
`flakes` experimental features. Add this to `~/.config/nix/nix.conf` or
`/etc/nix/nix.conf`:

```ini
experimental-features = nix-command flakes
```

## Quick start

Run Plainix from the root of an existing project:

```console
nix run github:yawarakatai/plainix -- init
```

For a Rust project, it generates a package list like this:

```nix
{
  systems = [ "x86_64-linux" ];

  develop.packages = [
    "cargo"
    "clippy"
    "rust-analyzer"
    "rustc"
    "rustfmt"
  ];
}
```

It also creates:

- `flake.nix` — compatibility with standard Nix commands
- `flake.lock` — pinned versions for reproducibility

Commit all three files. In a Git repository, Plainix prints the required
`git add` command because Nix cannot see untracked flake files.

Enter and leave the environment with:

```console
nix develop
# use cargo, python, go, etc.
exit
```

## Supported projects

Plainix can combine presets when a repository contains multiple languages.

| Language   | Detected from                                  | Development tools                            |
| ---------- | ---------------------------------------------- | -------------------------------------------- |
| Rust       | `Cargo.toml`                                   | Cargo, Clippy, rust-analyzer, rustc, rustfmt |
| Python     | `pyproject.toml`                               | Python, Ruff, uv                             |
| TypeScript | `package.json`, `tsconfig.json`                | Node.js, TypeScript, language server         |
| Go         | `go.mod`                                       | Go, gopls                                    |
| Zig        | `build.zig`, `build.zig.zon`, top-level `.zig` | Zig, ZLS                                     |
| C/C++      | CMake, Meson, top-level C/C++ source           | compiler, CMake, Make, Ninja, pkg-config     |

Generated environments currently target `x86_64-linux`, `aarch64-linux`, and
`aarch64-darwin`.

## Change the environment

Edit `plainix.nix` directly. Package names come from
[search.nixos.org/packages](https://search.nixos.org/packages).

```nix
{
  systems = [ "x86_64-linux" ];

  develop = {
    packages = [
      "git"
      "python3"
      "sqlite"
    ];

    env.RUST_BACKTRACE = "1";

    shellHook = ''
      echo "Development environment ready"
    '';
  };
}
```

A typo or unsupported field produces a Plainix error instead of being silently
ignored. See [Configuration reference](docs/reference.md) for all fields and
the two system-specific helpers.

## Optional direnv integration

If `direnv` and `nix-direnv` are installed, Plainix can generate `.envrc`:

```console
nix run github:yawarakatai/plainix -- init --direnv
direnv allow
```

Plainix never runs `direnv allow` automatically. An existing `.envrc` is not
overwritten.

## More information

- [Configuration reference](docs/reference.md)
- [Configuration examples](examples/README.md)

Plainix is an early MVP. The generated files use standard Nix flakes, so a
project remains usable with Nix even when the Plainix CLI is not installed.
