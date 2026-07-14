{
  description = "crunch — Rust project built with Crane";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    onix-nixpkgs.url = "github:NixOS/nixpkgs/6201e203d09599479a3b3450ed24fa81537ebc4e";
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
    wasi-virt = {
      url = "github:bytecodealliance/wasi-virt/19b174a3244f81ed9b91e067b6901f71665316a8";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      onix-nixpkgs,
      crane,
      rust-overlay,
      flake-utils,
      tigerstyle,
      wasi-virt,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        onixPkgs = import onix-nixpkgs { inherit system; };
        kernelscriptExperiment =
          if system == "x86_64-linux" then
            import ./nix/kernelscript-experiment.nix {
              inherit onixPkgs;
              coreAdapter = kernelscriptCoreAdapter;
              sourceRoot = src;
            }
          else
            null;

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        componentRustVersion = "1.90.0";
        componentRustToolchain = pkgs.rust-bin.stable.${componentRustVersion}.default.override {
          targets = [ "wasm32-wasip2" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        cargoManifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        firstPartyCargoScope =
          pkgs.lib.concatStringsSep " " cargoManifest.workspace.metadata.tigerstyle.default_scope;

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
            || pkgs.lib.hasPrefix "${toString ./packages/kernelscript-experiment}/" (toString path)
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

        astGrepVersion = "0.42.1";
        astGrepUpstream = pkgs.ast-grep;
        astGrepToolchain =
          assert pkgs.lib.assertMsg (
            astGrepUpstream.version == astGrepVersion
          ) "Mantle ast-grep pin drifted: expected ${astGrepVersion}, got ${astGrepUpstream.version}";
          pkgs.runCommand "mantle-ast-grep-toolchain-${astGrepVersion}"
            {
              passthru = {
                version = astGrepVersion;
                upstream = astGrepUpstream;
              };
              meta = astGrepUpstream.meta // {
                mainProgram = "ast-grep";
              };
            }
            ''
              set -eu
              mkdir -p "$out/bin" "$out/share/mantle"
              cp "${astGrepUpstream}/bin/ast-grep" "$out/bin/ast-grep"
              chmod u=rwx,go=rx "$out/bin/ast-grep"
              ln -s ast-grep "$out/bin/sg"

              binaryDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$out/bin/ast-grep")"
              ${pkgs.jq}/bin/jq --null-input --sort-keys \
                --arg schema "mantle-ast-grep-toolchain-identity-v1" \
                --arg package "ast-grep" \
                --arg version "${astGrepVersion}" \
                --arg binary "bin/ast-grep" \
                --arg digestAlgorithm "blake3" \
                --arg binaryDigestBlake3 "$binaryDigest" \
                --arg upstreamStorePath "${astGrepUpstream}" \
                '{
                  schema: $schema,
                  package: $package,
                  version: $version,
                  binary: $binary,
                  digest_algorithm: $digestAlgorithm,
                  binary_digest_blake3: $binaryDigestBlake3,
                  upstream_store_path: $upstreamStorePath
                }' > "$out/share/mantle/ast-grep-toolchain.json"
            '';

        wasiVirtVersion = "0.2.0";
        wasiVirt = pkgs.rustPlatform.buildRustPackage {
          pname = "wasi-virt";
          version = wasiVirtVersion;
          src = wasi-virt;
          cargoLock.lockFile = "${wasi-virt}/Cargo.lock";
          cargoBuildFlags = [
            "--package"
            "wasi-virt"
            "--no-default-features"
          ];
          doCheck = false;
          meta.mainProgram = "wasi-virt";
        };

        wasmComponentToolchain = pkgs.runCommand "mantle-wasm-component-toolchain-v1"
          {
            nativeBuildInputs = [
              pkgs.b3sum
              pkgs.coreutils
              pkgs.jq
            ];
          }
          ''
            set -eu
            mkdir -p "$out/bin" "$out/share/mantle"

            install_tool() {
              source_path="$1"
              tool_name="$2"
              test -x "$source_path"
              cp --dereference "$source_path" "$out/bin/$tool_name"
              chmod u=rwx,go=rx "$out/bin/$tool_name"
            }

            wrap_rust_tool() {
              source_path="$1"
              tool_name="$2"
              test -x "$source_path"
              printf '%s\n' '#!${pkgs.bash}/bin/bash' "exec \"$source_path\" \"\$@\"" > "$out/bin/$tool_name"
              chmod u=rwx,go=rx "$out/bin/$tool_name"
            }

            wrap_rust_tool "${componentRustToolchain}/bin/cargo" cargo
            wrap_rust_tool "${componentRustToolchain}/bin/rustc" rustc
            install_tool "${componentRustToolchain}/lib/rustlib/${pkgs.stdenv.hostPlatform.rust.rustcTarget}/bin/wasm-component-ld" wasm-component-ld
            install_tool "${pkgs.wkg}/bin/wkg" wkg
            install_tool "${pkgs.wit-bindgen}/bin/wit-bindgen" wit-bindgen
            install_tool "${pkgs.wasm-tools}/bin/wasm-tools" wasm-tools
            install_tool "${pkgs.wac-cli}/bin/wac" wac
            install_tool "${wasiVirt}/bin/wasi-virt" wasi-virt
            install_tool "${pkgs.wizer}/bin/wizer" wizer
            install_tool "${pkgs.wasmtime}/bin/wasmtime" wasmtime
            install_tool "${pkgs.bubblewrap}/bin/bwrap" bwrap

            tool_record() {
              tool_name="$1"
              expected_version="$2"
              binary="$out/bin/$tool_name"
              version_output="$($binary --version 2>&1 | ${pkgs.coreutils}/bin/head -n 1)"
              binary_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$binary")"
              ${pkgs.jq}/bin/jq --null-input --sort-keys \
                --arg name "$tool_name" \
                --arg version "$expected_version" \
                --arg versionOutput "$version_output" \
                --arg path "bin/$tool_name" \
                --arg binaryDigestBlake3 "$binary_digest" \
                '{
                  name: $name,
                  version: $version,
                  version_output: $versionOutput,
                  path: $path,
                  binary_digest_blake3: $binaryDigestBlake3
                }'
            }

            tool_record cargo "${componentRustVersion}" > "$TMPDIR/cargo.json"
            tool_record rustc "${componentRustVersion}" > "$TMPDIR/rustc.json"
            tool_record wasm-component-ld "0.5.15" > "$TMPDIR/wasm-component-ld.json"
            tool_record wkg "${pkgs.wkg.version}" > "$TMPDIR/wkg.json"
            tool_record wit-bindgen "${pkgs.wit-bindgen.version}" > "$TMPDIR/wit-bindgen.json"
            tool_record wasm-tools "${pkgs.wasm-tools.version}" > "$TMPDIR/wasm-tools.json"
            tool_record wac "${pkgs.wac-cli.version}" > "$TMPDIR/wac.json"
            tool_record wasi-virt "${wasiVirtVersion}" > "$TMPDIR/wasi-virt.json"
            tool_record wizer "${pkgs.wizer.version}" > "$TMPDIR/wizer.json"
            tool_record wasmtime "${pkgs.wasmtime.version}" > "$TMPDIR/wasmtime.json"
            tool_record bwrap "${pkgs.bubblewrap.version}" > "$TMPDIR/bwrap.json"

            ${pkgs.jq}/bin/jq --null-input --compact-output --sort-keys \
              --arg schema "mantle-wasm-component-toolchain-v1" \
              --arg target "wasm32-wasip2" \
              --slurpfile cargo "$TMPDIR/cargo.json" \
              --slurpfile rustc "$TMPDIR/rustc.json" \
              --slurpfile componentLd "$TMPDIR/wasm-component-ld.json" \
              --slurpfile wkg "$TMPDIR/wkg.json" \
              --slurpfile witBindgen "$TMPDIR/wit-bindgen.json" \
              --slurpfile wasmTools "$TMPDIR/wasm-tools.json" \
              --slurpfile wac "$TMPDIR/wac.json" \
              --slurpfile wasiVirt "$TMPDIR/wasi-virt.json" \
              --slurpfile wizer "$TMPDIR/wizer.json" \
              --slurpfile wasmtime "$TMPDIR/wasmtime.json" \
              --slurpfile bwrap "$TMPDIR/bwrap.json" \
              '{
                schema: $schema,
                rust_target: $target,
                tools: [
                  $cargo[0],
                  $rustc[0],
                  $componentLd[0],
                  $wkg[0],
                  $witBindgen[0],
                  $wasmTools[0],
                  $wac[0],
                  $wasiVirt[0],
                  $wizer[0],
                  $wasmtime[0],
                  $bwrap[0]
                ]
              }' > "$TMPDIR/cohort-input.json"
            cohort_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$TMPDIR/cohort-input.json")"
            ${pkgs.jq}/bin/jq --sort-keys --arg cohortDigest "$cohort_digest" \
              '. + {cohort_identity_blake3: $cohortDigest}' \
              "$TMPDIR/cohort-input.json" > "$out/share/mantle/wasm-component-toolchain.json"
          '';

        wasmComponentToolchainIdentity = pkgs.runCommand "mantle-wasm-component-toolchain-identity"
          {
            nativeBuildInputs = [
              pkgs.b3sum
              pkgs.jq
            ];
          }
          ''
            set -eu
            manifest="${wasmComponentToolchain}/share/mantle/wasm-component-toolchain.json"
            ${pkgs.jq}/bin/jq --exit-status \
              '.schema == "mantle-wasm-component-toolchain-v1"
                and .rust_target == "wasm32-wasip2"
                and (.tools | length) == 11
                and ([.tools[].name] | unique | length) == 11
                and ([.tools[].binary_digest_blake3 | test("^[0-9a-f]{64}$")] | all)
                and (.cohort_identity_blake3 | test("^[0-9a-f]{64}$"))' \
              "$manifest" > /dev/null

            ${pkgs.jq}/bin/jq -cS 'del(.cohort_identity_blake3)' "$manifest" > "$TMPDIR/cohort-input.json"
            actual_cohort_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$TMPDIR/cohort-input.json")"
            expected_cohort_digest="$(${pkgs.jq}/bin/jq --raw-output '.cohort_identity_blake3' "$manifest")"
            test "$actual_cohort_digest" = "$expected_cohort_digest"

            for tool in cargo rustc wasm-component-ld wkg wit-bindgen wasm-tools wac wasi-virt wizer wasmtime bwrap; do
              binary="${wasmComponentToolchain}/bin/$tool"
              test -x "$binary"
              actual_binary_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$binary")"
              expected_binary_digest="$(${pkgs.jq}/bin/jq --raw-output --arg tool "$tool" '.tools[] | select(.name == $tool) | .binary_digest_blake3' "$manifest")"
              test "$actual_binary_digest" = "$expected_binary_digest"
              "$binary" --version > /dev/null 2>&1
            done

            mkdir -p "$out"
            cp "$manifest" "$out/wasm-component-toolchain.json"
          '';

        wasmComponentToolchainCompatibility = pkgs.runCommand "mantle-wasm-component-toolchain-compatibility"
          {
            nativeBuildInputs = [ wasmComponentToolchain ];
          }
          ''
            set -eu
            work="$TMPDIR/component-cohort"
            mkdir -p "$work/src" "$work/home" "$work/cargo-home" "$work/target"
            cat > "$work/Cargo.toml" <<'EOF'
            [package]
            name = "mantle-cohort-smoke"
            version = "0.1.0"
            edition = "2024"
            EOF
            cat > "$work/Cargo.lock" <<'EOF'
            # This file is automatically @generated by Cargo.
            version = 4

            [[package]]
            name = "mantle-cohort-smoke"
            version = "0.1.0"
            EOF
            cat > "$work/src/main.rs" <<'EOF'
            fn main() {
                println!("mantle-component-cohort-ok");
            }
            EOF

            export HOME="$work/home"
            export CARGO_HOME="$work/cargo-home"
            export CARGO_TARGET_DIR="$work/target"
            export CARGO_NET_OFFLINE=true
            export RUSTC="${wasmComponentToolchain}/bin/rustc"
            export PATH="${wasmComponentToolchain}/bin:${pkgs.coreutils}/bin:${pkgs.gnugrep}/bin"
            cd "$work"
            cargo build --locked --offline --release --target wasm32-wasip2
            component="$work/target/wasm32-wasip2/release/mantle-cohort-smoke.wasm"
            wasm-tools validate "$component"
            wasm-tools component wit "$component" > "$work/component.wit"
            grep -F '@0.2.3' "$work/component.wit" > /dev/null
            wasi-virt --wasi-version 0.2.3 --allow-stdio=true --out "$work/virtualized.wasm" "$component"
            wasm-tools validate "$work/virtualized.wasm"
            test "$(wasmtime run "$work/virtualized.wasm")" = "mantle-component-cohort-ok"
            mkdir -p "$out"
            cp "$work/component.wit" "$out/component.wit"
            cp "$work/virtualized.wasm" "$out/virtualized.wasm"
          '';

        astGrepPackageIdentity = pkgs.runCommand "mantle-ast-grep-package-identity-smoke" { } ''
          set -eu
          identity="${astGrepToolchain}/share/mantle/ast-grep-toolchain.json"
          binary="${astGrepToolchain}/bin/ast-grep"
          actualDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$binary")"
          versionOutput="$("$binary" --version)"
          aliasVersionOutput="$("${astGrepToolchain}/bin/sg" --version)"
          test "$aliasVersionOutput" = "$versionOutput"

          ${pkgs.jq}/bin/jq --exit-status \
            --arg schema "mantle-ast-grep-toolchain-identity-v1" \
            --arg package "ast-grep" \
            --arg version "${astGrepVersion}" \
            --arg binaryPath "bin/ast-grep" \
            --arg actualDigest "$actualDigest" \
            --arg upstreamStorePath "${astGrepUpstream}" \
            '.schema == $schema
              and .package == $package
              and .version == $version
              and .binary == $binaryPath
              and .digest_algorithm == "blake3"
              and .binary_digest_blake3 == $actualDigest
              and .upstream_store_path == $upstreamStorePath' \
            "$identity" > /dev/null
          printf '%s\n' "$versionOutput" | ${pkgs.gnugrep}/bin/grep --fixed-strings -- "${astGrepVersion}" > /dev/null

          mkdir -p "$out"
          cp "$identity" "$out/identity.json"
          printf '%s\n' "$versionOutput" > "$out/version.txt"
          printf '%s\n' "$actualDigest" > "$out/binary.blake3"
        '';

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

        kernelscriptCoreAdapter = craneLib.buildPackage {
          pname = "crunch-kernelscript-adapter";
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            ;
          cargoExtraArgs = "--locked -p crunch-kernelscript-adapter --bin mantle-kernelscript-core-adapter";
          doCheck = false;
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
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            ;
          cargoNextestExtraArgs = "--test transcript_cli";
          partitions = 1;
          partitionType = "count";
          SNIX_BUILD_SANDBOX_SHELL = "/bin/sh";
        };

        bootstrapBlockerInventory =
          pkgs.runCommand "bootstrap-blocker-inventory"
            {
              nativeBuildInputs = nativeBuildInputs ++ [
                pkgs.bash
                pkgs.coreutils
                rustToolchain
              ];
            }
            ''
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
          ast-grep-toolchain = astGrepToolchain;
          ast-grep-package-identity = astGrepPackageIdentity;
          wasm-component-toolchain = wasmComponentToolchain;
          wasm-component-toolchain-identity = wasmComponentToolchainIdentity;
          wasm-component-toolchain-compatibility = wasmComponentToolchainCompatibility;
          mantle-transcript-quality = mantleTranscriptQuality;
          release-nix-witness-quality = releaseNixWitnessQuality;
        }
        // pkgs.lib.optionalAttrs (system == "x86_64-linux") {
          kernelscript-compiler = kernelscriptExperiment.compiler;
          kernelscript-core-adapter = kernelscriptCoreAdapter;
          kernelscript-production = kernelscriptExperiment.artifacts;
          kernelscript-production-cohort = kernelscriptExperiment.cohort;
          kernelscript-production-shell = kernelscriptExperiment.productionShell;
          kernelscript-production-runtime-check = kernelscriptExperiment.runtimeCheck;
        };

        apps = {
          tigerstyle = flake-utils.lib.mkApp {
            drv = tigerstyleRunner;
            exePath = "/bin/crunch-tigerstyle";
          };
        };

        checks = {
          inherit crunch;
          ast-grep-package-identity = astGrepPackageIdentity;
          wasm-component-toolchain-identity = wasmComponentToolchainIdentity;
          wasm-component-toolchain-compatibility = wasmComponentToolchainCompatibility;
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
            cargoExtraArgs = firstPartyCargoScope;
          };
        }
        // pkgs.lib.optionalAttrs (system == "x86_64-linux") {
          kernelscript-production = kernelscriptExperiment.structuralCheck;
        };

        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages =
            with pkgs;
            [
              cargo-deny
              cargo-nextest
              cargo-watch
              rust-analyzer
            ]
            ++ [
              astGrepToolchain
              wasmComponentToolchain
              tigerstyle.packages.${system}.cargo-tigerstyle
            ];

          # Ensure the nightly toolchain is available
          inputsFrom = [ crunch ];
        };
      }
    );
}
