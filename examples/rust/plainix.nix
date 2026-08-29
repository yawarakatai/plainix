{
  systems = [
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
  ];

  develop.packages = [
    "cargo"
    "clippy"
    "rust-analyzer"
    "rustc"
    "rustfmt"
  ];
}
