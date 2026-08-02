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
    nickelExportCore = {
      url = "github:OnixResearch/nickel-export/257fafc1c746f1faf156207043a4c826bfb16d49";
      flake = false;
    };
    artifactAuthSource = {
      url = "git+https://git.onix.computer/z4JGYYW7WsesXUq7MXVdx16Fawu2f.git?rev=799459346d5416fbd7b9f55840a7371441b55afa";
      flake = false;
    };
    secretSpecSource = {
      url = "github:cachix/secretspec/a8794e46ec9664a0e1a3869cc3105d0853937e48";
      flake = false;
    };
    octet.url = "github:OnixResearch/octet/86ee46b3b9257b145d2dbeb6ce9d9897607db99c";
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
      nickelExportCore,
      artifactAuthSource,
      octet,
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
        spacewasmReference =
          if system == "x86_64-linux" then
            import ./nix/spacewasm-reference.nix {
              inherit pkgs;
              packageRoot = ./packages/spacewasm-reference;
              coreCrate = ./crates/crunch-spacewasm-core;
              shellCrate = ./crates/crunch-spacewasm;
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
        artifactAuthRevision = "799459346d5416fbd7b9f55840a7371441b55afa";
        artifactAuthRepository = "https://git.onix.computer/z4JGYYW7WsesXUq7MXVdx16Fawu2f.git";
        artifactAuthExpectedPackages = [
          "artifact-auth-core"
          "artifact-auth-ed25519"
        ];
        artifactAuthCoreManifest = builtins.fromTOML (
          builtins.readFile ./crates/crunch-action-result-core/Cargo.toml
        );
        artifactAuthShellManifest = builtins.fromTOML (builtins.readFile ./crates/crunch-build/Cargo.toml);
        artifactAuthCargoDependencies = [
          artifactAuthCoreManifest.dependencies.artifact-auth-core
          artifactAuthShellManifest.dependencies.artifact-auth-core
          artifactAuthShellManifest.dependencies.artifact-auth-ed25519
        ];
        artifactAuthLockPackages = builtins.filter (
          package: builtins.elem package.name artifactAuthExpectedPackages
        ) (builtins.fromTOML (builtins.readFile ./Cargo.lock)).package;
        artifactAuthExpectedLockSource = "git+${artifactAuthRepository}?rev=${artifactAuthRevision}#${artifactAuthRevision}";
        artifactAuthWorkspace = builtins.fromTOML (builtins.readFile (artifactAuthSource + "/Cargo.toml"));
        artifactAuthSourceAdmitted =
          assert pkgs.lib.assertMsg (
            builtins.all (
              dependency: dependency.git == artifactAuthRepository && dependency.rev == artifactAuthRevision
            ) artifactAuthCargoDependencies
            && artifactAuthSource.rev == artifactAuthRevision
            && map (package: package.name) artifactAuthLockPackages == artifactAuthExpectedPackages
            && builtins.all (package: package.source == artifactAuthExpectedLockSource) artifactAuthLockPackages
            && builtins.elem "crates/artifact-auth-core" artifactAuthWorkspace.workspace.members
            && builtins.elem "crates/artifact-auth-ed25519" artifactAuthWorkspace.workspace.members
            && artifactAuthWorkspace.workspace.package.license == "MIT OR Apache-2.0"
          ) "Mantle artifact-auth Cargo/Nix source identity, package set, or license drifted";
          true;
        nickelExportCoreRevision = "257fafc1c746f1faf156207043a4c826bfb16d49";
        nickelExportCoreSource =
          assert pkgs.lib.assertMsg (
            nickelExportCore.rev == nickelExportCoreRevision
          ) "Mantle nickel-export-core Nix input drifted from ${nickelExportCoreRevision}";
          nickelExportCore;
        firstPartyCargoScope = pkgs.lib.concatStringsSep " " cargoManifest.workspace.metadata.tigerstyle.default_scope;
        catalogExampleRelativePaths = builtins.filter (path: path != null) (
          map (
            line:
            let
              matched = builtins.match "[[:space:]]*path = \"([^\"]+)\",[[:space:]]*" line;
            in
            if matched == null then null else builtins.head matched
          ) (pkgs.lib.splitString "\n" (builtins.readFile ./examples/catalog.ncl))
        );
        catalogExamplePaths = map (
          relativePath: "${toString ./.}/${relativePath}"
        ) catalogExampleRelativePaths;
        caCertificateBundlePath = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
        sandboxShellPath =
          if pkgs.stdenv.isLinux then "${pkgs.pkgsStatic.busybox}/bin/busybox" else "/bin/sh";

        # Common source filtering. The Rust workspace embeds Nickel stdlib,
        # generated runtime policy, archived lifecycle evidence, and CI workflow
        # fixtures with include_str!/include_bytes!. Bootstrap tests read checked
        # Nickel definitions, benchmark checks read checked example workloads,
        # and executable transcript tests read checked Markdown fixtures. Keep
        # these inputs alongside normal Cargo sources for Nix-built checks. The
        # bootstrap blocker inventory gate also needs scripts/ and OpenSpec
        # bootstrap text so flake checks inspect the same repo-controlled sources
        # as the local script.
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            let
              rootPath = toString ./.;
              pathString = toString path;
              gitMetadataPath = "${rootPath}/.git";
            in
            pathString != gitMetadataPath
            && !pkgs.lib.hasPrefix "${gitMetadataPath}/" pathString
            && (
              (craneLib.filterCargoSources path type)
              || pathString == toString ./README.md
              || pathString == toString ./flake.nix
              || pkgs.lib.hasPrefix "${toString ./.github/workflows}/" pathString
              || pkgs.lib.hasPrefix "${toString ./lib}/" pathString
              || pkgs.lib.hasPrefix "${toString ./bootstrap}/" pathString
              || pkgs.lib.hasPrefix "${toString ./builders}/" pathString
              || pkgs.lib.hasPrefix "${toString ./cairn-policy/evidence}/" pathString
              || pkgs.lib.hasPrefix "${toString ./cairn/archive}/" pathString
              || pkgs.lib.hasPrefix "${toString ./config/action-result-policy}/" pathString
              || builtins.elem pathString catalogExamplePaths
              || pathString == toString ./examples/catalog.ncl
              || pathString == toString ./examples/README.md
              || pathString == toString ./examples/project/README.md
              || pkgs.lib.hasPrefix "${toString ./examples/projects}/" pathString
              || pkgs.lib.hasPrefix "${toString ./examples/transcripts}/" pathString
              || pkgs.lib.hasPrefix "${toString ./schemas/machine-contracts}/" pathString
              || pkgs.lib.hasPrefix "${toString ./tests/fixtures}/" pathString
              || pkgs.lib.hasPrefix "${toString ./packages/kernelscript-experiment}/" pathString
              || pathString == toString ./nix/kernelscript-experiment.nix
              || pkgs.lib.hasPrefix "${toString ./docs}/" pathString
              || pkgs.lib.hasPrefix "${toString ./scripts}/" pathString
              || pkgs.lib.hasPrefix "${toString ./openspec}/" pathString
            );
        };

        artifactAuthCargoVendorDir = craneLib.vendorCargoDeps {
          inherit src;
          cargoLock = ./Cargo.lock;
          overrideVendorGitCheckout =
            packages: checkout:
            if
              artifactAuthSourceAdmitted
              && builtins.any (package: builtins.elem package.name artifactAuthExpectedPackages) packages
            then
              checkout.overrideAttrs (_old: {
                src = artifactAuthSource;
              })
            else
              checkout;
        };

        # Common build inputs
        nativeBuildInputs =
          with pkgs;
          [
            pkg-config
            clang
            mold
            lld
            git
            cmake
            bash
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.bubblewrap
            pkgs.fuse3.bin
            pkgs.pkgsCross.musl64.stdenv.cc
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
        octetSourceRevision = "86ee46b3b9257b145d2dbeb6ce9d9897607db99c";
        octetSourceRepository = "https://github.com/OnixResearch/octet";
        octetPackageName = "cargo-octet";
        octetPackageVersion = "0.1.0";
        octetProfileId = "portable-component-baseline";
        octetPackage = octet.packages.${system}.cargo-octet;
        octetProfileConfig = "${octet}/standards/wasm-artifact/generated/profiles.json";
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

        wasmComponentToolchain =
          pkgs.runCommand "mantle-wasm-component-toolchain-v1"
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
                wrap_rust_tool "${octetPackage}/bin/cargo-octet" cargo-octet
                mkdir -p "$out/share/mantle/octet"
                cp "${octetProfileConfig}" "$out/share/mantle/octet/wasm-artifact-profiles.json"
                chmod u=rw,go=r "$out/share/mantle/octet/wasm-artifact-profiles.json"

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
                tool_record cargo-octet "${octetPackageVersion}" > "$TMPDIR/cargo-octet.json"

                octet_config="$out/share/mantle/octet/wasm-artifact-profiles.json"
                octet_config_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$octet_config")"
                "$out/bin/wasm-tools" parse "${octet}/standards/wasm-artifact/fixtures/positive/component.wat" \
                  -o "$TMPDIR/octet-positive-component.wasm"
                "$out/bin/cargo-octet" evidence collect \
                  --rail wasm-artifact \
                  --input "$TMPDIR/octet-positive-component.wasm" \
                  --config "$octet_config" \
                  --profile "${octetProfileId}" \
                  --artifact-dir "$TMPDIR/octet-profile-evidence" \
                  --output-format json > "$TMPDIR/octet-profile-run.json"
                "$out/bin/cargo-octet" artifact verify \
                  --artifact-dir "$TMPDIR/octet-profile-evidence" \
                  --output-format json > "$TMPDIR/octet-profile-verify.json"
                ${pkgs.jq}/bin/jq --exit-status '.status == "valid" and (.diagnostics | length) == 0' \
                  "$TMPDIR/octet-profile-verify.json" > /dev/null
                octet_profile_identity="$(${pkgs.jq}/bin/jq --raw-output '.wasm_artifact.profile_identity | sub("^b3:"; "")' "$TMPDIR/octet-profile-evidence/evidence-rail-receipt.json")"
                octet_cohort_identity="$(${pkgs.jq}/bin/jq --raw-output '.wasm_artifact.cohort_identity | sub("^b3:"; "")' "$TMPDIR/octet-profile-evidence/evidence-rail-receipt.json")"
                octet_registry_identity="$(${pkgs.jq}/bin/jq --raw-output '.wasm_artifact.registry_identity | sub("^b3:"; "")' "$TMPDIR/octet-profile-evidence/evidence-rail-receipt.json")"
                test "$octet_registry_identity" = "$octet_config_digest"
                test "$octet_cohort_identity" = "c50e2d7f0e8c49de4a1d44afae196bdf96bb14e67e7de0a153de146a6207449a"
                ${pkgs.jq}/bin/jq --null-input --compact-output --sort-keys \
                  --arg sourceRepository "${octetSourceRepository}" \
                  --arg sourceRevision "${octetSourceRevision}" \
                  --arg packageName "${octetPackageName}" \
                  --arg packageVersion "${octetPackageVersion}" \
                  --arg configPath "share/mantle/octet/wasm-artifact-profiles.json" \
                  --arg configDigest "$octet_config_digest" \
                  --arg profileId "${octetProfileId}" \
                  --arg profileIdentity "$octet_profile_identity" \
                  --arg cohortIdentity "$octet_cohort_identity" \
                  '{
                    source_repository: $sourceRepository,
                    source_revision: $sourceRevision,
                    package_name: $packageName,
                    package_version: $packageVersion,
                    config_path: $configPath,
                    config_digest_blake3: $configDigest,
                    profile_id: $profileId,
                    profile_identity_blake3: $profileIdentity,
                    wasm_tools_cohort_identity_blake3: $cohortIdentity
                  }' > "$TMPDIR/octet.json"

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
                  --slurpfile cargoOctet "$TMPDIR/cargo-octet.json" \
                  --slurpfile octet "$TMPDIR/octet.json" \
                '{
                  schema: $schema,
                  rust_target: $target,
                    octet: $octet[0],
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
                      $bwrap[0],
                      $cargoOctet[0]
                  ]
                }' > "$TMPDIR/cohort-input.json"
              cohort_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$TMPDIR/cohort-input.json")"
              ${pkgs.jq}/bin/jq --sort-keys --arg cohortDigest "$cohort_digest" \
                '. + {cohort_identity_blake3: $cohortDigest}' \
                "$TMPDIR/cohort-input.json" > "$out/share/mantle/wasm-component-toolchain.json"
            '';

        wasmComponentToolchainIdentity =
          pkgs.runCommand "mantle-wasm-component-toolchain-identity"
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
                    and (.tools | length) == 12
                    and ([.tools[].name] | unique | length) == 12
                  and ([.tools[].binary_digest_blake3 | test("^[0-9a-f]{64}$")] | all)
                    and (.octet.source_repository == "${octetSourceRepository}")
                    and (.octet.source_revision == "${octetSourceRevision}")
                    and (.octet.package_name == "${octetPackageName}")
                    and (.octet.package_version == "${octetPackageVersion}")
                    and (.octet.profile_id == "${octetProfileId}")
                    and (.octet.config_digest_blake3 | test("^[0-9a-f]{64}$"))
                    and (.octet.profile_identity_blake3 | test("^[0-9a-f]{64}$"))
                    and (.octet.wasm_tools_cohort_identity_blake3 == "c50e2d7f0e8c49de4a1d44afae196bdf96bb14e67e7de0a153de146a6207449a")
                  and (.cohort_identity_blake3 | test("^[0-9a-f]{64}$"))' \
                "$manifest" > /dev/null

              ${pkgs.jq}/bin/jq -cS 'del(.cohort_identity_blake3)' "$manifest" > "$TMPDIR/cohort-input.json"
              actual_cohort_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$TMPDIR/cohort-input.json")"
              expected_cohort_digest="$(${pkgs.jq}/bin/jq --raw-output '.cohort_identity_blake3' "$manifest")"
              test "$actual_cohort_digest" = "$expected_cohort_digest"

                for tool in cargo rustc wasm-component-ld wkg wit-bindgen wasm-tools wac wasi-virt wizer wasmtime bwrap cargo-octet; do
                binary="${wasmComponentToolchain}/bin/$tool"
                test -x "$binary"
                actual_binary_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$binary")"
                expected_binary_digest="$(${pkgs.jq}/bin/jq --raw-output --arg tool "$tool" '.tools[] | select(.name == $tool) | .binary_digest_blake3' "$manifest")"
                test "$actual_binary_digest" = "$expected_binary_digest"
                "$binary" --version > /dev/null 2>&1
              done
                octet_config="${wasmComponentToolchain}/share/mantle/octet/wasm-artifact-profiles.json"
                actual_octet_config_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$octet_config")"
                expected_octet_config_digest="$(${pkgs.jq}/bin/jq --raw-output '.octet.config_digest_blake3' "$manifest")"
                test "$actual_octet_config_digest" = "$expected_octet_config_digest"

              mkdir -p "$out"
              cp "$manifest" "$out/wasm-component-toolchain.json"
            '';

        wasmComponentToolchainCompatibility =
          pkgs.runCommand "mantle-wasm-component-toolchain-compatibility"
            {
              nativeBuildInputs = [
                wasmComponentToolchain
                pkgs.b3sum
                pkgs.jq
              ];
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

                printf '\x00asm\x0d\x00\x01\x00' > "$work/octet-smoke.component.wasm"
                octet_input_digest="$(${pkgs.b3sum}/bin/b3sum --no-names "$work/octet-smoke.component.wasm")"
                octet_config="${wasmComponentToolchain}/share/mantle/octet/wasm-artifact-profiles.json"
                if ! cargo-octet evidence collect \
                  --rail wasm-artifact \
                  --input "$work/octet-smoke.component.wasm" \
                  --config "$octet_config" \
                  --profile portable-component-baseline \
                  --parent-artifact "mantle:$octet_input_digest" \
                  --artifact-dir "$work/octet-evidence" \
                  --output-format json > "$work/octet-collect.json"; then
                  cat "$work/octet-collect.json"
                  exit 1
                fi
                if ! cargo-octet artifact verify \
                  --artifact-dir "$work/octet-evidence" \
                  --output-format json > "$work/octet-verify.json"; then
                  cat "$work/octet-verify.json"
                  exit 1
                fi
                ${pkgs.jq}/bin/jq --exit-status \
                  --arg input "b3:$octet_input_digest" \
                  '.status == "passed"
                    and .profile == "portable-component-baseline"
                    and .wasm_artifact.exact_artifact_identity == $input
                    and .wasm_artifact.verification_role == "recorded_only"' \
                  "$work/octet-evidence/evidence-rail-receipt.json" > /dev/null
                ${pkgs.jq}/bin/jq --exit-status '.status == "valid" and (.diagnostics | length) == 0' \
                  "$work/octet-verify.json" > /dev/null

                mkdir -p "$out/octet-evidence"
              cp "$work/component.wit" "$out/component.wit"
              cp "$work/virtualized.wasm" "$out/virtualized.wasm"
                cp "$work/octet-verify.json" "$out/octet-verification.json"
                cp "$work/octet-evidence/"* "$out/octet-evidence/"
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
          cargoVendorDir = artifactAuthCargoVendorDir;
        };

        # Build the actual package
        crunch = craneLib.buildPackage {
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            ;
          cargoVendorDir = artifactAuthCargoVendorDir;
          SNIX_BUILD_SANDBOX_SHELL = sandboxShellPath;
          GIT = "${pkgs.git}/bin/git";
          SSL_CERT_FILE = caCertificateBundlePath;
          NIX_SSL_CERT_FILE = caCertificateBundlePath;
          MANTLE_TEST_REAL_BWRAP = "${pkgs.bubblewrap}/bin/bwrap";
          MANTLE_TEST_SCRIPT_SHELL = "${pkgs.bash}/bin/bash";
          MANTLE_WASM_COMPONENT_TOOLCHAIN = "${wasmComponentToolchain}";
          CRUNCH_NO_FUSE = "1";
          MANTLE_TEST_OFFLINE = "1";
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

        checkNickelConfigs = pkgs.writeShellApplication {
          name = "check-nickel-configs";
          runtimeInputs = [
            pkgs.coreutils
            pkgs.diffutils
            pkgs.nickel
          ];
          text = ''
            set -eu
            profile_root="config/foreign-execution-profiles"
            audit_root="config/foreign-provenance-audit"
            cache_closure_root="config/foreign-cache-closure"
            scratch="$(mktemp -d)"
            trap 'rm -rf "$scratch"' EXIT

            nickel typecheck "$profile_root/guix.ncl"
            nickel typecheck "$profile_root/nix.ncl"
            nickel export --format json "$profile_root/guix.ncl" > "$scratch/guix.json"
            nickel export --format json "$profile_root/nix.ncl" > "$scratch/nix.json"
            diff -u "$profile_root/generated/guix.json" "$scratch/guix.json"
            diff -u "$profile_root/generated/nix.json" "$scratch/nix.json"

            nickel typecheck "$audit_root/default.ncl"
            nickel export --format json "$audit_root/default.ncl" > "$scratch/audit.json"
            diff -u "$audit_root/generated/default.json" "$scratch/audit.json"

            nickel typecheck "$cache_closure_root/default.ncl"
            nickel typecheck "$cache_closure_root/preserve-nix.ncl"
            nickel export --format json "$cache_closure_root/default.ncl" > "$scratch/cache-closure.json"
            nickel export --format json "$cache_closure_root/preserve-nix.ncl" > "$scratch/preserve-nix.json"
            diff -u "$cache_closure_root/generated/default.json" "$scratch/cache-closure.json"
            diff -u "$cache_closure_root/generated/preserve-nix.json" "$scratch/preserve-nix.json"

            if nickel export --format json "$cache_closure_root/tests/invalid-unknown-field.ncl" > /dev/null; then
              echo "unknown-field cache closure fixture unexpectedly passed" >&2
              exit 1
            fi
            if nickel export --format json "$cache_closure_root/tests/invalid-zero-limit.ncl" > /dev/null; then
              echo "zero-limit cache closure fixture unexpectedly passed" >&2
              exit 1
            fi
            if nickel export --format json "$cache_closure_root/tests/invalid-preserve-prefix.ncl" > /dev/null; then
              echo "preserved path prefix drift fixture unexpectedly passed" >&2
              exit 1
            fi

            if nickel export --format json "$audit_root/tests/invalid-unknown-field.ncl" > /dev/null; then
              echo "unknown-field provenance audit fixture unexpectedly passed" >&2
              exit 1
            fi
            if nickel export --format json "$audit_root/tests/invalid-zero-limit.ncl" > /dev/null; then
              echo "zero-limit provenance audit fixture unexpectedly passed" >&2
              exit 1
            fi

            if nickel export --format json "$profile_root/tests/invalid-unknown-field.ncl" > /dev/null; then
              echo "unknown-field execution profile fixture unexpectedly passed" >&2
              exit 1
            fi
            if nickel export --format json "$profile_root/tests/invalid-setid.ncl" > /dev/null; then
              echo "setid execution profile fixture unexpectedly passed" >&2
              exit 1
            fi
          '';
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
          SSL_CERT_FILE = caCertificateBundlePath;
          NIX_SSL_CERT_FILE = caCertificateBundlePath;
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
          SSL_CERT_FILE = caCertificateBundlePath;
          NIX_SSL_CERT_FILE = caCertificateBundlePath;
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
          SSL_CERT_FILE = caCertificateBundlePath;
          NIX_SSL_CERT_FILE = caCertificateBundlePath;
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

              "${rustToolchain}/bin/cargo" -q -Zscript scripts/check-gcc40-configure-bridge.rs --self-test
              bash scripts/check-bootstrap-blocker-inventory.sh \
                --self-test \
                --json "$TMPDIR/bootstrap-blocker-inventory.json" \
                --markdown "$TMPDIR/bootstrap-blocker-inventory.md"

              mkdir -p "$out"
              cp "$TMPDIR/bootstrap-blocker-inventory.json" "$out/current.json"
              cp "$TMPDIR/bootstrap-blocker-inventory.md" "$out/current.md"
            '';

        nickelExportCorePin =
          pkgs.runCommand "mantle-nickel-export-core-pin"
            {
              nativeBuildInputs = [
                pkgs.diffutils
                pkgs.jq
                pkgs.nickel
                rustToolchain
              ];
            }
            ''
              mkdir -p source/.cargo source/config/generated source/scripts
              cp ${./Cargo.toml} source/Cargo.toml
              cp ${./Cargo.lock} source/Cargo.lock
              cp ${./.cargo/vendor-config.toml} source/.cargo/vendor-config.toml
              cp ${./flake.nix} source/flake.nix
              cp ${./flake.lock} source/flake.lock
              cp ${./config/nickel-export-core-source.ncl} source/config/nickel-export-core-source.ncl
              cp ${./config/generated/nickel-export-core-source.json} source/config/generated/nickel-export-core-source.json
              cp ${./scripts/check-nickel-export-core-pin.rs} source/scripts/check-nickel-export-core-pin.rs

              export HOME="$TMPDIR/home"
              export CARGO_HOME="$TMPDIR/cargo-home"
              mkdir -p "$HOME" "$CARGO_HOME"

              cargo -Zscript source/scripts/check-nickel-export-core-pin.rs --root source
              cargo -Zscript source/scripts/check-nickel-export-core-pin.rs --self-test

              nickel export --format json source/config/nickel-export-core-source.ncl > "$TMPDIR/actual.json"
              jq --sort-keys . "$TMPDIR/actual.json" > "$TMPDIR/actual.sorted.json"
              jq --sort-keys . source/config/generated/nickel-export-core-source.json > "$TMPDIR/expected.sorted.json"
              diff -u "$TMPDIR/expected.sorted.json" "$TMPDIR/actual.sorted.json"

              test -f "${nickelExportCoreSource}/crates/nickel-export-core/Cargo.toml"
              mkdir -p "$out"
              cp "$TMPDIR/actual.sorted.json" "$out/source-pin.json"
            '';

        contentBoundRequirementEvidence =
          pkgs.runCommand "mantle-content-bound-requirement-evidence"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.diffutils
                pkgs.jq
                pkgs.nickel
              ];
              src = self;
            }
            ''
              set -eu
              cd "$src"

              receipt_ncl="fixtures/content-bound-requirements/integration-receipt.ncl"
              receipt_json="fixtures/content-bound-requirements/integration-receipt.json"
              nickel typecheck "$receipt_ncl"
              nickel export --format json "$receipt_ncl" > "$TMPDIR/actual.json"
              jq --sort-keys . "$TMPDIR/actual.json" > "$TMPDIR/actual.sorted.json"
              jq --sort-keys . "$receipt_json" > "$TMPDIR/expected.sorted.json"
              diff -u "$TMPDIR/expected.sorted.json" "$TMPDIR/actual.sorted.json"

              jq --raw-output '.fixtures[] | [.path, .blake3] | @tsv' "$receipt_json" |
                while IFS=$'\t' read -r path expected; do
                  actual="$(b3sum --no-names "$path")"
                  test "$actual" = "$expected"
                done

              mkdir -p "$out"
              cp "$TMPDIR/actual.sorted.json" "$out/integration-receipt.json"
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
          oci-distribution-registry = pkgs.distribution;
          kernelscript-compiler = kernelscriptExperiment.compiler;
          kernelscript-core-adapter = kernelscriptCoreAdapter;
          kernelscript-production = kernelscriptExperiment.artifacts;
          kernelscript-production-cohort = kernelscriptExperiment.cohort;
          kernelscript-production-shell = kernelscriptExperiment.productionShell;
          kernelscript-production-runtime-check = kernelscriptExperiment.runtimeCheck;
          spacewasm-reference-bundle = spacewasmReference.bundle;
          spacewasm-reference-bundler = spacewasmReference.bundler;
          spacewasm-reference-evidence = spacewasmReference.evidence;
          spacewasm-reference-rust-toolchain = spacewasmReference.toolchain;
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
          nickel-export-core-pin = nickelExportCorePin;
          content-bound-requirement-evidence = contentBoundRequirementEvidence;
          release-determinism-quality = releaseDeterminismQuality;
          release-nix-witness-quality = releaseNixWitnessQuality;

          # r[verify mantle.artifact_auth_adoption.radicle_transport]
          # r[verify mantle.artifact_auth_adoption.lock_agreement]
          # r[verify mantle.artifact_auth_adoption.fallback]
          # r[verify mantle.artifact_auth_adoption.radicle_evidence]
          artifact-auth-radicle-cutover =
            assert artifactAuthSourceAdmitted;
            pkgs.runCommand "mantle-artifact-auth-radicle-cutover"
              {
                nativeBuildInputs = [
                  pkgs.b3sum
                  pkgs.jq
                  pkgs.nickel
                  pkgs.nix
                  pkgs.ripgrep
                ];
                src = self;
              }
              ''
                set -eu
                cd "$src"

                nickel typecheck evidence/radicle/artifact-auth-cutover-v1.ncl
                nickel typecheck lib/artifact-auth-cutover-receipt.ncl
                nickel export --format json tests/artifact-auth-cutover.ncl > "$TMPDIR/tests.json"
                grep -Fq '"tests": true' "$TMPDIR/tests.json"

                nickel export --format json evidence/radicle/artifact-auth-cutover-v1.ncl > "$TMPDIR/cutover.json"
                jq -S . "$TMPDIR/cutover.json" > "$TMPDIR/cutover.normalized.json"
                jq -S . evidence/radicle/artifact-auth-cutover-v1.json > "$TMPDIR/evidence.normalized.json"
                cmp "$TMPDIR/cutover.normalized.json" "$TMPDIR/evidence.normalized.json"

                receipt_hash="$(b3sum evidence/radicle/artifact-auth-cutover-v1.json | cut -d ' ' -f 1)"
                expected_receipt_hash="$(tr -d '\n' < evidence/radicle/artifact-auth-cutover-v1.blake3)"
                test "$receipt_hash" = "$expected_receipt_hash"

                source_url='https://git.onix.computer/z4JGYYW7WsesXUq7MXVdx16Fawu2f.git'
                source_rev='799459346d5416fbd7b9f55840a7371441b55afa'
                source_nar_hash='sha256-nEgz2FtVuDesX95yyxidp0vhjxL4INB6Ve8rkpLyJk0='
                wrong_source_rev='1111111111111111111111111111111111111111'
                expected_flake_source="git+$source_url?rev=$source_rev"
                wrong_flake_source="git+$source_url?rev=$wrong_source_rev"
                expected_flake_input="url = \"$expected_flake_source\";"
                wrong_flake_input="url = \"$wrong_flake_source\";"
                export NIX_STATE_DIR="$TMPDIR/nix-state"
                export NIX_LOG_DIR="$TMPDIR/nix-log"
                mkdir -p "$NIX_STATE_DIR" "$NIX_LOG_DIR"

                # r[impl mantle.artifact_auth_adoption.live_validation_scope]
                scoped_flake_source() {
                  source_file="$1"
                  nix-instantiate --eval --strict --json \
                    --attr inputs.artifactAuthSource.url "$source_file" | jq -r .
                }

                # r[verify mantle.artifact_auth_adoption.live_validation_scope]
                test "$(scoped_flake_source flake.nix)" = "$expected_flake_source"

                cp flake.nix "$TMPDIR/unrelated-flake.nix"
                chmod u+w "$TMPDIR/unrelated-flake.nix"
                printf '\n# unrelated flake maintenance fixture\n' >> "$TMPDIR/unrelated-flake.nix"
                test "$(scoped_flake_source "$TMPDIR/unrelated-flake.nix")" = "$expected_flake_source"

                cp flake.nix "$TMPDIR/wrong-source-flake.nix"
                chmod u+w "$TMPDIR/wrong-source-flake.nix"
                substituteInPlace "$TMPDIR/wrong-source-flake.nix" \
                  --replace-fail "$expected_flake_input" "$wrong_flake_input"
                actual_wrong_source="$(scoped_flake_source "$TMPDIR/wrong-source-flake.nix")"
                test "$actual_wrong_source" = "$wrong_flake_source"
                if test "$actual_wrong_source" = "$expected_flake_source"; then
                  echo 'wrong artifact-auth flake revision passed scoped validation' >&2
                  exit 1
                fi

                for binding in \
                  'cargo.core_manifest_blake3:crates/crunch-action-result-core/Cargo.toml' \
                  'cargo.shell_manifest_blake3:crates/crunch-build/Cargo.toml' \
                  'cargo.lock_blake3:Cargo.lock' \
                  'nix.lock_blake3:flake.lock'; do
                  field="''${binding%%:*}"
                  path="''${binding#*:}"
                  expected="$(jq -r ".$field" evidence/radicle/artifact-auth-cutover-v1.json)"
                  actual="$(b3sum "$path" | cut -d ' ' -f 1)"
                  test "$actual" = "$expected"
                done

                jq -e \
                  --arg url "$source_url" \
                  --arg rev "$source_rev" \
                  --arg nar_hash "$source_nar_hash" \
                  '.nodes.artifactAuthSource as $source
                   | $source.locked.url == $url
                   and $source.original.url == $url
                   and $source.locked.rev == $rev
                   and $source.original.rev == $rev
                   and $source.locked.narHash == $nar_hash' \
                  flake.lock >/dev/null

                github_host='github.com'
                forbidden_source="$github_host/OnixResearch/artifact-auth"
                if rg -F "$forbidden_source" \
                  crates/crunch-action-result-core/Cargo.toml \
                  crates/crunch-build/Cargo.toml \
                  Cargo.lock flake.nix flake.lock; then
                  echo 'executable artifact-auth GitHub fallback remains' >&2
                  exit 1
                fi

                touch "$out"
              '';

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
            SNIX_BUILD_SANDBOX_SHELL = sandboxShellPath;
            GIT = "${pkgs.git}/bin/git";
            SSL_CERT_FILE = caCertificateBundlePath;
            NIX_SSL_CERT_FILE = caCertificateBundlePath;
            MANTLE_TEST_REAL_BWRAP = "${pkgs.bubblewrap}/bin/bwrap";
            MANTLE_TEST_SCRIPT_SHELL = "${pkgs.bash}/bin/bash";
            MANTLE_WASM_COMPONENT_TOOLCHAIN = "${wasmComponentToolchain}";
            CRUNCH_NO_FUSE = "1";
            MANTLE_TEST_OFFLINE = "1";
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
          spacewasm-reference-profile = spacewasmReference.profileExport;
          spacewasm-reference-host-library = spacewasmReference.hostLibrary;
          spacewasm-reference-wasm-library = spacewasmReference.wasmLibrary;
          spacewasm-reference-host-runner = spacewasmReference.hostRunner;
          spacewasm-reference-upstream-unit-tests = spacewasmReference.upstreamUnitTests;
          spacewasm-reference-spectest-address = spacewasmReference.upstreamSpectestAddress;
          spacewasm-reference-fixtures = spacewasmReference.fixtureReport;
          spacewasm-reference-negative = spacewasmReference.negativeCheck;
          spacewasm-reference-bundle = spacewasmReference.bundle;
        };

        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages =
            with pkgs;
            [
              cargo-deny
              cargo-nextest
              cargo-watch
              nickel
              rust-analyzer
            ]
            ++ [
              astGrepToolchain
              checkNickelConfigs
              wasmComponentToolchain
              tigerstyle.packages.${system}.cargo-tigerstyle
            ];

          MANTLE_WASM_COMPONENT_TOOLCHAIN = "${wasmComponentToolchain}";

          # Ensure the nightly toolchain is available
          inputsFrom = [ crunch ];
        };
      }
    );
}
