# r[impl mantle.spacewasm_pilot.boundary]
{ pkgs, bundle, bundler }:
pkgs.runCommand "spacewasm-pilot-fixture-negative-check" {
  nativeBuildInputs = [ pkgs.coreutils pkgs.jq ];
} ''
  set -eu
  verify=${bundler}/bin/mantle-spacewasm-reference
  "$verify" verify ${bundle} > "$TMPDIR/positive.json"
  jq -e '.valid == true' "$TMPDIR/positive.json" > /dev/null
  for name in molten-host-segments molten-abi-roundtrip molten-abi-post-dealloc-state molten-abi-hostcall \
    molten-abi-invalid-pointer molten-abi-invalid-output molten-abi-memory-limit \
    molten-abi-stack-limit molten-abi-out-of-fuel molten-abi-trap; do
    test -s "${bundle}/fixtures/wasm/$name.wasm"
    test -s "${bundle}/fixtures/wat/$name.wat"
    test -s "${bundle}/fixtures/descriptors/$name.json"
  done
  mkdir -p "$out"
  for control in missing mismatch substitution; do
    target="$TMPDIR/$control"
    cp -r ${bundle} "$target"
    chmod -R u+w "$target"
    artifact="$target/fixtures/wasm/molten-abi-hostcall.wasm"
    case "$control" in
      missing) rm "$artifact" ;;
      mismatch) printf '\000' >> "$artifact" ;;
      substitution) cp "$target/fixtures/wasm/mvp-positive.wasm" "$artifact" ;;
    esac
    if "$verify" verify "$target" > "$TMPDIR/$control.json"; then
      echo "pilot fixture $control was accepted" >&2
      exit 1
    fi
    printf '%s rejected\n' "$control" >> "$out/results.txt"
  done
  # Byte-identical local recompilation cannot establish the producer identity.
  # That admission check belongs to the consumer's pinned publication contract,
  # not this byte-integrity verifier. Do not fabricate a local-build verdict.
''
