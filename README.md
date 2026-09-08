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
is no longer required. Select no tools to start with an empty package list ready
to customize.

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
FreeCAD is currently offered on Linux only.

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
