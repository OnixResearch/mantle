#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/frozen-runner-build-72f6d21f.log" 2>&1
trap 's=$?; printf "frozen_runner_build_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/frozen-runner-build-72f6d21f.exit"; exit "$s"' EXIT
set -x
date -Is
ARCHIVE="$RUN/incoming/source-72f6d21f290b7be6c885c98a2a3433416b7d03be.tar"
test "$("$B3" --no-names "$ARCHIVE")" = 6f5d2b2483b9f09884945d82cc09d7278fb42f5f5294f470b16edb2a84c85629
test ! -e "$RUN/runner-build-source"
mkdir "$RUN/runner-build-source" "$RUN/runner-build-tmp"
tar --extract --no-same-owner --file "$ARCHIVE" --directory "$RUN/runner-build-source"
source_digest() {
  tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$RUN/runner-build-source" . | "$B3" --no-names
}
SOURCE_BEFORE=$(source_digest)
printf 'frozen_source_revision=%s\nsource_tree_before_blake3=%s\n' 72f6d21f290b7be6c885c98a2a3433416b7d03be "$SOURCE_BEFORE"
cd "$RUN/runner-build-source"
export TMPDIR="$RUN/runner-build-tmp" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo -V; cargo fmt --check -p mantle -v; cargo build --release --locked --offline -p mantle --bin mantle'
SOURCE_AFTER=$(source_digest)
printf 'source_tree_after_blake3=%s\n' "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
install -m 755 "$CARGO_TARGET_DIR/release/mantle" "$RUN/bin/mantle-frozen"
"$B3" "$RUN/bin/mantle-frozen" > "$RUN/frozen-runner-blake3.txt"
"$RUN/bin/mantle-frozen" --version
date -Is
