# Configuration examples

Each example contains only the compatibility `flake.nix` and the `plainix.nix`
a project member edits.

| Example | What it shows |
| --- | --- |
| [`minimal`](minimal/plainix.nix) | An empty environment ready to customize |
| [`rust`](rust/plainix.nix) | An automatically generated language preset |
| [`sqlite-c`](sqlite-c/plainix.nix) | Variables, a shell hook, and the two advanced helpers |

From a checked-out Plainix repository:

```console
cd examples/rust
nix develop
cargo --version
```

Plainix supplies shared project tools and leaves project-specific workflows to
the project.
