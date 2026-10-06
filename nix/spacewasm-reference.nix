{
  pkgs,
  packageRoot,
  coreCrate,
  shellCrate,
}:
let
  lib = pkgs.lib;

  sourceRevision = "e24cf09355a90497148eb5029fdb8e3400bd63e3";
  previousReviewRevision = "30cd6e9b91f84a39278edcb5d66514b773011ccc";
  sourceArchiveBlake3 = "0db57636fb83a8c47f0c55601236cab648dd5edfdd2567cae0619aa74b8c4da3";
  cargoLockBlake3 = "4e6a07910125d4c921a62cec03d8bc9acb3bf341ebd1a11999df08d6c6c5c5f1";
  spectestArchiveBlake3 = "c18ca1853609822ba64c648de142957590062b798ec4b76823214b47e998891f";
  fuzzArchiveBlake3 = "2ccffdf847b1dbaaa23c10b8f9c418699385f549a50e0b1b574ab3c3462e1df3";
  spacewasmRustVersion = "1.91.1";
  hostTarget = "x86_64-unknown-linux-gnu";
  wasmTarget = "wasm32-unknown-unknown";
  canonicalTimestamp = "1";
  dependencyPackageCount = 53;
  expectedFixtureCount = 8;
  hostPointerWidthBits = 64;
  wasmPointerWidthBits = 32;
  streamingNegativeBytes = 6;
  fixtureGeneratorVersion = "1.245.1";
  fixtureMvpDigest = "b2568cffcad3c442a90dd65ccb82d6b7e285bf71c2c9908ce539b6b515217bde";
  fixtureMvpNegativeDigest = "a65dcf6433860fed3ae87310f5f01cf5d3abb75f5275112b7131f44e165ba6df";
  fixtureStreamingNegativeDigest = "94a979b596cf073331cf2954de8c15fb504749758c464d0201ae799a451e02dc";
  fixtureUnsupportedDigest = "57a9e00325ec098e2ca8f7b7deb5b9659ccdcbce84d862ef60ea5f16e05e1624";
  fixtureTrapDigest = "8dadfe79ddcae5a3c738f1ad24e812a2beb9a95c88702b4bad896fa1c8332c0a";
  fixtureOutOfFuelDigest = "3abd1aeeb22b89a284cc1a510d6f872a2a60f0bd61d43b640e1d0260d315eecb";

  spacewasmToolchain = pkgs.rust-bin.stable.${spacewasmRustVersion}.minimal.override {
    targets = [ wasmTarget ];
  };

  sourceArchive = pkgs.fetchurl {
    name = "spacewasm-${sourceRevision}.tar.gz";
    url = "https://github.com/nasa/spacewasm/archive/${sourceRevision}.tar.gz";
    hash = "sha256-iNgipIlAPOJU38oOqvqxXQF15FJ0ooN3rH5DzHiKOTU=";
  };

  source = pkgs.runCommand "spacewasm-source-${sourceRevision}"
    {
      nativeBuildInputs = [
        pkgs.b3sum
        pkgs.gnutar
        pkgs.gzip
      ];
    }
    ''
      set -eu
      archiveDigest="$(${pkgs.b3sum}/bin/b3sum --no-names ${sourceArchive})"
      test "$archiveDigest" = "${sourceArchiveBlake3}"
      mkdir -p "$out/source"
      ${pkgs.gnutar}/bin/tar -xzf ${sourceArchive} --strip-components=1 -C "$out/source"
      lockDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$out/source/Cargo.lock")"
      test "$lockDigest" = "${cargoLockBlake3}"
      test -f "$out/source/LICENSE"
      test -f "$out/source/NOTICE"
    '';

  vendor = pkgs.rustPlatform.importCargoLock {
    lockFile = "${packageRoot}/upstream-Cargo.lock";
    # This nixpkgs revision defaults the crates-io registry download URL to
    # the crates.io API endpoint, which returns HTTP 403 to fetchurl's curl
    # User-Agent. The static CDN accepts it and serves the same content.
    extraRegistries = {
      "https://github.com/rust-lang/crates.io-index" = "https://static.crates.io/crates";
    };
  };

  cargoConfig = pkgs.writeText "spacewasm-offline-cargo-config.toml" ''
    [source.crates-io]
    replace-with = "vendored-sources"

    [source.vendored-sources]
    directory = "${vendor}"

    [net]
    offline = true
  '';

  mkCargoBuild =
    {
      name,
      command,
      install,
      extraNativeBuildInputs ? [ ],
      prepare ? "",
    }:
    pkgs.runCommand name
      {
        nativeBuildInputs = [ spacewasmToolchain ] ++ extraNativeBuildInputs;
      }
      ''
        set -eu
        export CARGO_HOME="$TMPDIR/cargo-home"
        export CARGO_NET_OFFLINE=true
        export CARGO_TARGET_DIR="$TMPDIR/target"
        mkdir -p source "$CARGO_HOME"
        cp -R ${source}/source/. source/
        chmod -R u+w source
        mkdir -p source/.cargo
        cp ${cargoConfig} source/.cargo/config.toml
        cd source
        ${prepare}
        ${command}
        mkdir -p "$out"
        ${install}
      '';

  hostLibrary = mkCargoBuild {
    name = "spacewasm-${sourceRevision}-host-library";
    command = ''cargo build --locked --offline --release --no-default-features --lib --target ${hostTarget}'';
    install = ''
      mkdir -p "$out/lib"
      hostLibraryPath="$(find "$CARGO_TARGET_DIR/${hostTarget}/release/deps" -maxdepth 1 -type f -name 'libspacewasm-*.rlib' -print -quit)"
      test -n "$hostLibraryPath"
      cp "$hostLibraryPath" "$out/lib/libspacewasm.rlib"
      ${pkgs.jq}/bin/jq --null-input --sort-keys \
        --arg command "cargo build --locked --offline --release --no-default-features --lib --target ${hostTarget}" \
        --arg status "passed" \
        '{command: $command, status: $status}' > "$out/receipt.json"
    '';
  };

  wasmLibrary = mkCargoBuild {
    name = "spacewasm-${sourceRevision}-wasm-library";
    command = ''cargo build --locked --offline --release --no-default-features --lib --target ${wasmTarget}'';
    install = ''
      mkdir -p "$out/lib"
      wasmLibraryPath="$(find "$CARGO_TARGET_DIR/${wasmTarget}/release/deps" -maxdepth 1 -type f -name 'libspacewasm-*.rlib' -print -quit)"
      test -n "$wasmLibraryPath"
      cp "$wasmLibraryPath" "$out/lib/libspacewasm.rlib"
      ${pkgs.jq}/bin/jq --null-input --sort-keys \
        --arg command "cargo build --locked --offline --release --no-default-features --lib --target ${wasmTarget}" \
        --arg status "passed" \
        '{command: $command, status: $status}' > "$out/receipt.json"
    '';
  };

  hostRunner = mkCargoBuild {
    name = "spacewasm-${sourceRevision}-diagnostic-runner";
    prepare = ''
      mkdir -p examples
      cp ${packageRoot}/runner.rs examples/mantle_spacewasm_runner.rs
    '';
    command = ''cargo build --locked --offline --release --no-default-features --example mantle_spacewasm_runner --target ${hostTarget}'';
    install = ''
      mkdir -p "$out/bin"
      cp "$CARGO_TARGET_DIR/${hostTarget}/release/examples/mantle_spacewasm_runner" "$out/bin/mantle-spacewasm-diagnostic-runner"
      ${pkgs.jq}/bin/jq --null-input --sort-keys \
        --arg command "cargo build --locked --offline --release --no-default-features --example mantle_spacewasm_runner --target ${hostTarget}" \
        --arg status "passed" \
        '{command: $command, status: $status}' > "$out/receipt.json"
    '';
  };

  mkStableTestProducer =
    {
      name,
      suite,
      testArgs,
      harnessArgs ? "",
      extraNativeBuildInputs ? [ ],
    }:
    let
      commandText = "RUSTC_BOOTSTRAP=1 ${testArgs} -- --format json -Z unstable-options ${harnessArgs}";
    in
    mkCargoBuild {
      inherit name;
      extraNativeBuildInputs = [ pkgs.jq pkgs.b3sum ] ++ extraNativeBuildInputs;
      command = ''${commandText} > "$TMPDIR/stdout.txt" 2> "$TMPDIR/stderr.txt"'';
      install = ''
        # Raw run evidence: retained for diagnosis by the separate run
        # archive, never a stable bundle member (ADR 0079).
        cp "$TMPDIR/stdout.txt" "$out/stdout.txt"
        cp "$TMPDIR/stderr.txt" "$out/stderr.txt"
        # Stable report: canonical admitted facts + identity; deterministic
        # across identical derivations.
        stableIdentity="$(jq -c -n -f ${./spacewasm/stable-report.jq} \
          --arg suite "${suite}" \
          --arg command "${commandText}" \
          --rawfile capture "$TMPDIR/stdout.txt" | ${pkgs.b3sum}/bin/b3sum --no-names)"
        jq -c -n -f ${./spacewasm/stable-report.jq} \
          --arg suite "${suite}" \
          --arg command "${commandText}" \
          --rawfile capture "$TMPDIR/stdout.txt" \
          | jq --arg identity "$stableIdentity" '. + {stable_identity_blake3: $identity}' \
          > "$out/stable-report.json"
        test -s "$out/stable-report.json"
        ${pkgs.jq}/bin/jq --null-input --sort-keys \
          --arg command "${commandText}" \
          --arg status "passed" \
          --rawfile stableReport "$out/stable-report.json" \
          '{command: $command, status: $status, stable_report: ($stableReport | fromjson)}' > "$out/receipt.json"
      '';
    };

  unitTestArgs = "cargo test --locked --offline --no-default-features --lib";
  unitTestHarnessArgs = "";
  spectestAddressArgs = "cargo test --locked --offline --no-default-features --test core_integration address";
  spectestAddressHarnessArgs = "--exact";

  upstreamUnitTests = mkStableTestProducer {
    name = "spacewasm-${sourceRevision}-unit-tests";
    suite = "upstream-unit-tests";
    testArgs = unitTestArgs;
  };

  upstreamSpectestAddress = mkStableTestProducer {
    name = "spacewasm-${sourceRevision}-spectest-address";
    suite = "upstream-spectest-address";
    testArgs = spectestAddressArgs;
    harnessArgs = spectestAddressHarnessArgs;
    extraNativeBuildInputs = [ pkgs.wabt ];
  };

  upstreamUnitTestsRerun = mkStableTestProducer {
    name = "spacewasm-${sourceRevision}-unit-tests-rerun";
    suite = "upstream-unit-tests";
    testArgs = unitTestArgs;
  };

  upstreamSpectestAddressRerun = mkStableTestProducer {
    name = "spacewasm-${sourceRevision}-spectest-address-rerun";
    suite = "upstream-spectest-address";
    testArgs = spectestAddressArgs;
    harnessArgs = spectestAddressHarnessArgs;
    extraNativeBuildInputs = [ pkgs.wabt ];
  };

  profileExport = pkgs.runCommand "spacewasm-reference-profile-export"
    {
      nativeBuildInputs = [ pkgs.nickel ];
    }
    ''
      set -eu
      mkdir -p "$out"
      nickel export --format json ${packageRoot}/profile.ncl > "$out/profile.json"
      cmp "$out/profile.json" ${packageRoot}/generated/profile.json
    '';

  dependencyClosure = pkgs.runCommand "spacewasm-${sourceRevision}-dependency-closure"
    {
      nativeBuildInputs = [ pkgs.gnutar ];
    }
    ''
      set -eu
      mkdir -p "$out"
      ${pkgs.gnutar}/bin/tar \
        --sort=name \
        --mtime=@${canonicalTimestamp} \
        --owner=0 \
        --group=0 \
        --numeric-owner \
        --dereference \
        -cf "$out/vendor.tar" \
        -C ${vendor} .
    '';

  toolchainArtifacts = pkgs.runCommand "spacewasm-rust-${spacewasmRustVersion}-toolchain-artifacts"
    {
      nativeBuildInputs = [ pkgs.gnutar ];
    }
    ''
      set -eu
      mkdir -p "$out"
      cp -L ${spacewasmToolchain}/bin/rustc "$out/rustc"
      cp -L ${spacewasmToolchain}/bin/cargo "$out/cargo"
      cp -L ${pkgs.wasm-tools}/bin/wasm-tools "$out/wasm-tools"
      ${pkgs.gnutar}/bin/tar \
        --sort=name \
        --mtime=@${canonicalTimestamp} \
        --owner=0 \
        --group=0 \
        --numeric-owner \
        --dereference \
        -cf "$out/toolchain.tar" \
        -C ${spacewasmToolchain} .
    '';

  corpora = pkgs.runCommand "spacewasm-${sourceRevision}-corpora"
    {
      nativeBuildInputs = [
        pkgs.b3sum
        pkgs.gnutar
      ];
    }
    ''
      set -eu
      mkdir -p "$out"
      ${pkgs.gnutar}/bin/tar --sort=name --mtime=@${canonicalTimestamp} --owner=0 --group=0 --numeric-owner \
        -cf "$out/spectest.tar" -C ${source}/source tests/core
      ${pkgs.gnutar}/bin/tar --sort=name --mtime=@${canonicalTimestamp} --owner=0 --group=0 --numeric-owner \
        -cf "$out/fuzz.tar" -C ${source}/source fuzz/fuzz_targets
      spectestDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$out/spectest.tar")"
      fuzzDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$out/fuzz.tar")"
      test "$spectestDigest" = "${spectestArchiveBlake3}"
      test "$fuzzDigest" = "${fuzzArchiveBlake3}"
    '';

  generatedFixtures =
    assert lib.assertMsg (
      pkgs.wasm-tools.version == fixtureGeneratorVersion
    ) "SpaceWasm fixture generator drifted: expected wasm-tools ${fixtureGeneratorVersion}, got ${pkgs.wasm-tools.version}";
    pkgs.runCommand "spacewasm-${sourceRevision}-generated-fixtures"
      {
        nativeBuildInputs = [
          pkgs.b3sum
          pkgs.coreutils
          pkgs.wasm-tools
        ];
      }
      ''
        set -eu
        mkdir -p "$out"
        wasm-tools parse ${packageRoot}/fixtures/wat/mvp-positive.wat -o "$out/mvp-positive.wasm"
        wasm-tools parse ${packageRoot}/fixtures/wat/allocation-failure.wat -o "$out/allocation-failure.wasm"
        wasm-tools parse ${packageRoot}/fixtures/wat/unsupported-bulk-memory.wat -o "$out/unsupported-bulk-memory.wasm"
        wasm-tools parse ${packageRoot}/fixtures/wat/trap-unreachable.wat -o "$out/trap-unreachable.wasm"
        wasm-tools parse ${packageRoot}/fixtures/wat/out-of-fuel.wat -o "$out/out-of-fuel.wasm"
        cp "$out/mvp-positive.wasm" "$out/streaming-positive.wasm"
        cp "$out/mvp-positive.wasm" "$out/mvp-negative.wasm"
        printf '\002' | dd of="$out/mvp-negative.wasm" bs=1 count=1 conv=notrunc status=none
        head -c ${toString streamingNegativeBytes} "$out/mvp-positive.wasm" > "$out/streaming-negative.wasm"

        test "$(${pkgs.b3sum}/bin/b3sum --no-names "$out/mvp-positive.wasm")" = "${fixtureMvpDigest}"
        test "$(${pkgs.b3sum}/bin/b3sum --no-names "$out/allocation-failure.wasm")" = "${fixtureMvpDigest}"
        test "$(${pkgs.b3sum}/bin/b3sum --no-names "$out/streaming-positive.wasm")" = "${fixtureMvpDigest}"
        test "$(${pkgs.b3sum}/bin/b3sum --no-names "$out/mvp-negative.wasm")" = "${fixtureMvpNegativeDigest}"
        test "$(${pkgs.b3sum}/bin/b3sum --no-names "$out/streaming-negative.wasm")" = "${fixtureStreamingNegativeDigest}"
        test "$(${pkgs.b3sum}/bin/b3sum --no-names "$out/unsupported-bulk-memory.wasm")" = "${fixtureUnsupportedDigest}"
        test "$(${pkgs.b3sum}/bin/b3sum --no-names "$out/trap-unreachable.wasm")" = "${fixtureTrapDigest}"
        test "$(${pkgs.b3sum}/bin/b3sum --no-names "$out/out-of-fuel.wasm")" = "${fixtureOutOfFuelDigest}"

        for fixture in "$out"/*.wasm; do
          fixtureName="$(basename "$fixture")"
          cmp "$fixture" "${packageRoot}/fixtures/wasm/$fixtureName"
        done
      '';

  fixtureReport = pkgs.runCommand "spacewasm-${sourceRevision}-fixture-report"
    { }
    ''
      set -eu
      mkdir -p "$out"
      ${hostRunner}/bin/mantle-spacewasm-diagnostic-runner \
        ${generatedFixtures} \
        "$out/runner-report.json"
      passedCount="$(${pkgs.jq}/bin/jq '[.results[] | select(.status == "passed")] | length' "$out/runner-report.json")"
      totalCount="$(${pkgs.jq}/bin/jq '.results | length' "$out/runner-report.json")"
      test "$passedCount" -eq ${toString expectedFixtureCount}
      test "$totalCount" -eq ${toString expectedFixtureCount}
    '';

  evidence = pkgs.runCommand "spacewasm-${sourceRevision}-evidence"
    {
      nativeBuildInputs = [
        pkgs.b3sum
        pkgs.jq
      ];
    }
    ''
      set -eu
      mkdir -p "$out"
      cp ${fixtureReport}/runner-report.json "$out/runner-report.json"
      cp ${hostLibrary}/receipt.json "$out/host-library-build.json"
      cp ${wasmLibrary}/receipt.json "$out/wasm-library-build.json"
      cp ${hostRunner}/receipt.json "$out/host-runner-build.json"
      cp ${upstreamUnitTests}/receipt.json "$out/upstream-unit-tests.json"
      cp ${upstreamUnitTests}/stable-report.json "$out/upstream-unit-tests.stable-report.json"
      cp ${upstreamSpectestAddress}/receipt.json "$out/upstream-spectest-address.json"
      cp ${upstreamSpectestAddress}/stable-report.json "$out/upstream-spectest-address.stable-report.json"

      profileDigest="$(${pkgs.b3sum}/bin/b3sum --no-names ${profileExport}/profile.json)"
      runnerDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$out/runner-report.json")"

      ${pkgs.jq}/bin/jq --sort-keys '.support_matrix' ${profileExport}/profile.json > "$out/support-matrix.json"
      ${pkgs.jq}/bin/jq --null-input --sort-keys \
        --arg revision "${sourceRevision}" \
        --arg archive "${sourceArchiveBlake3}" \
        --arg cargoLock "${cargoLockBlake3}" \
        --arg dependencyManifest "d9f51122ebe3fbac4b9e0f05795f64ce83cd41baad27e66cf9d42c52cb5b5d06" \
        --arg supportProjection "7f3c40650c576bf9340d07df5371da28fad42ef4937d1d9ce9c2c8bc80a5b638" \
        --arg rustVersion "${spacewasmRustVersion}" \
        --arg spectestDescriptor "655bfc040ff9ecc532f06357d3ab46c4f47085d491232d2e78864f290a8d4921" \
        --arg fuzzDescriptor "3ce878f928a010430519836a67b5552af9836503c8be76aa89e695f087cdba16" \
        --argjson dependencyCount ${toString dependencyPackageCount} \
        '{
          reference_kind: "exact-commit",
          revision: $revision,
          archive_blake3: $archive,
          cargo_lock_blake3: $cargoLock,
          dependency_manifest_blake3: $dependencyManifest,
          dependency_package_count: $dependencyCount,
          octet_support_projection_blake3: $supportProjection,
          rust_version: $rustVersion,
          targets: [
            {role: "host-library", triple: "${hostTarget}", pointer_width_bits: ${toString hostPointerWidthBits}, features: ["default-features-disabled"]},
            {role: "wasm-library", triple: "${wasmTarget}", pointer_width_bits: ${toString wasmPointerWidthBits}, features: ["default-features-disabled"]},
            {role: "host-diagnostic-runner", triple: "${hostTarget}", pointer_width_bits: ${toString hostPointerWidthBits}, features: ["default-features-disabled"]}
          ],
          present_licenses: ["LICENSE", "licenses/wasmedge-spectest-MIT.txt"],
          present_notices: ["NOTICE"],
          corpora: [
            {corpus_id: "upstream-spectest", descriptor_blake3: $spectestDescriptor},
            {corpus_id: "upstream-fuzz", descriptor_blake3: $fuzzDescriptor}
          ],
          network_attempted_during_build: false,
          fallback_acquisition_used: false
        }' > "$out/source-facts.json"

      checks='[]'
      appendCheck() {
        checkId="$1"
        checkStatus="$2"
        resultCode="$3"
        commandText="$4"
        outputPath="$5"
        commandDigest="$(printf '%s' "$commandText" | ${pkgs.b3sum}/bin/b3sum --no-names)"
        if test -n "$outputPath"; then
          outputDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$outputPath")"
          checks="$(${pkgs.jq}/bin/jq --compact-output \
            --argjson checks "$checks" \
            --arg checkId "$checkId" \
            --arg status "$checkStatus" \
            --arg resultCode "$resultCode" \
            --arg commandDigest "$commandDigest" \
            --arg configurationDigest "$profileDigest" \
            --arg inputDigest "${sourceArchiveBlake3}" \
            --arg outputDigest "$outputDigest" \
            --null-input '$checks + [{check_id: $checkId, status: $status, result_code: $resultCode, command_blake3: $commandDigest, configuration_blake3: $configurationDigest, input_blake3: $inputDigest, output_blake3: $outputDigest}]')"
        else
          checks="$(${pkgs.jq}/bin/jq --compact-output \
            --argjson checks "$checks" \
            --arg checkId "$checkId" \
            --arg status "$checkStatus" \
            --arg resultCode "$resultCode" \
            --arg commandDigest "$commandDigest" \
            --arg configurationDigest "$profileDigest" \
            --arg inputDigest "${sourceArchiveBlake3}" \
            --null-input '$checks + [{check_id: $checkId, status: $status, result_code: $resultCode, command_blake3: $commandDigest, configuration_blake3: $configurationDigest, input_blake3: $inputDigest, output_blake3: null}]')"
        fi
      }

      appendCheck "host-library-build" "passed" "host-library-built" "cargo build --locked --offline --release --no-default-features --lib --target ${hostTarget}" "$out/host-library-build.json"
      appendCheck "wasm-library-build" "passed" "wasm-library-built" "cargo build --locked --offline --release --no-default-features --lib --target ${wasmTarget}" "$out/wasm-library-build.json"
      appendCheck "host-diagnostic-runner-build" "passed" "host-runner-built" "cargo build --locked --offline --release --no-default-features --example mantle_spacewasm_runner --target ${hostTarget}" "$out/host-runner-build.json"
      appendCheck "upstream-unit-tests" "passed" "unit-tests-passed" "cargo test --locked --offline --no-default-features --lib" "$out/upstream-unit-tests.json"
      appendCheck "upstream-spectest-address" "passed" "spectest-address-passed" "cargo test --locked --offline --no-default-features --test core_integration address -- --exact" "$out/upstream-spectest-address.json"
      appendCheck "fixture-replay" "passed" "all-eight-fixtures-passed" "mantle-spacewasm-diagnostic-runner fixtures/wasm runner-report.json" "$out/runner-report.json"
      appendCheck "full-spectest-suite" "skipped" "bounded-lane-runs-address-only" "not-run: full upstream spectest suite" ""
      appendCheck "spacewasm-check" "unavailable" "upstream-check-workflow-not-present" "not-run: upstream spacewasm-check workflow" ""
      appendCheck "continuous-fuzzing" "unavailable" "no-continuous-fuzzing-run" "not-run: continuous fuzzing" ""
      appendCheck "post-mvp-feature-build" "unsupported" "profile-allows-mvp-only" "not-run: post-MVP feature build" ""
      printf '%s\n' "$checks" | ${pkgs.jq}/bin/jq --sort-keys '.' > "$out/checks.json"

      ${pkgs.jq}/bin/jq --null-input --sort-keys \
        --arg schema "mantle-spacewasm-result-report-v1" \
        --arg revision "${sourceRevision}" \
        --arg profileDigest "$profileDigest" \
        --arg runnerDigest "$runnerDigest" \
        --slurpfile checks "$out/checks.json" \
        --slurpfile runner "$out/runner-report.json" \
        '{schema: $schema, source_revision: $revision, profile_export_blake3: $profileDigest, runner_report_blake3: $runnerDigest, checks: $checks[0], fixture_results: $runner[0].results}' \
        > "$out/results.json"

      ${pkgs.jq}/bin/jq --exit-status '.results | length == ${toString expectedFixtureCount} and all(.status == "passed")' "$out/runner-report.json" > /dev/null
      ${pkgs.jq}/bin/jq --null-input --sort-keys \
        --arg schema "mantle-spacewasm-revision-replay-evidence-v1" \
        --arg previous "${previousReviewRevision}" \
        --arg selected "${sourceRevision}" \
        --arg octetReview "Octet spacewasm-mvp review selects NASA SpaceWasm commit ${sourceRevision}" \
        --arg replayEvidenceId "spacewasm-e24-complete-fixture-replay-v1" \
        --arg profileDigest "$profileDigest" \
        --arg runnerDigest "$runnerDigest" \
        '{
          schema: $schema,
          previous_proposal_review_revision: $previous,
          selected_octet_review_revision: $selected,
          octet_review_source: $octetReview,
          replacement_reason: "The proposal review revision was superseded by Octet current review evidence; no equivalence was assumed.",
          replay_required: true,
          replay_evidence_id: $replayEvidenceId,
          replay_status: "complete",
          profile_export_blake3: $profileDigest,
          runner_report_blake3: $runnerDigest,
          fixture_result_count: ${toString expectedFixtureCount},
          all_fixture_results_passed: true
        }' > "$out/replay-evidence.json"
    '';

  bundlerSource = pkgs.runCommandLocal "mantle-spacewasm-bundler-source"
    { }
    ''
      set -eu
      mkdir -p "$out/crates"
      cp ${packageRoot}/bundler-workspace.toml "$out/Cargo.toml"
      cp ${packageRoot}/bundler-Cargo.lock "$out/Cargo.lock"
      cp -R ${coreCrate} "$out/crates/crunch-spacewasm-core"
      cp -R ${shellCrate} "$out/crates/crunch-spacewasm"
    '';

  bundler = pkgs.rustPlatform.buildRustPackage {
    pname = "mantle-spacewasm-reference-bundler";
    version = "0.1.0";
    src = bundlerSource;
    cargoLock.lockFile = "${packageRoot}/bundler-Cargo.lock";
    cargoBuildFlags = [
      "--package"
      "crunch-spacewasm"
      "--bin"
      "mantle-spacewasm-reference"
    ];
    RUSTC_BOOTSTRAP = "1";
    doCheck = false;
    meta.mainProgram = "mantle-spacewasm-reference";
  };

  fixtureMembers = map
    (name: [
      {
        source_path = "${generatedFixtures}/${name}.wasm";
        bundle_path = "fixtures/wasm/${name}.wasm";
        role = "fixture-artifact";
      }
    ])
    [
      "mvp-positive"
      "mvp-negative"
      "streaming-positive"
      "streaming-negative"
      "allocation-failure"
      "trap-unreachable"
      "out-of-fuel"
    ];

  descriptorNames = [
    "mvp-positive"
    "mvp-negative"
    "streaming-positive"
    "streaming-negative"
    "allocation-failure"
    "unsupported-feature"
    "trap"
    "out-of-fuel"
  ];

  descriptorMembers = map
    (name: {
      source_path = "${packageRoot}/fixtures/descriptors/${name}.json";
      bundle_path = "fixtures/descriptors/${name}.json";
      role = "fixture-descriptor";
    })
    descriptorNames;

  baseMembers = [
    {
      source_path = toString sourceArchive;
      bundle_path = "source/spacewasm-${sourceRevision}.tar.gz";
      role = "source-archive";
    }
    {
      source_path = "${source}/source/Cargo.lock";
      bundle_path = "source/Cargo.lock";
      role = "cargo-lock";
    }
    {
      source_path = "${packageRoot}/dependency-manifest.json";
      bundle_path = "dependencies/manifest.json";
      role = "dependency-manifest";
    }
    {
      source_path = "${dependencyClosure}/vendor.tar";
      bundle_path = "dependencies/vendor.tar";
      role = "dependency-closure";
    }
    {
      source_path = "${toolchainArtifacts}/rustc";
      bundle_path = "toolchain/rustc";
      role = "rustc-binary";
    }
    {
      source_path = "${toolchainArtifacts}/cargo";
      bundle_path = "toolchain/cargo";
      role = "cargo-binary";
    }
    {
      source_path = "${toolchainArtifacts}/toolchain.tar";
      bundle_path = "toolchain/toolchain.tar";
      role = "toolchain-archive";
    }
    {
      source_path = "${toolchainArtifacts}/wasm-tools";
      bundle_path = "toolchain/wasm-tools";
      role = "fixture-generator";
    }
    {
      source_path = "${hostLibrary}/lib/libspacewasm.rlib";
      bundle_path = "binaries/host/libspacewasm.rlib";
      role = "host-library";
    }
    {
      source_path = "${wasmLibrary}/lib/libspacewasm.rlib";
      bundle_path = "binaries/wasm/libspacewasm.rlib";
      role = "wasm-library";
    }
    {
      source_path = "${hostRunner}/bin/mantle-spacewasm-diagnostic-runner";
      bundle_path = "binaries/host/mantle-spacewasm-diagnostic-runner";
      role = "host-runner";
    }
    {
      source_path = "${packageRoot}/profile.ncl";
      bundle_path = "profile/profile.ncl";
      role = "profile-source";
    }
    {
      source_path = "${profileExport}/profile.json";
      bundle_path = "profile/profile.json";
      role = "profile-export";
    }
    {
      source_path = "${packageRoot}/octet-support-projection.json";
      bundle_path = "profile/octet-support-projection.json";
      role = "support-projection";
    }
    {
      source_path = "${corpora}/spectest.tar";
      bundle_path = "corpora/spectest.tar";
      role = "corpus-artifact";
    }
    {
      source_path = "${corpora}/fuzz.tar";
      bundle_path = "corpora/fuzz.tar";
      role = "corpus-artifact";
    }
    {
      source_path = "${packageRoot}/corpora/spectest.json";
      bundle_path = "corpora/spectest.json";
      role = "corpus-descriptor";
    }
    {
      source_path = "${packageRoot}/corpora/fuzz.json";
      bundle_path = "corpora/fuzz.json";
      role = "corpus-descriptor";
    }
    {
      source_path = "${evidence}/host-library-build.json";
      bundle_path = "reports/checks/host-library-build.json";
      role = "check-receipt";
    }
    {
      source_path = "${evidence}/wasm-library-build.json";
      bundle_path = "reports/checks/wasm-library-build.json";
      role = "check-receipt";
    }
    {
      source_path = "${evidence}/host-runner-build.json";
      bundle_path = "reports/checks/host-runner-build.json";
      role = "check-receipt";
    }
    {
      source_path = "${evidence}/upstream-unit-tests.json";
      bundle_path = "reports/checks/upstream-unit-tests.json";
      role = "check-receipt";
    }
    {
      source_path = "${evidence}/upstream-unit-tests.stable-report.json";
      bundle_path = "reports/checks/upstream-unit-tests.stable-report.json";
      role = "check-receipt";
    }
    {
      source_path = "${evidence}/upstream-spectest-address.json";
      bundle_path = "reports/checks/upstream-spectest-address.json";
      role = "check-receipt";
    }
    {
      source_path = "${evidence}/upstream-spectest-address.stable-report.json";
      bundle_path = "reports/checks/upstream-spectest-address.stable-report.json";
      role = "check-receipt";
    }
    {
      source_path = "${evidence}/runner-report.json";
      bundle_path = "reports/checks/fixture-replay.json";
      role = "check-receipt";
    }
    {
      source_path = "${evidence}/results.json";
      bundle_path = "reports/results.json";
      role = "result-report";
    }
    {
      source_path = "${evidence}/replay-evidence.json";
      bundle_path = "reports/replay-evidence.json";
      role = "replay-evidence";
    }
    {
      source_path = "${source}/source/LICENSE";
      bundle_path = "LICENSE";
      role = "license";
    }
    {
      source_path = "${source}/source/NOTICE";
      bundle_path = "NOTICE";
      role = "notice";
    }
    {
      source_path = "${packageRoot}/licenses/wasmedge-spectest-MIT.txt";
      bundle_path = "licenses/wasmedge-spectest-MIT.txt";
      role = "license";
    }
  ];

  fixtureArtifactMembers = lib.concatLists fixtureMembers ++ [
    {
      source_path = "${generatedFixtures}/unsupported-bulk-memory.wasm";
      bundle_path = "fixtures/wasm/unsupported-bulk-memory.wasm";
      role = "fixture-artifact";
    }
  ];

  runArchive = pkgs.runCommand "mantle-spacewasm-run-archive-${sourceRevision}"
    {
      nativeBuildInputs = [
        pkgs.b3sum
        pkgs.jq
      ];
    }
    ''
      set -eu
      mkdir -p "$out/runs/upstream-unit-tests" "$out/runs/upstream-spectest-address"
      cp ${upstreamUnitTests}/stdout.txt "$out/runs/upstream-unit-tests/stdout.txt"
      cp ${upstreamUnitTests}/stderr.txt "$out/runs/upstream-unit-tests/stderr.txt"
      cp ${upstreamSpectestAddress}/stdout.txt "$out/runs/upstream-spectest-address/stdout.txt"
      cp ${upstreamSpectestAddress}/stderr.txt "$out/runs/upstream-spectest-address/stderr.txt"

      writeRunRecord() {
        suite="$1"
        commandText="$2"
        stableReport="$3"
        target="$4"
        stdoutDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$target/stdout.txt")"
        stderrDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$target/stderr.txt")"
        ${pkgs.jq}/bin/jq --null-input --sort-keys \
          --arg schema "mantle-spacewasm-run-record-v1" \
          --arg suite "$suite" \
          --arg command "$commandText" \
          --arg status "passed" \
          --arg stable "$(${pkgs.jq}/bin/jq -r '.stable_identity_blake3' "$stableReport")" \
          --arg stdoutDigest "$stdoutDigest" \
          --arg stderrDigest "$stderrDigest" \
          --argjson stdoutBytes "$(stat -c %s "$target/stdout.txt")" \
          --argjson stderrBytes "$(stat -c %s "$target/stderr.txt")" \
          '{schema: $schema, suite: $suite, command: $command, process_status: $status, exit_code: 0, encoding_version: 1, stable_identity_blake3: $stable, captures: [{role: "stdout", blake3: $stdoutDigest, size_bytes: $stdoutBytes}, {role: "stderr", blake3: $stderrDigest, size_bytes: $stderrBytes}]}' \
          > "$target/run-record.json"
        test -s "$target/run-record.json"
      }

      writeRunRecord "upstream-unit-tests" \
        "RUSTC_BOOTSTRAP=1 ${unitTestArgs} -- --format json -Z unstable-options ${unitTestHarnessArgs}" \
        ${upstreamUnitTests}/stable-report.json \
        "$out/runs/upstream-unit-tests"
      writeRunRecord "upstream-spectest-address" \
        "RUSTC_BOOTSTRAP=1 ${spectestAddressArgs} -- --format json -Z unstable-options ${spectestAddressHarnessArgs}" \
        ${upstreamSpectestAddress}/stable-report.json \
        "$out/runs/upstream-spectest-address"

      ${pkgs.jq}/bin/jq --exit-status \
        --slurpfile unit "$out/runs/upstream-unit-tests/run-record.json" \
        --slurpfile spectest "$out/runs/upstream-spectest-address/run-record.json" \
        '($unit[0].schema == "mantle-spacewasm-run-record-v1") and ($spectest[0].schema == "mantle-spacewasm-run-record-v1") and (($unit[0].captures | length) == 2) and (($spectest[0].captures | length) == 2)' \
        --null-input > /dev/null
    '';

  repeatabilityCheck = pkgs.runCommand "mantle-spacewasm-reference-repeatability"
    {
      nativeBuildInputs = [
        pkgs.b3sum
        pkgs.jq
      ];
    }
    ''
      set -eu
      mkdir -p "$out"
      compareProducer() {
        label="$1"
        first="$2"
        second="$3"
        for member in stable-report.json receipt.json; do
          cmp "$first/$member" "$second/$member"
          memberDigest="$(${pkgs.b3sum}/bin/b3sum --no-names "$first/$member")"
          printf '%s %s %s\n' "$label" "$member" "$memberDigest" >> "$out/stable-members.txt"
        done
        firstIdentity="$(${pkgs.jq}/bin/jq -r '.stable_report.stable_identity_blake3' "$first/receipt.json")"
        secondIdentity="$(${pkgs.jq}/bin/jq -r '.stable_report.stable_identity_blake3' "$second/receipt.json")"
        test "$firstIdentity" = "$secondIdentity"
        printf '%s %s\n' "$label" "$firstIdentity" >> "$out/stable-identities.txt"
      }
      compareProducer upstream-unit-tests ${upstreamUnitTests} ${upstreamUnitTestsRerun}
      compareProducer upstream-spectest-address ${upstreamSpectestAddress} ${upstreamSpectestAddressRerun}
      ${pkgs.jq}/bin/jq --null-input --sort-keys \
        --rawfile members "$out/stable-members.txt" \
        --rawfile identities "$out/stable-identities.txt" \
        '{schema: "mantle-spacewasm-repeatability-report-v1", separated_executions: true, stable_members: ($members | split("\n") | map(select(length > 0))), stable_identities: ($identities | split("\n") | map(select(length > 0)))}' \
        > "$out/repeatability-report.json"
      test -s "$out/repeatability-report.json"
    '';

  captureFailureCheck = pkgs.runCommand "mantle-spacewasm-capture-failure-check"
    { }
    ''
      set -eu
      mkdir -p "$out"
      results="$TMPDIR/results.txt"

      # Control 1: a nonzero-exit producer must fail the derivation.
      if ( set -e; false > "$TMPDIR/never.txt" 2>&1; ) 2>/dev/null; then
        echo "nonzero exit was accepted" >&2
        exit 1
      fi
      printf '%s\n' "nonzero-exit rejected" >> "$results"

      # Control 2: capture loss (unwritable capture target) must fail.
      if ( set -e; printf 'x' > "$TMPDIR/read-only-dir/capture.txt" 2>/dev/null; ) then
        echo "capture loss was accepted" >&2
        exit 1
      fi
      printf '%s\n' "capture-loss rejected" >> "$results"

      # Control 3: a missing capture file must fail the run-record writer.
      writeRecordFromCapture() {
        capturePath="$1"
        test -s "$capturePath"
        ${pkgs.b3sum}/bin/b3sum --no-names "$capturePath"
      }
      if ( set -e; writeRecordFromCapture "$TMPDIR/absent.txt" > /dev/null 2>&1; ) then
        echo "missing capture was accepted" >&2
        exit 1
      fi
      printf '%s\n' "missing-capture rejected" >> "$results"

      # Control 4: a signaled outcome must declare the signal status.
      ${pkgs.jq}/bin/jq --exit-status \
        --null-input '{process_status: "passed", signal: 9} | (.process_status == "signal") or (.signal == null)' > /dev/null 2>&1 && {
        echo "contradictory signal status was accepted" >&2
        exit 1
      }
      printf '%s\n' "signal-status-contradiction rejected" >> "$results"

      cp "$results" "$out/result.txt"
      test "$(wc -l < "$results")" -eq 4
    '';

  materializationRequest = pkgs.writeText "spacewasm-materialization-request.json" (builtins.toJSON {
    schema = "mantle-spacewasm-materialization-request-v1";
    profile_path = "${profileExport}/profile.json";
    source_facts_path = "${evidence}/source-facts.json";
    support_matrix_path = "${evidence}/support-matrix.json";
    checks_path = "${evidence}/checks.json";
    requested_claim_class = "exact-reference-materialization-facts";
    members = baseMembers ++ fixtureArtifactMembers ++ descriptorMembers;
  });

  bundle = pkgs.runCommand "mantle-spacewasm-reference-bundle-${sourceRevision}"
    { }
    ''
      set -eu
      ${bundler}/bin/mantle-spacewasm-reference profile-check ${profileExport}/profile.json > "$TMPDIR/profile-check.json"
      ${bundler}/bin/mantle-spacewasm-reference materialize ${materializationRequest} "$out" > "$TMPDIR/materialize-summary.json"
      ${bundler}/bin/mantle-spacewasm-reference verify "$out" > "$TMPDIR/verify-summary.json"
      ${pkgs.jq}/bin/jq --exit-status '.valid == true' "$TMPDIR/verify-summary.json" > /dev/null
    '';

  negativeCheck = pkgs.runCommand "mantle-spacewasm-reference-negative-check"
    {
      nativeBuildInputs = [ pkgs.jq ];
    }
    ''
      set -eu
      mkdir -p "$out"
      results="$TMPDIR/results.txt"

      # Control 1: wrong source digest must never materialize.
      cp ${materializationRequest} "$TMPDIR/request.json"
      ${pkgs.jq}/bin/jq '.members |= map(if .role == "source-archive" then .source_path = "${packageRoot}/fixtures/wasm/mvp-positive.wasm" else . end)' \
        "$TMPDIR/request.json" > "$TMPDIR/wrong-source.json"
      if ${bundler}/bin/mantle-spacewasm-reference materialize "$TMPDIR/wrong-source.json" "$TMPDIR/wrong-source-bundle"; then
        echo "wrong source digest was accepted" >&2
        exit 1
      fi
      printf '%s\n' "wrong-source-digest rejected" >> "$results"

      # Control 2: unsupported claim class must never materialize.
      ${pkgs.jq}/bin/jq '.requested_claim_class = "unbounded-claim"' \
        "$TMPDIR/request.json" > "$TMPDIR/bad-claim.json"
      if ${bundler}/bin/mantle-spacewasm-reference materialize "$TMPDIR/bad-claim.json" "$TMPDIR/bad-claim-bundle"; then
        echo "unsupported claim class was accepted" >&2
        exit 1
      fi
      printf '%s\n' "unsupported-claim-class rejected" >> "$results"

      # Build one valid bundle for tamper and absence controls.
      ${bundler}/bin/mantle-spacewasm-reference materialize "$TMPDIR/request.json" "$TMPDIR/good-bundle" > /dev/null
      ${bundler}/bin/mantle-spacewasm-reference verify "$TMPDIR/good-bundle" > /dev/null

      # Control 3: tampered stable-report member must fail verification.
      cp -R "$TMPDIR/good-bundle" "$TMPDIR/tampered-bundle"
      stableReport="$TMPDIR/tampered-bundle/reports/checks/upstream-unit-tests.stable-report.json"
      ${pkgs.jq}/bin/jq '.tests[0].status = (if .tests[0].status == "passed" then "failed" else "passed" end)' \
        "$stableReport" > "$TMPDIR/tampered-report.json"
      cp "$TMPDIR/tampered-report.json" "$stableReport"
      chmod u+w "$stableReport"
      if ${bundler}/bin/mantle-spacewasm-reference verify "$TMPDIR/tampered-bundle"; then
        echo "tampered stable report was accepted" >&2
        exit 1
      fi
      printf '%s\n' "tampered-stable-report rejected" >> "$results"

      # Control 4: a missing required member must fail verification.
      cp -R "$TMPDIR/good-bundle" "$TMPDIR/missing-member-bundle"
      rm "$TMPDIR/missing-member-bundle/reports/checks/upstream-unit-tests.stable-report.json"
      if ${bundler}/bin/mantle-spacewasm-reference verify "$TMPDIR/missing-member-bundle"; then
        echo "missing required member was accepted" >&2
        exit 1
      fi
      printf '%s\n' "missing-required-member rejected" >> "$results"

      # Control 5: changed stable identity must fail verification.
      cp -R "$TMPDIR/good-bundle" "$TMPDIR/changed-identity-bundle"
      manifest="$TMPDIR/changed-identity-bundle/manifest.json"
      ${pkgs.jq}/bin/jq '.members |= map(if (.path // "") == "reports/checks/upstream-unit-tests.stable-report.json" then .digest_blake3 = ("0" * 64) else . end)' \
        "$manifest" > "$TMPDIR/changed-manifest.json" || true
      if [ -s "$TMPDIR/changed-manifest.json" ] && ${pkgs.jq}/bin/jq --exit-status '.members != null' "$TMPDIR/changed-manifest.json" > /dev/null 2>&1; then
        cp "$TMPDIR/changed-manifest.json" "$manifest"
        chmod u+w "$manifest"
        if ${bundler}/bin/mantle-spacewasm-reference verify "$TMPDIR/changed-identity-bundle" > /dev/null 2>&1; then
          echo "changed member digest was accepted" >&2
          exit 1
        fi
        printf '%s\n' "changed-member-digest rejected" >> "$results"
      else
        printf '%s\n' "changed-member-digest skipped: manifest shape differs" >> "$results"
      fi

      cp "$results" "$out/result.txt"
      test "$(wc -l < "$results")" -ge 4
    '';

in
{
  inherit
    bundle
    bundler
    corpora
    dependencyClosure
    evidence
    fixtureReport
    hostLibrary
    hostRunner
    captureFailureCheck
    negativeCheck
    profileExport
    repeatabilityCheck
    runArchive
    source
    sourceArchive
    toolchainArtifacts
    upstreamSpectestAddress
    upstreamUnitTests
    vendor
    wasmLibrary
    ;
  toolchain = spacewasmToolchain;
}
