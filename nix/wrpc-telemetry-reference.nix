# r[impl materialize_wrpc_telemetry.producer]
{ pkgs, producer, packageRoot }:
let
  inherit (producer) toolchain mantle author sharedRoot wrpcTools;
  revision = "e5262a84044aff064c9599eebc05013bb3432795";
  archive = pkgs.fetchurl {
    url = "https://codeload.github.com/WebAssembly/wasi-otel/tar.gz/${revision}";
    hash = "sha256-ZSESQKRr7xZkrkeK53J/TOtc8hQWTJKKg0Aw2tWZHS8=";
  };
  vendor = pkgs.rustPlatform.importCargoLock {
    lockFile = packageRoot + "/guest/Cargo.lock";
    extraRegistries."https://github.com/rust-lang/crates.io-index" = "https://static.crates.io/crates";
  };
  python = pkgs.python3.withPackages (ps: [ ps.blake3 ]);
  source = pkgs.runCommand "mantle-wrpc-telemetry-source-${revision}" {
    nativeBuildInputs = [ toolchain python pkgs.gnutar pkgs.gzip pkgs.diffutils ];
  } ''
    set -eu
    mkdir -p "$out/.cargo" "$out/vendor" upstream "$TMPDIR/cache" "$TMPDIR/registry"
    tar -xzf ${archive} --strip-components=1 -C upstream
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
    # Both roles resolve these retained upstream bytes through wkg.toml overrides.
    for file in world.wit tracing.wit types.wit metrics.wit logs.wit; do
      cmp "$out/wit-deps/otel/$file" "upstream/wit/$file"
    done
    for dependency in wasi-clocks-0.2.0 wasi-io-0.2.0; do
      cmp "$out/wit-deps/$dependency/package.wit" "upstream/wit/deps/$dependency/package.wit"
    done
    cp upstream/LICENSE.md "$out/upstream-license.txt"
    cp ${packageRoot}/cohort.json "$out/cohort.json"
    ${python}/bin/python ${sharedRoot}/profile.py "$out" \
      ${toolchain}/share/mantle/wasm-component-toolchain.json telemetry
    cat > "$TMPDIR/wkg-config.toml" <<EOF
    default_registry = "local"
    [namespace_registries]
    "wasi" = "local"
    [registry."local"]
    default = "local"
    [registry."local".local]
    root = "$TMPDIR/registry"
    EOF
    cd "$out"
    for role in telemetry sink; do
      world=emitter
      if test "$role" = sink; then world=sink; fi
      wkg wit fetch --config "$TMPDIR/wkg-config.toml" --cache "$TMPDIR/cache" \
        --wit-dir "$out/$role/wit" --type wasm
      cmp "$out/$role/wit/deps/wasi-otel-0.2.0-rc.2/tracing.wit" "$out/wit-deps/otel/tracing.wit"
      # Dependency-free bindings: guest_rt.rs supplies the emitted runtime path.
      wit-bindgen rust --world "$world" --generate-all --disable-run-ctors-once-workaround \
        --disable-custom-section-link-helpers --runtime-path crate::guest_rt::rt \
        --bitflags-path crate::guest_rt::rt::bitflags --out-dir "$out/$role/src" "$out/$role/wit"
    done
    test -s wkg.lock
    # Every WIT dependency is a local override; the registry stays empty.
    test -z "$(ls -A "$TMPDIR/registry")"
  '';
  compiled = pkgs.runCommand "mantle-wrpc-telemetry-components-${revision}" {
    nativeBuildInputs = [ toolchain wrpcTools pkgs.b3sum pkgs.jq ];
  } ''
    set -eu
    cp -R ${source} source
    chmod -R u+w source
    export CARGO_HOME="$TMPDIR/cargo-home"
    export CARGO_TARGET_DIR="$TMPDIR/target"
    export CARGO_NET_OFFLINE=true
    export RUSTC="${toolchain}/bin/rustc"
    mkdir -p "$CARGO_HOME" "$out/components" "$out/wit" "$out/bindings"
    cd source
    cargo build --locked --offline --release --target wasm32-wasip2
    for role in telemetry sink; do
      world=emitter
      if test "$role" = sink; then world=sink; fi
      component="$CARGO_TARGET_DIR/wasm32-wasip2/release/mantle_wrpc_$role.wasm"
      wasm-tools validate "$component"
      cp "$component" "$out/components/$role.wasm"
      wasm-tools component wit "$component" > "$out/wit/$role.wit"
      mkdir -p "$out/bindings/$role/component" "$out/bindings/$role/wrpc"
      wit-bindgen rust --world "$world" --generate-all --out-dir "$out/bindings/$role/component" "$role/wit"
      wit-bindgen-wrpc rust --world "$world" --generate-all --out-dir "$out/bindings/$role/wrpc" "$role/wit"
      jq -n --sort-keys --arg role "$role" --arg revision "${revision}" \
        --arg generator "$(wit-bindgen-wrpc --version)" \
        --arg generatorDigest "$(b3sum --no-names ${wrpcTools}/bin/wit-bindgen-wrpc)" \
        --arg witDigest "$(b3sum --no-names "$role/wit/world.wit")" \
        --arg componentDigest "$(b3sum --no-names "$component")" \
        '{schema:"mantle-wrpc-telemetry-binding-generation-v1", role:$role, source_revision:$revision, generator:$generator, generator_blake3:$generatorDigest, wit_blake3:$witDigest, component_blake3:$componentDigest}' \
        > "$out/bindings/$role/generation.json"
    done
  '';
  bundle = pkgs.runCommand "mantle-wrpc-telemetry-reference-bundle-${revision}" {
    nativeBuildInputs = [ toolchain author mantle pkgs.jq python ];
  } ''
    set -eu
    # Mantle materializes its embedded Nickel stdlib under a writable cache root.
    export HOME="$TMPDIR/home" XDG_CACHE_HOME="$TMPDIR/cache"
    mkdir -p "$HOME" "$XDG_CACHE_HOME"
    mkdir -p "$out/source" "$out/comparison" "$out/registry"
    cp ${archive} "$out/source/wasi-otel-${revision}.tar.gz"
    cp ${packageRoot}/cohort.json "$out/cohort.json"
    cp ${packageRoot}/cases.json "$out/cases.json"
    cp ${source}/upstream-license.txt "$out/source/upstream-license.txt"
    cp ${sharedRoot}/cohort.json "$out/source/runtime-cohort.json"
    cp -R ${packageRoot}/guest "$out/source/authoring"
    cp ${sharedRoot}/profile.py ${sharedRoot}/manifest.py "$out/source/"
    cp -R ${compiled}/bindings ${compiled}/wit "$out/"
    cp ${compiled}/components/sink.wasm "$out/comparison/sink.wasm"
    mantle-wrpc-author telemetry ${toolchain} ${source} "$out/registry" \
      ${compiled}/components/telemetry.wasm "$out/comparison/sink.wasm" > "$TMPDIR/telemetry.ncl"
    mantle --json --store "$TMPDIR/store" --state-dir "$TMPDIR/state" \
      wasm-component build "$TMPDIR/telemetry.ncl" --out "$out/telemetry" \
      --scratch-parent "$out" > "$TMPDIR/execution.json" \
      || { jq -c "{final_status, blockers}" "$out/telemetry/execution-report.json" >&2; exit 1; }
    jq -e '.final_status == "succeeded" and (.blockers | length == 0)' "$TMPDIR/execution.json"
    cp "$TMPDIR/telemetry.ncl" "$out/telemetry/request.ncl"
    cp ${source}/telemetry/runtime-profile.json "$out/telemetry/runtime-profile.json"
    cp ${toolchain}/share/mantle/wasm-component-toolchain.json "$out/toolchain.json"
    mirror="$out/capability/$out"
    mkdir -p "$mirror"
    cp -R "$out/telemetry" "$mirror/"
    ${python}/bin/python ${sharedRoot}/manifest.py "$out" telemetry
  '';
in {
  inherit source compiled bundle;
}
