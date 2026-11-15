#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
test ! -e "$RUN/qualification-e9a58881-typed-index.log"
exec > "$RUN/qualification-e9a58881-typed-index.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; printf "qualification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/qualification-e9a58881-typed-index.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
test "$(cat "$RUN/assemble-complete-source-e9a58881-typed-index.exit")" = 0
META="$RUN/complete-source-e9a58881-typed-index.json"
jq -e '.schema == "mantle-complete-source-snapshot-v1" and .source_code_revision == "e9a58881b9ff491f47864632367c9b8ea4af76e3" and .code_archive_blake3 == "494a677f912de7f25d9efadde55ba6bcc3052bf9193a80022332a1da810ee4a5" and .original_git_commit == "02561621ffa4cf17c0c4f70bc11a82df36b42ae9" and .original_git_tree_sha1 == "969d75db5c0494de4a26bbb2dcbf6837058054c1" and .original_source_archive_blake3 == "282f13d4bb302e497915301156539943fd3e1e45f6a60f36678ae13e40636a88" and .original_source_normalized_tar_blake3 == "8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89"' "$META"
ARCHIVE="$RUN/incoming/complete-source-e9a58881-typed-index.tar.gz"
test "$("$B3" --no-names "$ARCHIVE")" = "$(jq -er .compressed_archive_blake3 "$META")"
SOURCE="$RUN/runner-source-e9a58881-typed-index"
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/runner-tmp-e9a58881-typed-index"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
source_digest() {
 tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names
}
SOURCE_BEFORE=$(source_digest)
test "$SOURCE_BEFORE" = "$(jq -er .complete_archive_blake3 "$META")"
printf 'frozen_source_revision=%s\nsource_tree_before_blake3=%s\n' e9a58881b9ff491f47864632367c9b8ea4af76e3 "$SOURCE_BEFORE"
cd "$SOURCE"
export TMPDIR="$RUN/runner-tmp-e9a58881-typed-index" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo -V; perl --version; cargo fmt --check -p mantle -v; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings; cargo build --release --locked --offline -p mantle --bin mantle'
SOURCE_AFTER=$(source_digest)
printf 'source_tree_after_blake3=%s\n' "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
install -m 755 "$CARGO_TARGET_DIR/release/mantle" "$RUN/bin/mantle-e9a58881-typed-index"
"$B3" "$RUN/bin/mantle-e9a58881-typed-index" > "$RUN/runner-blake3-e9a58881-typed-index.txt"
"$RUN/bin/mantle-e9a58881-typed-index" --version
date -Is
finished=1
printf 'qualification_complete=1\n'
