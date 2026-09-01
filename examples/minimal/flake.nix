{
  description = "Empty project environment with Plainix";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.plainix = {
    url = "path:../..";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs: inputs.plainix.lib.project inputs ./plainix.nix;
}
