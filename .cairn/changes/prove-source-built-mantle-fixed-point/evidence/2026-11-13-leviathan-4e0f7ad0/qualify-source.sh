#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/qualification-4e0f7ad0.log" 2>&1
trap 's=$?; printf "qualification_4e0f7ad0_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/qualification-4e0f7ad0.exit"; exit "$s"' EXIT
set -x
date -Is
ARCHIVE="$RUN/incoming/source-4e0f7ad0f228773710e57ea8938e1d70d73fdc20.tar.gz"
SOURCE="$RUN/runner-source-4e0f7ad0"
test "$("$B3" --no-names "$ARCHIVE")" = abecc71c89e6a0bba7d7fc19beeb29a062d6f64928ef9b986bf0972739609a7f
test "$(gzip --decompress --stdout "$ARCHIVE" | "$B3" --no-names)" = 88fc87d89af06757004943586df32b910330d745fb57b58b2793c784ec7212b8
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/runner-tmp-4e0f7ad0"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
source_digest() {
  tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names
}
SOURCE_BEFORE=$(source_digest)
printf 'frozen_source_revision=%s\nsource_tree_before_blake3=%s\n' 4e0f7ad0f228773710e57ea8938e1d70d73fdc20 "$SOURCE_BEFORE"
cd "$SOURCE"
export TMPDIR="$RUN/runner-tmp-4e0f7ad0" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo -V; cargo fmt --check -p mantle -v; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings; cargo build --release --locked --offline -p mantle --bin mantle'
SOURCE_AFTER=$(source_digest)
printf 'source_tree_after_blake3=%s\n' "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
install -m 755 "$CARGO_TARGET_DIR/release/mantle" "$RUN/bin/mantle-4e0f7ad0"
"$B3" "$RUN/bin/mantle-4e0f7ad0" > "$RUN/runner-blake3-4e0f7ad0.txt"
"$RUN/bin/mantle-4e0f7ad0" --version
date -Is
