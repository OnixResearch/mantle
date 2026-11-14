#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/qualification-4653ddef.log" 2>&1
trap 's=$?; printf "qualification_4653ddef_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/qualification-4653ddef.exit"; exit "$s"' EXIT
set -x
date -Is
ARCHIVE="$RUN/incoming/source-4653ddef4edd196620f0a3337b2c7045c8cf4efd.tar.gz"
SOURCE="$RUN/runner-source-4653ddef"
test "$("$B3" --no-names "$ARCHIVE")" = 0030d50e5c78a90d325e969eec8848174c06cacbae561f1a3768a905c8792c14
test "$(gzip --decompress --stdout "$ARCHIVE" | "$B3" --no-names)" = 75845ae712c685ca9a9aa2c2120c5f13934551845b65de9b329aec2868fae4a4
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/runner-tmp-4653ddef"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
source_digest() {
  tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names
}
SOURCE_BEFORE=$(source_digest)
printf 'frozen_source_revision=%s\nsource_tree_before_blake3=%s\n' 4653ddef4edd196620f0a3337b2c7045c8cf4efd "$SOURCE_BEFORE"
cd "$SOURCE"
export TMPDIR="$RUN/runner-tmp-4653ddef" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo -V; cargo fmt --check -p mantle -v; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings; cargo build --release --locked --offline -p mantle --bin mantle'
SOURCE_AFTER=$(source_digest)
printf 'source_tree_after_blake3=%s\n' "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
install -m 755 "$CARGO_TARGET_DIR/release/mantle" "$RUN/bin/mantle-4653ddef"
"$B3" "$RUN/bin/mantle-4653ddef" > "$RUN/runner-blake3-4653ddef.txt"
"$RUN/bin/mantle-4653ddef" --version
date -Is
