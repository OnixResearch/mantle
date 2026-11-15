#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
NAME=787784fb-helper-feature
SOURCE=/home/brittonr/sbfp-20261113-berdKH/runner-source-787784fb-helper-feature
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
META="$RUN/complete-source-$NAME.json"
ARCHIVE="$RUN/incoming/complete-source-$NAME.tar.gz"
test ! -e "$RUN/qualification-$NAME.log"
exec > "$RUN/qualification-$NAME.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ] && [ "$finished" -ne 1 ]; then status=1; fi; printf "qualification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/qualification-$NAME.exit"; exit "$status"' EXIT
set -x
date -Is
test "$(cat "$RUN/assemble-complete-source-$NAME.exit")" = 0
jq -e '.schema == "mantle-complete-source-snapshot-v1" and .source_code_revision == "787784fbb6bf2141fce5aa2be9f1741ac9f2f013" and .code_archive_blake3 == "8ed485dbd6fa2824902ee6ad74a62af06efd484202ed8edd94542aa0583258a8" and .original_source_normalized_tar_blake3 == "8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89"' "$META"
test "$("$B3" --no-names "$ARCHIVE")" = "$(jq -er .compressed_archive_blake3 "$META")"
test ! -e "$SOURCE"
mkdir "$SOURCE"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
source_digest() { tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names; }
SOURCE_BEFORE=$(source_digest)
test "$SOURCE_BEFORE" = "$(jq -er .complete_archive_blake3 "$META")"
export CARGO_TARGET_DIR=/home/brittonr/sbfp-20261113-berdKH/runner-build-target-787784fb-helper-feature
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
