{
  systems = [
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
  ];

  packages = [
    "cargo"
    "clippy"
    "cmake"
    "go"
    "gnumake"
    "gopls"
    "ninja"
    "nodejs_22"
    "pkg-config"
    "python3"
    "ruff"
    "rust-analyzer"
    "rustc"
    "rustfmt"
    "stdenv.cc"
    "typescript"
    "typescript-language-server"
    "uv"
    "zig"
    "zls"
  ];
}
