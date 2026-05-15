{
  description = "crunch — Rust project built with Crane";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    tigerstyle = {
      url = "git+ssh://git@github.com/brittonr/tigerstyle-rs.git?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.crane.follows = "crane";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      flake-utils,
      tigerstyle,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common source filtering. The Rust workspace embeds Nickel stdlib files
        # from ./lib with include_str!, so keep that directory alongside normal
        # Cargo sources for Nix-built checks.
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            (craneLib.filterCargoSources path type) || pkgs.lib.hasPrefix "${toString ./lib}/" (toString path);
        };

        # Common build inputs
        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
          mold
        ];

        buildInputs =
          with pkgs;
          [
            openssl
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

        # Build just the cargo dependencies for caching
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src nativeBuildInputs buildInputs;
        };

        # Build the actual package
        crunch = craneLib.buildPackage {
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            ;
        };

        tigerstyleRunner = pkgs.writeShellApplication {
          name = "crunch-tigerstyle";
          runtimeInputs = nativeBuildInputs ++ [
            rustToolchain
            tigerstyle.packages.${system}.cargo-tigerstyle
          ];
          text = ''
            export PKG_CONFIG_PATH="${pkgs.lib.makeSearchPath "lib/pkgconfig" buildInputs}:''${PKG_CONFIG_PATH:-}"
            export SNIX_BUILD_SANDBOX_SHELL="''${SNIX_BUILD_SANDBOX_SHELL:-/bin/sh}"
            exec cargo-tigerstyle "$@"
          '';
        };

        releaseDeterminismQuality = craneLib.cargoNextest {
          pname = "crunch-release-determinism-quality";
          inherit src cargoArtifacts buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ [ pkgs.git ];
          cargoNextestExtraArgs = "--test release_cli release_reproduce_generated_two_clean_store_proof_verifies_deterministic_release";
          partitions = 1;
          partitionType = "count";
          SNIX_BUILD_SANDBOX_SHELL = "/bin/sh";
          MANTLE_FAKE_BWRAP_HOST_PATH = pkgs.lib.makeBinPath [ pkgs.coreutils ];
        };
      in
      {
        packages = {
          default = crunch;
          crunch = crunch;
        };

        apps = {
          tigerstyle = flake-utils.lib.mkApp {
            drv = tigerstyleRunner;
            exePath = "/bin/crunch-tigerstyle";
          };
        };

        checks = {
          inherit crunch;
          release-determinism-quality = releaseDeterminismQuality;

          tigerstyle =
            (tigerstyle.lib.mkConsumerCheck {
              inherit system nativeBuildInputs buildInputs;
              src = ./.;
              cargoLock = ./Cargo.lock;
            }).overrideAttrs
              (_old: {
                SNIX_BUILD_SANDBOX_SHELL = "/bin/sh";
              });

          # Run tests with nextest
          nextest = craneLib.cargoNextest {
            inherit
              src
              cargoArtifacts
              nativeBuildInputs
              buildInputs
              ;
            partitions = 1;
            partitionType = "count";
          };

          # Clippy lints
          clippy = craneLib.cargoClippy {
            inherit
              src
              cargoArtifacts
              nativeBuildInputs
              buildInputs
              ;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          };

          # Format check
          fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages =
            with pkgs;
            [
              cargo-nextest
              cargo-watch
              rust-analyzer
            ]
            ++ [
              tigerstyle.packages.${system}.cargo-tigerstyle
            ];

          # Ensure the nightly toolchain is available
          inputsFrom = [ crunch ];
        };
      }
    );
}
