#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/qualification-22eb45ca.log" 2>&1
trap 's=$?; printf "qualification_22eb45ca_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/qualification-22eb45ca.exit"; exit "$s"' EXIT
set -x
date -Is
ARCHIVE="$RUN/incoming/source-22eb45ca5321416fce3278a228a43224dd0026c5.tar.gz"
SOURCE="$RUN/runner-source-22eb45ca"
test "$("$B3" --no-names "$ARCHIVE")" = ecdd35a1099f16746496811c57d90115e3c57411828f9737173d98ce29cff7db
test "$(gzip --decompress --stdout "$ARCHIVE" | "$B3" --no-names)" = 75622e1209102d6f1cdbaf5187688a05710e0380c9525d18e9d759ea8d93f0f0
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/runner-tmp-22eb45ca"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
source_digest() {
  tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names
}
SOURCE_BEFORE=$(source_digest)
printf 'frozen_source_revision=%s\nsource_tree_before_blake3=%s\n' 22eb45ca5321416fce3278a228a43224dd0026c5 "$SOURCE_BEFORE"
cd "$SOURCE"
export TMPDIR="$RUN/runner-tmp-22eb45ca" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo -V; cargo fmt --check -p mantle -v; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings; cargo build --release --locked --offline -p mantle --bin mantle'
SOURCE_AFTER=$(source_digest)
printf 'source_tree_after_blake3=%s\n' "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
install -m 755 "$CARGO_TARGET_DIR/release/mantle" "$RUN/bin/mantle-22eb45ca"
"$B3" "$RUN/bin/mantle-22eb45ca" > "$RUN/runner-blake3-22eb45ca.txt"
"$RUN/bin/mantle-22eb45ca" --version
date -Is
