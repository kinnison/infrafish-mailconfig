{
  description = "Infrafish Mail Config";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        mailconfig = pkgs.rustPlatform.buildRustPackage {
          pname = "infrafish-mailconfig";
          version = "git";
          src = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };

          nativeBuildInputs = with pkgs; [
            postgresql
            libpq
            libpq.pg_config
            pkg-config
          ];
        };
      in {
        packages = {
          inherit mailconfig;
          default = mailconfig;
        };
        devShell = pkgs.callPackage ./shell.nix { };
      }));
}
