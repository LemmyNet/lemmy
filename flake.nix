{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils

    , fenix
    , crane
    }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      stdenv =
        if pkgs.stdenv.isLinux then
          pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv
        else
          pkgs.stdenv;

      mkToolchain = fenix.packages.${system}.combine;

      toolchain = fenix.packages.${system}.stable;

      buildToolchain = mkToolchain (with toolchain; [
        cargo
        rustc
      ]);

      devToolchain = mkToolchain (with toolchain; [
        cargo
        clippy
        rust-src
        rustc

        # Always use nightly rustfmt because most of its options are unstable
        fenix.packages.${system}.latest.rustfmt
      ]);

      builder =
        ((crane.mkLib pkgs).overrideToolchain buildToolchain).buildPackage;

      nativeBuildInputs = with pkgs; [
        pkg-config
      ];

      buildInputs = with pkgs; [
        openssl
        postgresql
      ];

      PROTOC = "${pkgs.lib.getExe pkgs.protobuf}";
      PROTOC_INCLUDE = "${pkgs.protobuf}/include";
    in
    {
      packages.default = builder {
        src = ./.;

        inherit
          stdenv
          nativeBuildInputs
          buildInputs
          PROTOC
          PROTOC_INCLUDE;
      };

      devShells.default = (pkgs.mkShell.override { inherit stdenv; }) {
        # Rust Analyzer needs to be able to find the path to default crate
        # sources, and it can read this environment variable to do so. The
        # `rust-src` component is required in order for this to work.
        RUST_SRC_PATH = "${devToolchain}/lib/rustlib/src/rust/library";

        inherit
          PROTOC
          PROTOC_INCLUDE;

        # Development tools
        nativeBuildInputs = buildInputs ++ nativeBuildInputs ++ [
          devToolchain
        ] ++ (with pkgs; [
          engage
          diesel-cli
          nixpkgs-fmt
        ]) ++ (with pkgs.nodePackages; [
          prettier
        ]);
      };

      checks = {
        packagesDefault = self.packages.${system}.default;
        devShellsDefault = self.devShells.${system}.default;
      };
    });
}
