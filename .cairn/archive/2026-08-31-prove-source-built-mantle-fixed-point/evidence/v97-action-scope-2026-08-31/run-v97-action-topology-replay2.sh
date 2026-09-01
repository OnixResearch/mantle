#!/bin/sh
set -eu
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
ROOT="$RUN/.source-built-fixed-point-v97-action-scope-20260831.source-built-fixed-point-staging-3483179"
BIN=/tmp/mantle-v97-diagnostic
SOURCE="$ROOT/inputs/mantle-source"
RUSTC="$ROOT/cargo-free-fixed-point/toolchain/rustc-receipt-bound-runtime"
GUARD="$ROOT/cargo-free-fixed-point/stage1/cargo-guard-bin"
CARGO="$ROOT/cargo-free-fixed-point/stage1/cargo-forbidden"
EXECUTION="$ROOT/cargo-free-fixed-point/execution"
AUTHORITY="$ROOT/cargo-free-fixed-point/stage1/rust-child-actions/authority.json"
EVIDENCE=/tmp/v97-action-replay-evidence2
REPORT="$RUN/v97-action-topology-replay2.json"
STDERR="$RUN/v97-action-topology-replay2.stderr"
STATUS="$RUN/v97-action-topology-replay2.status"
POLICY=6b8131ea6ab5ddfebbb13ff1003727a074680fe8cd7ba91a8209353af45eb5c5
ROUTE='{"role":"c-compiler","name":"cc","execution_path":"/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v97-action-scope-20260831.source-built-fixed-point-staging-3483179/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/x86_64-linux-musl-gcc","content_digest_blake3":"d7a9976841679e85b6dfeabbef802e0fb2f78730d3c6e0e95b590184113256af","source":{"kind":"local-tree","name":"mantle-source-built-native-root","digest_blake3":"281615af3158122f084e4839cce8955ca2cbc9759e5d8def9bf4387a26fe4475"},"build_receipt":{"kind":"external-attested-build","name":"mantle-source-built-native-root-receipt","digest_blake3":"281615af3158122f084e4839cce8955ca2cbc9759e5d8def9bf4387a26fe4475"},"compiler_family":"gcc"}'
for required in "$BIN" "$SOURCE" "$RUSTC" "$GUARD" "$CARGO" "$EXECUTION" "$AUTHORITY"; do test -e "$required"; done
for absent in "$EVIDENCE" "$REPORT" "$STDERR" "$STATUS"; do test ! -e "$absent"; done
export CARGO
export RUSTC_BOOTSTRAP=1
export CRUNCH_NO_FUSE=1
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export PATH="$GUARD"
export MANTLE_SOURCE_BUILT_C_COMPILER_ROUTE="$ROUTE"
export MANTLE_RUST_TOOLCHAIN_POLICY_DIGEST_BLAKE3="$POLICY"
cd "$SOURCE"
set +e
"$BIN" --json rust-plan --root "$SOURCE" --cargo "$CARGO" --rustc "$RUSTC" --target x86_64-unknown-linux-musl --no-cargo-oracle --deterministic-release-paths --execute-topology --execution-output-root "$EXECUTION" --source-built-c-compiler-route-json "$ROUTE" --rust-child-action-authority "$AUTHORITY" --rust-child-action-evidence-dir "$EVIDENCE" > "$REPORT" 2> "$STDERR"
exit_code=$?
set -e
{
  echo "exit_code=$exit_code"
  echo "report=$REPORT"
  echo "stderr=$STDERR"
  echo "evidence=$EVIDENCE"
} > "$STATUS"
exit 0
