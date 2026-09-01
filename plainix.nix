{
  systems = [
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
  ];

  packages = [
    "cargo"
    "clippy"
    "nixfmt-tree"
    "rust-analyzer"
    "rustc"
    "rustfmt"
  ];
}
