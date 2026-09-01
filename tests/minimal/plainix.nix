{
  systems = [
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
  ];

  packages = [ "git" ];
  env.RUST_BACKTRACE = "1";
  shellHook = "echo Plainix test";
}
