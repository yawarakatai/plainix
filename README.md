# Plainix

Plainix adds a reproducible Nix development environment to an existing project.
You do not need to understand flakes to get started.

Plainix manages development tools only. It does not build or run your
application.

## Get started

Install [Nix](https://nixos.org/download/) and enable the `nix-command` and
`flakes` experimental features. If needed, add
`experimental-features = nix-command flakes` to `~/.config/nix/nix.conf` or
`/etc/nix/nix.conf`.

From the root of an existing Git project:

```console
cd your-project
nix run github:yawarakatai/plainix -- init
git add plainix.nix flake.nix flake.lock
nix develop
```

Plainix detects the project and creates three files:

- `plainix.nix` — a readable list of development tools
- `flake.nix` — compatibility with standard Nix commands
- `flake.lock` — pinned versions for reproducibility

After initialization, use `nix develop` directly; the Plainix CLI is no longer
needed. Outside a Git repository, omit the `git add` command.

For direnv integration, initialize with
`nix run github:yawarakatai/plainix -- init --direnv`, then run `direnv allow`.
Plainix never runs `direnv allow` or overwrites an existing `.envrc`.

## Supported projects

Plainix combines presets when a repository contains multiple languages.

| Language   | Detected from                                  | Development tools                            |
| ---------- | ---------------------------------------------- | -------------------------------------------- |
| Rust       | `Cargo.toml`                                   | Cargo, Clippy, rust-analyzer, rustc, rustfmt |
| Python     | `pyproject.toml`                               | Python, Ruff, uv                             |
| TypeScript | `package.json`, `tsconfig.json`                | Node.js, TypeScript, language server         |
| Go         | `go.mod`                                       | Go, gopls                                    |
| Zig        | `build.zig`, `build.zig.zon`, top-level `.zig` | Zig, ZLS                                     |
| C/C++      | CMake, Meson, top-level C/C++ source           | compiler, CMake, Make, Ninja, pkg-config     |

Supported platforms are `x86_64-linux`, `aarch64-linux`, and `aarch64-darwin`.

## Customize the environment

Edit `plainix.nix` directly. Package names come from
[search.nixos.org/packages](https://search.nixos.org/packages).

```nix
{
  systems = [ "x86_64-linux" ];

  develop.packages = [
    "git"
    "python3"
    "sqlite"
  ];

  develop.env.RUST_BACKTRACE = "1";
}
```

Typos, unsupported fields, invalid package values, and unsupported platforms
produce Plainix errors instead of being silently ignored.
