#!/bin/sh
set -eu

: "${MANTLE_REBUILD_SOURCE_ARCHIVE:?content-bound release source archive is required}"
: "${MANTLE_REBUILD_EXECUTABLE:?content-bound rebuild executor is required}"
: "${MANTLE_REPRODUCE_OUTPUT_DIR:?fresh rebuild output root is required}"
: "${MANTLE_DETERMINISTIC_PROOF_STORE_DIR:?fresh rebuild store root is required}"

readonly BB="$MANTLE_REBUILD_EXECUTABLE"
readonly TOOLCHAIN_ARCHIVE="${1:?content-bound toolchain archive argument is required}"
readonly SOURCE_ROOT="$MANTLE_DETERMINISTIC_PROOF_STORE_DIR/source"
readonly TOOLCHAIN_ROOT="$MANTLE_DETERMINISTIC_PROOF_STORE_DIR/toolchain"
readonly CARGO_TARGET_ROOT="$MANTLE_DETERMINISTIC_PROOF_STORE_DIR/cargo-target"
readonly TEMP_ROOT="$MANTLE_DETERMINISTIC_PROOF_STORE_DIR/tmp"
readonly OUTPUT_RELATIVE_PATH="binaries/01-stage2-mantle"

"$BB" mkdir -p "$SOURCE_ROOT" "$TOOLCHAIN_ROOT" "$CARGO_TARGET_ROOT" "$TEMP_ROOT"
"$BB" tar -xf "$MANTLE_REBUILD_SOURCE_ARCHIVE" -C "$SOURCE_ROOT"
"$BB" tar -xf "$TOOLCHAIN_ARCHIVE" -C "$TOOLCHAIN_ROOT"

readonly CARGO="$TOOLCHAIN_ROOT/bin/cargo"
readonly RUSTC="$TOOLCHAIN_ROOT/bin/rustc"
"$BB" test -x "$CARGO"
"$BB" test -x "$RUSTC"
"$BB" test -f "$SOURCE_ROOT/Cargo.toml"
"$BB" test -f "$SOURCE_ROOT/Cargo.lock"
"$BB" test -f "$SOURCE_ROOT/.cargo/vendor-config.toml"

export CARGO_TARGET_DIR="$CARGO_TARGET_ROOT"
export RUSTC
export TMPDIR="$TEMP_ROOT"
export TEMP="$TEMP_ROOT"
export TMP="$TEMP_ROOT"
export TEMPDIR="$TEMP_ROOT"
export SNIX_BUILD_SANDBOX_SHELL="$BB"
export RUSTC_BOOTSTRAP=1

(
  cd "$SOURCE_ROOT"
  "$CARGO" build \
    --offline \
    --locked \
    --release \
    --bin mantle \
    --config .cargo/vendor-config.toml
)

readonly REBUILT_BINARY="$CARGO_TARGET_ROOT/release/mantle"
"$BB" test -x "$REBUILT_BINARY"
"$BB" mkdir -p "$MANTLE_REPRODUCE_OUTPUT_DIR/binaries"
"$BB" cp "$REBUILT_BINARY" "$MANTLE_REPRODUCE_OUTPUT_DIR/$OUTPUT_RELATIVE_PATH"
