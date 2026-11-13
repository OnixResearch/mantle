#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/qualification-07ccbf78.log" 2>&1
trap 's=$?; printf "qualification_07ccbf78_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/qualification-07ccbf78.exit"; exit "$s"' EXIT
set -x
date -Is
ARCHIVE="$RUN/incoming/source-07ccbf782ca5f60114222b19222ee944bdcd4cba.tar.gz"
SOURCE="$RUN/runner-source-07ccbf78"
test "$("$B3" --no-names "$ARCHIVE")" = 5a4987b7f26f59d698018d764ba154d6214fec92c5524b861f78de1189cb0eeb
test "$(gzip --decompress --stdout "$ARCHIVE" | "$B3" --no-names)" = 6f6223e3c4da94c8338093ba9e158df40baa02db3d2ae82087f6374dffd018c1
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/runner-tmp-07ccbf78"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
source_digest() {
  tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names
}
SOURCE_BEFORE=$(source_digest)
printf 'frozen_source_revision=%s\nsource_tree_before_blake3=%s\n' 07ccbf782ca5f60114222b19222ee944bdcd4cba "$SOURCE_BEFORE"
cd "$SOURCE"
export TMPDIR="$RUN/runner-tmp-07ccbf78" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo -V; cargo fmt --check -p mantle -v; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings; cargo build --release --locked --offline -p mantle --bin mantle'
SOURCE_AFTER=$(source_digest)
printf 'source_tree_after_blake3=%s\n' "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
install -m 755 "$CARGO_TARGET_DIR/release/mantle" "$RUN/bin/mantle-07ccbf78"
"$B3" "$RUN/bin/mantle-07ccbf78" > "$RUN/runner-blake3-07ccbf78.txt"
"$RUN/bin/mantle-07ccbf78" --version
date -Is
