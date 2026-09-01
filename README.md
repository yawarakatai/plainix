# Plainix

Plainix gives any project a reproducible, shareable environment of tools. The
project can be software, a game, device firmware, data work, or a creative
workspace.

Plainix uses Nix underneath, but you do not need to understand flakes. It
provides project tools and environment variables without prescribing how the
project is built or run.

## Get started

Install [Nix](https://nixos.org/download/) and enable the `nix-command` and
`flakes` experimental features. If needed, add
`experimental-features = nix-command flakes` to `~/.config/nix/nix.conf` or
`/etc/nix/nix.conf`.

From any project directory, including an empty one:

```console
cd your-project
nix run github:yawarakatai/plainix -- init
nix develop
```

Plainix creates:

- `plainix.nix` — the shared tools and environment
- `flake.nix` — compatibility with standard Nix commands
- `flake.lock` — pinned versions for reproducibility
- `.envrc` — optional automatic activation with direnv and nix-direnv

Inside a Git repository, Plainix also adds `.direnv/` to `.gitignore` and marks
the generated files as intent-to-add. This lets Nix see them without staging
their contents. Review the files and run the printed `git add` command before
committing them.

After initialization, `nix develop` is the only command needed; the Plainix CLI
is no longer required. If no automatic preset matches, Plainix starts with an
empty package list ready to customize.

Plainix never runs `direnv allow` or overwrites an existing `.envrc`. If you use
direnv, review the generated files before allowing them.

## Automatic presets

Presets are an onboarding convenience, not a limit on what Plainix can manage.
Plainix combines them when a repository contains multiple languages.

| Preset | Detected from | Initial tools |
| --- | --- | --- |
| Rust | `Cargo.toml` | Cargo, Clippy, rust-analyzer, rustc, rustfmt |
| Python | `pyproject.toml` | Python, Ruff, uv |
| TypeScript | `package.json`, `tsconfig.json` | Node.js, TypeScript, language server |
| Go | `go.mod` | Go, gopls |
| Zig | `build.zig`, `build.zig.zon`, top-level `.zig` | Zig, ZLS |
| C/C++ | CMake, Meson, top-level C/C++ source | compiler, CMake, Make, Ninja, pkg-config |

Supported platforms are `x86_64-linux`, `aarch64-linux`, and `aarch64-darwin`.

## Customize the environment

Edit `plainix.nix` directly. Generated files include the nixpkgs source and a
link to [search.nixos.org/packages](https://search.nixos.org/packages?channel=unstable).

```nix
{
  systems = [ "x86_64-linux" ];

  packages = [
    "blender"
    "godot_4"
    "openocd"
  ];

  env.PROJECT_ASSETS = "./assets";
}
```

Typos, unsupported fields, invalid package values, and unsupported platforms
produce Plainix errors instead of being silently ignored. Host-level setup such
as drivers, USB permissions, and proprietary SDK licensing remains outside
Plainix.

## Documentation

- [Configuration reference](docs/reference.md)
- [Configuration examples](examples/README.md)

Plainix is an early MVP. Generated projects use standard Nix flakes and remain
usable without the Plainix CLI.
