#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
NAME=71c2bd4d-fresh-target
SOURCE=/home/brittonr/sbfp-20261113-berdKH/runner-source-71c2bd4d-typed-invocation
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
META=/home/brittonr/sbfp-20261113-berdKH/complete-source-71c2bd4d-typed-invocation.json
ARCHIVE=/home/brittonr/sbfp-20261113-berdKH/incoming/complete-source-71c2bd4d-typed-invocation.tar.gz
test ! -e "$RUN/qualification-$NAME.log"
exec > "$RUN/qualification-$NAME.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ] && [ "$finished" -ne 1 ]; then status=1; fi; printf "qualification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/qualification-$NAME.exit"; exit "$status"' EXIT
set -x
date -Is
jq -e '.schema == "mantle-complete-source-snapshot-v1" and .source_code_revision == "71c2bd4de8ec66e545b42314ba95bf050396a36d" and .complete_archive_blake3 == "b49c087adf2e6f7c15fd6e4fb80bc2c614546a4efba8a59df279c61d79059778"' "$META"
test "$("$B3" --no-names "$ARCHIVE")" = "$(jq -er .compressed_archive_blake3 "$META")"
source_digest() { tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names; }
SOURCE_BEFORE=$(source_digest)
test "$SOURCE_BEFORE" = "$(jq -er .complete_archive_blake3 "$META")"
export CARGO_TARGET_DIR=/home/brittonr/sbfp-20261113-berdKH/runner-build-target-71c2bd4d-fresh-target
test ! -e "$CARGO_TARGET_DIR"
export TMPDIR="$RUN/runner-tmp-$NAME" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
mkdir "$TMPDIR"
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
cd "$SOURCE"
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; rustc -Vv; cargo -V; printf "fresh_target=%s\n" "$CARGO_TARGET_DIR"; cargo fmt --check -p mantle -v; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings; cargo build --release --locked --offline -p mantle --bin mantle'
SOURCE_AFTER=$(source_digest)
printf 'source_tree_before_blake3=%s\nsource_tree_after_blake3=%s\n' "$SOURCE_BEFORE" "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
install -m 755 "$CARGO_TARGET_DIR/release/mantle" "$RUN/bin/mantle-$NAME"
"$B3" "$RUN/bin/mantle-$NAME" > "$RUN/runner-blake3-$NAME.txt"
"$RUN/bin/mantle-$NAME" --version
date -Is
finished=1
printf 'qualification_complete=1\n'
