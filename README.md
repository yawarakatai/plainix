# Plainix

Plainix is the thinnest way for developers using Nix to hand a team a
reproducible project environment. List tools in one small file and teammates
run `nix develop`, on Linux or macOS.

There is no daemon, no runtime CLI, and no lock-in: Plainix generates a standard
Nix flake, so you do not have to write one. It provides tools and environment
variables without prescribing how the project is built or run.

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
is no longer required, while the generated flake keeps Plainix's small library
as a locked input. Select no tools to start with an empty package list ready to
customize.

Nix copies the project into its store when entering the environment: tracked
files inside a Git repository, and everything outside one. Use Git for projects
with large files, and keep large assets out of it where practical.

Plainix never runs `direnv allow` or overwrites an existing `.envrc`. If you use
direnv, review the generated files before allowing them.

## Choose project tools

`plainix init` opens a searchable selection where Space selects any combination
of tools and Enter continues. Presets are an onboarding convenience, not a
limit on what Plainix can manage. They are expanded into an ordinary package
list in `plainix.nix`.

For scripts or non-interactive use, repeat `--with` or explicitly select an
empty environment:

```console
nix run github:yawarakatai/plainix -- init --with godot --with blender
nix run github:yawarakatai/plainix -- init --empty
```

| Tool | `--with` name | Initial packages |
| --- | --- | --- |
| Blender | `blender` | Blender |
| C/C++ | `cpp` | compiler, build tools, clangd, clang-format, clang-tidy |
| FreeCAD | `freecad` | FreeCAD |
| Go | `go` | Go, gopls, golangci-lint |
| Godot | `godot` | Godot 4 |
| Haskell | `haskell` | GHC, Cabal, HLS, Ormolu, HLint |
| KiCad | `kicad` | KiCad |
| Lua | `lua` | Lua, language server, StyLua, Luacheck |
| Nix | `nix` | nixd, nixfmt, Statix, deadnix |
| OCaml | `ocaml` | OCaml, Dune, OCaml-LSP, ocamlformat |
| Python | `python` | Python, basedpyright, Ruff, uv |
| Rust | `rust` | Cargo, Clippy, rust-analyzer, rustc, rustfmt |
| Shell | `shell` | Bash, language server, shfmt, ShellCheck |
| TOML | `toml` | Taplo language server, formatter, validator |
| JavaScript / TypeScript | `typescript` | Node.js, TypeScript, language server, Biome |
| YAML | `yaml` | language server, Prettier, yamllint |
| Zig | `zig` | Zig, ZLS |

Supported platforms are `x86_64-linux`, `aarch64-linux`, and `aarch64-darwin`.
`plainix init` declares every platform the selected tools support. FreeCAD and
KiCad are currently offered on Linux only.

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
produce Plainix errors instead of being silently ignored. Add
`allowUnfree = true;` to use packages with unfree licenses. Host-level setup
such as drivers, USB permissions, and proprietary SDK licensing remains outside
Plainix.

## Update the environment

```console
nix run github:yawarakatai/plainix -- update
```

This moves `flake.lock` to the latest nixpkgs and Plainix, then evaluates the
environment for every system in `plainix.nix`. If any of them fails, Plainix
names the cause and restores the previous `flake.lock`. It does not build the
packages or commit anything. Plain `nix flake update` also works, without the
check.

## Documentation

- [Configuration reference](docs/reference.md)
- [Configuration examples](examples/README.md)

Plainix is an early MVP. Generated projects use standard Nix flakes and remain
usable without the Plainix CLI.
