{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    flake-utils.inputs.nixpkgs.follows = "nixpkgs";
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, devenv, fenix, flake-utils, crane, ... } @ inputs:
    flake-utils.lib.eachSystem [ flake-utils.lib.system.x86_64-linux ]
      (system:
        let
          pkgs = import nixpkgs { inherit system; };
          toolchain = (fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-gdYqng0y9iHYzYPAdkC/ka3DRny3La/S5G8ASj0Ayyc=";
          });
          craneLib = crane.lib.${system}.overrideToolchain toolchain;

          src = craneLib.cleanCargoSource (craneLib.path ./.);

          env = {
            OPENSSL_LIB_DIR = "${pkgs.openssl}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          };

          buildInputs = with pkgs; [
            postgresql
          ];

          commonArgs = {
            inherit buildInputs src env;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          lemmy = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        in
        {
          # TODO: nix build can't build the flake that has a submodule dependency
          #       see https://github.com/NixOS/nix/pull/7862 and
          #       https://github.com/NixOS/nix/issues/4423 for further details.
          # packages.default = lemmy;
          devShells.default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [{
              inherit env;

              packages = with pkgs; [
                toolchain
                cargo-whatfeatures
                cargo-watch
              ] ++ buildInputs;

              services.postgres = {
                enable = true;
                listen_addresses = "127.0.0.1";
                initialDatabases = [{ name = "lemmy"; }];
                initialScript = ''
                  CREATE USER postgres WITH PASSWORD 'password' SUPERUSER;
                  CREATE USER lemmy WITH PASSWORD 'password';
                  GRANT ALL PRIVILEGES ON DATABASE lemmy TO lemmy;
                '';
              };
            }];
          };
        }
      );
}
