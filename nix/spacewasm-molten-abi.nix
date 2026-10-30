# r[impl mantle.spacewasm_pilot.fixtures]
{ pkgs }:
let
  common = ''
    (memory (export "memory") 1 1)
    (func (export "molten_alloc") (param i32) (result i32) i32.const 1024)
    (func (export "molten_dealloc") (param i32 i32))
  '';
  packInput = ''
    local.get 0 i64.extend_i32_u i64.const 32 i64.shl
    local.get 1 i64.extend_i32_u i64.or
  '';
  cases = {
    "molten-abi-roundtrip" = {
      body = "(module ${common} (func (export \"molten_run\") (param i32 i32) (result i64) ${packInput}))";
      expected = "canonical-input-output-equality";
    };
    "molten-abi-post-dealloc-state" = {
      body = ''(module
        (memory (export "memory") 1 1)
        (global $dealloc-count (mut i32) (i32.const 0))
        (func (export "molten_alloc") (param i32) (result i32) i32.const 1024)
        (func (export "molten_dealloc") (param i32 i32)
          global.get $dealloc-count i32.const 1 i32.add global.set $dealloc-count
          i32.const 0 global.get $dealloc-count i32.store8)
        (func (export "molten_run") (param i32 i32) (result i64) ${packInput}))'';
      expected = "canonical-input-output-equality;after-one-aliased-deallocation:private-counter=1,memory[0]=1";
    };
    "molten-abi-hostcall" = {
      body = ''(module
        (import "molten:hostcall" "assert" (func $assert (param i32 i32) (result i64)))
        ${common}
        (func (export "molten_run") (param i32 i32) (result i64)
          local.get 0 local.get 1 call $assert drop ${packInput}))'';
      expected = "declared-assert-hostcall-and-canonical-input-output-equality";
    };
    "molten-abi-invalid-pointer" = {
      body = "(module ${common} (func (export \"molten_run\") (param i32 i32) (result i64) i64.const -4294967295))";
      expected = "output-pointer-out-of-bounds";
    };
    "molten-abi-invalid-output" = {
      body = ''(module ${common} (data (i32.const 0) "\ff")
        (func (export "molten_run") (param i32 i32) (result i64) i64.const 1))'';
      expected = "invalid-canonical-preserves-output";
    };
    "molten-abi-memory-limit" = {
      body = ''(module (memory (export "memory") 2 2)
        (func (export "molten_alloc") (param i32) (result i32) i32.const 1024)
        (func (export "molten_dealloc") (param i32 i32))
        (func (export "molten_run") (param i32 i32) (result i64) ${packInput}))'';
      expected = "deny-when-memory-budget-is-one-page";
    };
    "molten-abi-stack-limit" = {
      body = ''(module ${common}
        (func $recurse (result i64) call $recurse i64.const 1 i64.add)
        (func (export "molten_run") (param i32 i32) (result i64) call $recurse))'';
      expected = "bounded-stack-or-cumulative-fuel-exhaustion";
    };
    "molten-abi-out-of-fuel" = {
      body = ''(module ${common}
        (func (export "molten_run") (param i32 i32) (result i64)
          (loop $spin br $spin) unreachable))'';
      expected = "cumulative-fuel-exhaustion";
    };
    "molten-abi-trap" = {
      body = "(module ${common} (func (export \"molten_run\") (param i32 i32) (result i64) unreachable))";
      expected = "unreachable-trap";
    };
  };
  materialize = name: case:
    let
      wat = pkgs.writeText "${name}.wat" case.body;
      descriptor = pkgs.writeText "${name}.json" (builtins.toJSON {
        schema = "mantle-molten-abi-fixture-v1";
        abi = "molten.wasm.abi.v1";
        fixture_id = name;
        source_revision = "e24cf09355a90497148eb5029fdb8e3400bd63e3";
        octet_profile = "spacewasm-molten-abi-pilot";
        wasm_feature_profile = "wasm1";
        input_pointer = 1024;
        max_fixture_input_bytes = 64512;
        memory_grow = false;
        expected = case.expected;
        descriptor_encoding = "high32-pointer-low32-length";
        actor_export = "molten_run";
        execution_status = "requires-consumer-owned-receipt";
      });
      wasm = pkgs.runCommand "${name}-wasm" {
        nativeBuildInputs = [ pkgs.wasm-tools ];
      } ''
        set -eu
        mkdir -p "$out"
        wasm-tools parse ${wat} -o "$out/${name}.wasm"
        wasm-tools validate --features=wasm1 "$out/${name}.wasm"
        wasm-tools print "$out/${name}.wasm" > "$TMPDIR/readback.wat"
        wasm-tools parse "$TMPDIR/readback.wat" -o "$TMPDIR/readback.wasm"
        cmp "$out/${name}.wasm" "$TMPDIR/readback.wasm"
      '';
    in [
      { source_path = toString wat; bundle_path = "fixtures/wat/${name}.wat"; role = "fixture-artifact"; }
      { source_path = "${wasm}/${name}.wasm"; bundle_path = "fixtures/wasm/${name}.wasm"; role = "fixture-artifact"; }
      { source_path = toString descriptor; bundle_path = "fixtures/descriptors/${name}.json"; role = "fixture-descriptor"; }
    ];
in
[
  {
    source_path = "${./spacewasm-molten-abi.nix}";
    bundle_path = "fixtures/authoring/spacewasm-molten-abi.nix";
    role = "fixture-descriptor";
  }
] ++ pkgs.lib.concatLists (pkgs.lib.mapAttrsToList materialize cases)
