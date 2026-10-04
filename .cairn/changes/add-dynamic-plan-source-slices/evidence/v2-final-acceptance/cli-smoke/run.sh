#!/usr/bin/env bash
set -euo pipefail
I=/home/brittonr/.cargo-target/mantle-v2-cairn-dev46/source
R=/home/brittonr/.cargo-target/mantle-v2-integrated-final-yzLA5LVP
PROVEN=/home/brittonr/.cargo-target/mantle-v2-integrated-cli-PT7twP1R
BUSY=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
BWRAP=/nix/store/vvxfl9d1b9a944ca7miswhnxg06cs6jl-mantle-wasm-component-toolchain-v1/bin/bwrap
cd "$I"
printf '# Integrated committed source: %s\n# HEAD: 3dcdfaad7b5c77c0499ff1b4c1b3380c73f29e19\n# Fresh store/state: %s/{store,state}\n# Proven input fixture/scripts: %s\n' "$I" "$R" "$PROVEN" > "$R/commands.log"
record() {
  local name="$1" rc=0
  shift
  { printf '%s: ' "$name"; printf '%q ' "$@"; printf '\n'; } >> "$R/commands.log"
  "$@" > "$R/$name.stdout" 2> "$R/$name.stderr" || rc=$?
  printf '%s\n' "$rc" > "$R/$name.exit"
  printf '%s exit=%s\n' "$name" "$rc"
  return "$rc"
}
record_nix() {
  local name="$1"
  shift
  record "$name" nix develop --no-write-lock-file -c env HOME="$R/home" TMPDIR="$R/tmp" XDG_CACHE_HOME="$R/cache" CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-v2-cairn-dev46/first-party-quality-target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 SNIX_BUILD_SANDBOX_SHELL="$BUSY" SNIX_BUILD_BWRAP="$BWRAP" "$@"
}
record_nix build-cli cargo build --locked -q -p mantle --bin mantle
BIN=/home/brittonr/.cargo-target/mantle-v2-cairn-dev46/first-party-quality-target/debug/mantle
record_nix tool "$BIN" --store "$R/store" --state-dir "$R/state" --store-prefix /mantle/store --json build "$PROVEN/fixtures/tool.ncl" --no-substitute
record extract-tool jq -er '[.outcomes[] | select(.label=="v2-cli-tool") | .outputs[] | select(.name=="out") | .artifact_attestation.logical_path] | select(length==1) | .[0]' "$R/tool.stdout"
IFS= read -r TOOL_LOGICAL < "$R/extract-tool.stdout"
record verify-proven-tool-path test "$TOOL_LOGICAL" = /mantle/store/pzjzvwsvmia0bpfyhgr3zqcarapc7wcn-v2-cli-tool
record_nix one "$BIN" --store "$R/store" --state-dir "$R/state" --store-prefix /mantle/store --json build "$PROVEN/fixtures/producer-one.ncl" --no-substitute
record_nix two "$BIN" --store "$R/store" --state-dir "$R/state" --store-prefix /mantle/store --json build "$PROVEN/fixtures/producer-two.ncl" --no-substitute
record_nix absent "$BIN" --store "$R/store" --state-dir "$R/state" --store-prefix /mantle/store --json build "$PROVEN/fixtures/producer-absent.ncl" --no-substitute
record extract-slice jq -er '[.native_dynamic_plans[0].source_slices[] | select(.source_id=="src.main" and .disposition=="admitted") | .admitted_store_path] | select(length==1) | .[0]' "$R/one.stdout"
IFS= read -r SLICE < "$R/extract-slice.stdout"
record_nix fresh-store-info-basename "$BIN" --store "$R/store" --state-dir "$R/state" --store-prefix /mantle/store --json store info "${SLICE##*/}"
record assertions python3 "$PROVEN/assertions.py" "$R" "$SLICE"
record physical-proof python3 "$R/physical-proof.py" "$R"
