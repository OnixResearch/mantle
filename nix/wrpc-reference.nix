# r[impl materialize_wrpc_producer.component_pair]
# r[impl materialize_wrpc_producer.consumer_contract]
{ pkgs, toolchain, mantle, author, packageRoot }:
let
  revision = "aa01409ed38bf7a978570ff567c4475376925df6";
  archive = pkgs.fetchurl {
    url = "https://codeload.github.com/bytecodealliance/wrpc/tar.gz/${revision}";
    hash = "sha256-C1MFw8/ninRC5oqZc/Ff0Ymu2dZNdrrLSo3DfAUP+JI=";
  };
  upstream = pkgs.fetchzip {
    url = "https://codeload.github.com/bytecodealliance/wrpc/tar.gz/${revision}";
    extension = "tar.gz";
    hash = "sha256-atDMHeLJ72NRyglNDhaqB7cD+KyBbI0QDbRGGgEkxjg=";
  };
  vendor = pkgs.rustPlatform.importCargoLock {
    lockFile = packageRoot + "/guest/Cargo.lock";
    extraRegistries."https://github.com/rust-lang/crates.io-index" = "https://static.crates.io/crates";
  };
  wrpcTools = pkgs.rustPlatform.buildRustPackage {
    pname = "mantle-wrpc-bindgen";
    version = "0.15.0-${revision}";
    src = upstream;
    cargoLock.lockFile = "${upstream}/Cargo.lock";
    cargoBuildFlags = [ "--package" "wrpc" "--no-default-features" "--features" "bin-bindgen" "--bin" "wit-bindgen-wrpc" ];
    doCheck = false;
  };
  python = pkgs.python3.withPackages (ps: [ ps.blake3 ]);
  source = pkgs.runCommand "mantle-wrpc-authoring-${revision}" {
    nativeBuildInputs = [ toolchain pkgs.diffutils pkgs.jq python ];
  } ''
    set -eu
    mkdir -p "$out/.cargo" "$out/vendor" "$TMPDIR/cache" "$TMPDIR/registry"
    cp -R ${packageRoot}/guest/. "$out/"
    chmod -R u+w "$out"
    cp -RL ${vendor}/. "$out/vendor/"
    cat > "$out/.cargo/config.toml" <<EOF
    [source.crates-io]
    replace-with = "vendored-sources"
    [source.vendored-sources]
    directory = "vendor"
    [net]
    offline = true
    EOF
    # The original unversioned interface is retained byte-for-byte and is the
    # only source the checked-in wkg.toml override admits for both roles.
    cmp "$out/wit-deps/hello/hello.wit" \
      ${upstream}/examples/rust/hello-component-server/wit/deps/hello/hello.wit
    ${python}/bin/python ${packageRoot}/profile.py "$out" ${toolchain}/share/mantle/wasm-component-toolchain.json
    cp ${packageRoot}/cohort.json "$out/cohort.json"
    ${python}/bin/python ${packageRoot}/cohort.py ${upstream}/Cargo.lock ${packageRoot}/cohort.json \
      > "$out/upstream-cohort.json"
    cat > "$TMPDIR/wkg-config.toml" <<EOF
    default_registry = "local"
    [namespace_registries]
    "wrpc-examples" = "local"
    [registry."local"]
    default = "local"
    [registry."local".local]
    root = "$TMPDIR/registry"
    EOF
    cd "$out"
    for role in client server; do
      wkg wit fetch --config "$TMPDIR/wkg-config.toml" --cache "$TMPDIR/cache" \
        --wit-dir "$out/$role/wit" --type wasm
      cmp "$out/$role/wit/deps/wrpc-examples-hello/hello.wit" "$out/wit-deps/hello/hello.wit"
      # The pinned toolchain CLI emits dependency-free bindings into the source
      # closure, so offline guest builds need no binding macro or host linker.
      wit-bindgen rust --world "$role" --generate-all --disable-run-ctors-once-workaround \
        --disable-custom-section-link-helpers --out-dir "$out/$role/src" "$out/$role/wit"
    done
    test -f wkg.lock
    # Every WIT dependency is a local override; the registry stays empty.
    test -z "$(ls -A "$TMPDIR/registry")"
  '';
  compiled = pkgs.runCommand "mantle-wrpc-components-${revision}" {
    nativeBuildInputs = [ toolchain wrpcTools pkgs.b3sum pkgs.jq ];
  } ''
    set -eu
    cp -R ${source} source
    chmod -R u+w source
    export CARGO_HOME="$TMPDIR/cargo-home"
    export CARGO_TARGET_DIR="$TMPDIR/target"
    export CARGO_NET_OFFLINE=true
    export RUSTC="${toolchain}/bin/rustc"
    mkdir -p "$CARGO_HOME" "$out/components" "$out/bindings" "$out/wit"
    cd source
    cargo build --locked --offline --release --target wasm32-wasip2
    for role in client server; do
      component="$CARGO_TARGET_DIR/wasm32-wasip2/release/mantle_wrpc_$role.wasm"
      wasm-tools validate "$component"
      cp "$component" "$out/components/$role.wasm"
      wasm-tools component wit "$component" > "$out/wit/$role.wit"
      mkdir -p "$out/bindings/$role/component" "$out/bindings/$role/wrpc"
      wit-bindgen rust --world "$role" --generate-all --out-dir "$out/bindings/$role/component" "$role/wit"
      wit-bindgen-wrpc rust --world "$role" --generate-all --out-dir "$out/bindings/$role/wrpc" "$role/wit"
      jq -n --sort-keys --arg role "$role" --arg revision "${revision}" \
        --arg generator "$(wit-bindgen-wrpc --version)" \
        --arg generatorDigest "$(b3sum --no-names ${wrpcTools}/bin/wit-bindgen-wrpc)" \
        --arg witDigest "$(b3sum --no-names "$role/wit/world.wit")" \
        --arg componentDigest "$(b3sum --no-names "$component")" \
        '{schema:"mantle-wrpc-binding-generation-v1", role:$role, source_revision:$revision, generator:$generator, generator_blake3:$generatorDigest, wit_blake3:$witDigest, component_blake3:$componentDigest, component_bindings:("pinned toolchain wit-bindgen CLI output " + $role + "/src/" + $role + ".rs retained in the source closure"), wrpc_command:("wit-bindgen-wrpc rust --world " + $role + " --generate-all --out-dir bindings/" + $role + "/wrpc " + $role + "/wit")}' \
        > "$out/bindings/$role/generation.json"
    done
  '';
  bundle = pkgs.runCommand "mantle-wrpc-reference-bundle-${revision}" {
    nativeBuildInputs = [ toolchain author mantle pkgs.jq python ];
  } ''
    set -eu
    # Mantle materializes its embedded Nickel stdlib under a writable cache root.
    export HOME="$TMPDIR/home" XDG_CACHE_HOME="$TMPDIR/cache"
    mkdir -p "$HOME" "$XDG_CACHE_HOME"
    mkdir -p "$out/source" "$out/bindings" "$out/profiles" "$out/registry"
    cp ${archive} "$out/source/wrpc-${revision}.tar.gz"
    cp ${upstream}/Cargo.lock "$out/source/upstream-Cargo.lock"
    cp ${upstream}/LICENSE "$out/source/LICENSE"
    cp ${packageRoot}/cohort.json "$out/cohort.json"
    cp ${source}/upstream-cohort.json "$out/source/upstream-cohort.json"
    cp ${source}/client/runtime-profile.json "$out/profiles/client.json"
    cp ${source}/server/runtime-profile.json "$out/profiles/server.json"
    cp -R ${compiled}/bindings/. "$out/bindings/"
    cp -R ${compiled}/wit "$out/wit"
    cp -R ${packageRoot}/guest "$out/source/authoring"
    cp ${packageRoot}/profile.py ${packageRoot}/manifest.py ${packageRoot}/cohort.py "$out/source/"
    for role in server client; do
      server="-"
      if test "$role" = client; then
        server="$(jq -r '.final_portable.logical_path' "$out/server/materialization-bundle.json")"
      fi
      mantle-wrpc-author "$role" ${toolchain} ${source} "$out/registry" \
        ${compiled}/components/"$role.wasm" "$server" > "$TMPDIR/$role.ncl"
      mantle --json --store "$TMPDIR/store" --state-dir "$TMPDIR/state" \
        wasm-component build "$TMPDIR/$role.ncl" --out "$out/$role" \
        --scratch-parent "$out" > "$TMPDIR/$role-execution.json" \
        || { jq -c "{final_status, blockers}" "$out/$role/execution-report.json" >&2; exit 1; }
      jq -e '.final_status == "succeeded" and (.blockers | length == 0)' "$TMPDIR/$role-execution.json"
      cp "$TMPDIR/$role.ncl" "$out/$role/request.ncl"
      cp ${source}/"$role/runtime-profile.json" "$out/$role/runtime-profile.json"
    done
    cp ${toolchain}/share/mantle/wasm-component-toolchain.json "$out/toolchain.json"
    # Preserve the real CLI's logical paths under one narrow capability root.
    mirror="$out/capability/$out"
    mkdir -p "$mirror"
    cp -R "$out/client" "$out/server" "$mirror/"
    ${python}/bin/python ${packageRoot}/manifest.py "$out"
  '';
in {
  inherit source compiled bundle wrpcTools;
}
