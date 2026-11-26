#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
FUZZ_SECONDS="${FUZZ_SECONDS:-30}"
FUZZ_FILE_SECONDS="${FUZZ_FILE_SECONDS:-15}"
RSS_LIMIT_MB="${RSS_LIMIT_MB:-1024}"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/mantle-fuzz-work-XXXXXX")"
trap 'rm -rf "$WORK_DIR"' EXIT

export ASAN_OPTIONS="abort_on_error=1:halt_on_error=1:detect_stack_use_after_return=1:max_malloc_fill_size=$((1<<30))"
export UBSAN_OPTIONS="print_stacktrace=1:halt_on_error=1:abort_on_error=1"

echo "[1/4] Compiling C libFuzzer targets into $WORK_DIR"
clang -g3 -O1 -fsanitize=address,undefined,fuzzer -fno-sanitize-recover=all \
  "$SCRIPT_DIR/c/elf_canonicalizer_fuzz.c" -o "$WORK_DIR/elf_c_fuzz"
clang -g3 -O1 -fsanitize=address,undefined,fuzzer -fno-sanitize-recover=all \
  "$SCRIPT_DIR/c/elf_canonicalizer_file_fuzz.c" -o "$WORK_DIR/elf_c_file_fuzz"

echo "[2/4] Compiling Rust staticlib (SanCov + ASan + MANTLE_FUZZING=1)"
TARGET_DIR="${CARGO_TARGET_DIR:-$WORK_DIR/target}-fuzz"
FUZZ_RUSTFLAGS="-C opt-level=2 -C passes=sancov-module -C llvm-args=-sanitizer-coverage-level=4 -C llvm-args=-sanitizer-coverage-inline-8bit-counters -C llvm-args=-sanitizer-coverage-pc-table -Z sanitizer=address"
MANTLE_FUZZING=1 RUSTFLAGS="$FUZZ_RUSTFLAGS" FUZZ_SANITIZE_C=1 cargo build --offline \
  --manifest-path "$SCRIPT_DIR/Cargo.toml" \
  --lib \
  --target x86_64-unknown-linux-gnu \
  --target-dir "$TARGET_DIR"

LIB_A="$TARGET_DIR/x86_64-unknown-linux-gnu/debug/libmantle_fuzz.a"

echo "[3/4] Linking Rust & Differential libFuzzer binaries into $WORK_DIR"
for pair in \
  "elf_rust_fuzz:FUZZ_TARGET_ELF_RUST" \
  "archive_rust_fuzz:FUZZ_TARGET_ARCHIVE_RUST" \
  "elf_diff_fuzz:FUZZ_TARGET_ELF_DIFF" \
  "store_archive_list_fuzz:FUZZ_TARGET_STORE_ARCHIVE_LIST"
do
  bin_name="${pair%%:*}"
  macro_name="${pair##*:}"
  clang -g3 -O1 -fsanitize=address,undefined,fuzzer -fno-sanitize-recover=all \
    -D"$macro_name" \
    "$SCRIPT_DIR/c/rust_fuzz_shim.c" \
    "$LIB_A" \
    -lpthread -ldl -lm \
    -o "$WORK_DIR/$bin_name"
done

run_target() {
  local bin_name="$1"
  local corpus_kind="$2"
  local secs="$3"
  shift 3
  local run_corpus="$WORK_DIR/corpus-$bin_name"
  mkdir -p "$run_corpus"
  cp -f "$SCRIPT_DIR/corpus/$corpus_kind"/* "$run_corpus/" 2>/dev/null || true
  cp -f "$SCRIPT_DIR/regressions/$corpus_kind"/* "$run_corpus/" 2>/dev/null || true
  echo "=== Running $bin_name for ${secs}s (rss_limit_mb=$RSS_LIMIT_MB) ==="
  "$WORK_DIR/$bin_name" "$run_corpus" \
    -max_total_time="$secs" \
    -rss_limit_mb="$RSS_LIMIT_MB" \
    -artifact_prefix="$WORK_DIR/crash-$bin_name-" \
    "$@"
}

echo "[4/4] Running bounded libFuzzer smoke campaigns"
run_target elf_c_fuzz elf "$FUZZ_SECONDS"
run_target elf_c_file_fuzz elf "$FUZZ_FILE_SECONDS" -max_len=65536
run_target elf_rust_fuzz elf "$FUZZ_SECONDS"
run_target elf_diff_fuzz elf "$FUZZ_SECONDS"
run_target archive_rust_fuzz archive "$FUZZ_SECONDS"
run_target store_archive_list_fuzz store_archive "$FUZZ_SECONDS"

echo "All libFuzzer smoke campaigns completed cleanly."
