#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
NAME=b3a39a01-immutable-source
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
test ! -e "$RUN/canonical-qualification-b3a39a01-immutable-source.log"
exec > "$RUN/canonical-qualification-b3a39a01-immutable-source.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; printf "canonical-qualification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/canonical-qualification-b3a39a01-immutable-source.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
test "$(cat "$RUN/assemble-complete-source-b3a39a01-immutable-source.exit")" = 0
META="$RUN/complete-source-b3a39a01-immutable-source.json"
jq -e '.schema == "mantle-complete-source-snapshot-v1" and .source_code_revision == "b3a39a0118839fd6f7267c9b1c99258155ff40af" and .code_archive_blake3 == "1746db51499dae55736d17bda8c7a81f0572ace12dd9643c8c11cc35f760c393" and .original_git_commit == "02561621ffa4cf17c0c4f70bc11a82df36b42ae9" and .original_git_tree_sha1 == "969d75db5c0494de4a26bbb2dcbf6837058054c1" and .original_source_archive_blake3 == "282f13d4bb302e497915301156539943fd3e1e45f6a60f36678ae13e40636a88" and .original_source_normalized_tar_blake3 == "8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89"' "$META"
ARCHIVE="$RUN/incoming/complete-source-b3a39a01-immutable-source.tar.gz"
test "$("$B3" --no-names "$ARCHIVE")" = "$(jq -er .compressed_archive_blake3 "$META")"
SOURCE="$RUN/canonical-source-b3a39a01-immutable-source"
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/canonical-tmp-b3a39a01-immutable-source"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
cd "$SOURCE"
source_digest() { tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names; }
SOURCE_BEFORE=$(source_digest)
test "$SOURCE_BEFORE" = "$(jq -er .complete_archive_blake3 "$META")"
export TMPDIR="$RUN/canonical-tmp-b3a39a01-immutable-source" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="/home/brittonr/sbfp-20261113-berdKH/canonical-target-b3a39a01-immutable-source" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
test ! -e "$CARGO_TARGET_DIR"
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
export MANTLE_CAIRN_REPORT_DIR="$RUN/canonical-cairn-$NAME"
test ! -e "$MANTLE_CAIRN_REPORT_DIR"
mkdir "$MANTLE_CAIRN_REPORT_DIR"
/run/current-system/sw/bin/nix develop --profile "$RUN/tooling-profile-canonical-$NAME" --builders '' --no-write-lock-file -c /run/current-system/sw/bin/bash "$RUN/canonical-checks-b3a39a01-immutable-source-inside.sh"
SOURCE_AFTER=$(source_digest)
printf 'source_tree_before_blake3=%s\nsource_tree_after_blake3=%s\n' "$SOURCE_BEFORE" "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
date -Is
finished=1
printf 'canonical_qualification_complete=1\n'
