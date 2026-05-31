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
        # from ./lib with include_str!, bootstrap tests read checked Nickel
        # definitions from ./bootstrap, benchmark checks read checked example
        # workloads from ./examples, and the executable transcript tests read
        # checked Markdown fixtures from ./tests/fixtures. Keep these directories
        # alongside normal Cargo sources for Nix-built checks. The bootstrap
        # blocker inventory gate also needs scripts/ and OpenSpec
        # bootstrap text so flake checks inspect the same repo-controlled
        # sources as the local script.
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            (craneLib.filterCargoSources path type)
            || pkgs.lib.hasPrefix "${toString ./lib}/" (toString path)
            || pkgs.lib.hasPrefix "${toString ./bootstrap}/" (toString path)
            || pkgs.lib.hasPrefix "${toString ./builders}/" (toString path)
            || pkgs.lib.hasPrefix "${toString ./examples}/" (toString path)
            || pkgs.lib.hasPrefix "${toString ./tests/fixtures}/" (toString path)
            || pkgs.lib.hasPrefix "${toString ./docs}/" (toString path)
            || pkgs.lib.hasPrefix "${toString ./scripts}/" (toString path)
            || pkgs.lib.hasPrefix "${toString ./openspec}/" (toString path);
        };

        # Common build inputs
        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
          mold
          git
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
          SNIX_BUILD_SANDBOX_SHELL = "/bin/sh";
          GIT = "${pkgs.git}/bin/git";
          nativeCheckInputs = [ pkgs.git ];
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

        releaseNixWitnessQuality = craneLib.cargoNextest {
          pname = "crunch-release-nix-witness-quality";
          inherit src cargoArtifacts buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ [ pkgs.git ];
          cargoNextestExtraArgs = "--test release_cli release_nix_witness";
          partitions = 1;
          partitionType = "count";
          SNIX_BUILD_SANDBOX_SHELL = "/bin/sh";
        };

        mantleTranscriptQuality = craneLib.cargoNextest {
          pname = "mantle-transcript-quality";
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          cargoNextestExtraArgs = "--test transcript_cli";
          partitions = 1;
          partitionType = "count";
          SNIX_BUILD_SANDBOX_SHELL = "/bin/sh";
        };

        bootstrapBlockerInventory = pkgs.runCommand "bootstrap-blocker-inventory" {
          nativeBuildInputs = nativeBuildInputs ++ [
            pkgs.bash
            pkgs.coreutils
            rustToolchain
          ];
        } ''
          cp -R ${src} source
          chmod -R u+w source
          cd source

          export CRUNCH_NIGHTLY_CARGO="${rustToolchain}/bin/cargo"
          export CARGO_HOME="$TMPDIR/cargo-home"
          export RUSTUP_HOME="$TMPDIR/rustup-home"

          bash scripts/check-bootstrap-blocker-inventory.sh \
            --self-test \
            --json "$TMPDIR/bootstrap-blocker-inventory.json" \
            --markdown "$TMPDIR/bootstrap-blocker-inventory.md"

          mkdir -p "$out"
          cp "$TMPDIR/bootstrap-blocker-inventory.json" "$out/current.json"
          cp "$TMPDIR/bootstrap-blocker-inventory.md" "$out/current.md"
        '';
      in
      {
        packages = {
          default = crunch;
          crunch = crunch;
          mantle-transcript-quality = mantleTranscriptQuality;
          release-nix-witness-quality = releaseNixWitnessQuality;
        };

        apps = {
          tigerstyle = flake-utils.lib.mkApp {
            drv = tigerstyleRunner;
            exePath = "/bin/crunch-tigerstyle";
          };
        };

        checks = {
          inherit crunch;
          mantle-transcript-quality = mantleTranscriptQuality;
          bootstrap-blocker-inventory = bootstrapBlockerInventory;
          release-determinism-quality = releaseDeterminismQuality;
          release-nix-witness-quality = releaseNixWitnessQuality;

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
            SNIX_BUILD_SANDBOX_SHELL = "/bin/sh";
            GIT = "${pkgs.git}/bin/git";
          };

          # Clippy lints: keep the flake gate aligned with
          # scripts/check-first-party-clippy.sh by linting first-party targets
          # strictly while excluding vendored workspace members.
          clippy = craneLib.cargoClippy {
            inherit
              src
              cargoArtifacts
              nativeBuildInputs
              buildInputs
              ;
            cargoClippyExtraArgs = "--workspace --lib --no-deps --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing -- -D warnings";
            SNIX_BUILD_SANDBOX_SHELL = "/bin/sh";
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
