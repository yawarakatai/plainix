# Configuration examples

Each example contains only the generated `flake.nix` and the `plainix.nix` a
developer edits.

| Example | What it shows |
| --- | --- |
| [`minimal`](minimal/plainix.nix) | Smallest package-only environment |
| [`rust`](rust/plainix.nix) | A generated language preset without Fenix |
| [`sqlite-c`](sqlite-c/plainix.nix) | Environment variables and the two advanced helpers |

From a checked-out Plainix repository:

```console
cd examples/rust
nix develop
cargo --version
```

Plainix supplies development tools and leaves project builds to each language's
normal tooling.
