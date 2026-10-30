# r[impl mantle.spacewasm_pilot.fixtures]
{ pkgs, packageRoot }:
let
  fixture = pkgs.runCommand "spacewasm-molten-host-segments" {
    nativeBuildInputs = [ pkgs.nickel pkgs.wasm-tools ];
  } ''
    set -eu
    mkdir -p "$out"
    wasm-tools parse ${packageRoot}/fixtures/wat/molten-host-segments.wat \
      -o "$out/molten-host-segments.wasm"
    wasm-tools validate --features=wasm1 "$out/molten-host-segments.wasm"
    nickel export --format json ${packageRoot}/fixtures/molten-host-segments.ncl \
      > "$out/molten-host-segments.json"
    # Read back from the binary, not the WAT or compiler's rendered log.
    wasm-tools print "$out/molten-host-segments.wasm" > "$TMPDIR/readback.wat"
    wasm-tools parse "$TMPDIR/readback.wat" -o "$TMPDIR/readback.wasm"
    cmp "$out/molten-host-segments.wasm" "$TMPDIR/readback.wasm"
  '';
in
[
  {
    source_path = "${packageRoot}/fixtures/wat/molten-host-segments.wat";
    bundle_path = "fixtures/wat/molten-host-segments.wat";
    role = "fixture-artifact";
  }
  {
    source_path = "${fixture}/molten-host-segments.wasm";
    bundle_path = "fixtures/wasm/molten-host-segments.wasm";
    role = "fixture-artifact";
  }
  {
    source_path = "${packageRoot}/fixtures/molten-host-segments.ncl";
    bundle_path = "fixtures/descriptors/molten-host-segments.ncl";
    role = "fixture-descriptor";
  }
  {
    source_path = "${fixture}/molten-host-segments.json";
    bundle_path = "fixtures/descriptors/molten-host-segments.json";
    role = "fixture-descriptor";
  }
]
