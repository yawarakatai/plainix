{
  systems = [
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
  ];

  develop = {
    packages = [
      "gnumake"
      "stdenv.cc"
      "pkg-config"
      "sqlite"
    ];

    env = nix: {
      SQLITE_INCLUDE_DIR = "${nix.package "sqlite.dev"}/include";
      SQLITE_LIBRARY_PATH = nix.libraryPath [ "sqlite" ];
    };

    shellHook = ''
      echo "SQLite C environment ready; run make"
    '';
  };
}
