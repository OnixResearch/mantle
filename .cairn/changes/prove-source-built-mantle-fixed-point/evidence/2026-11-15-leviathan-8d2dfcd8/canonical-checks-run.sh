#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
test ! -e "$RUN/canonical-qualification-8d2dfcd8-typed-invocation.log"
exec > "$RUN/canonical-qualification-8d2dfcd8-typed-invocation.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; printf "canonical-qualification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/canonical-qualification-8d2dfcd8-typed-invocation.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
test "$(cat "$RUN/assemble-complete-source-8d2dfcd8-typed-invocation.exit")" = 0
META="$RUN/complete-source-8d2dfcd8-typed-invocation.json"
jq -e '.schema == "mantle-complete-source-snapshot-v1" and .source_code_revision == "8d2dfcd88e7b36a20637b2376f312ce29747d1bc" and .code_archive_blake3 == "6cf7a625552ae8e128a5b893806d70b2d8dd6bb199dec205e3f70800196c8e07" and .original_git_commit == "02561621ffa4cf17c0c4f70bc11a82df36b42ae9" and .original_git_tree_sha1 == "969d75db5c0494de4a26bbb2dcbf6837058054c1" and .original_source_archive_blake3 == "282f13d4bb302e497915301156539943fd3e1e45f6a60f36678ae13e40636a88" and .original_source_normalized_tar_blake3 == "8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89"' "$META"
ARCHIVE="$RUN/incoming/complete-source-8d2dfcd8-typed-invocation.tar.gz"
test "$("$B3" --no-names "$ARCHIVE")" = "$(jq -er .compressed_archive_blake3 "$META")"
SOURCE="$RUN/canonical-source-8d2dfcd8-typed-invocation"
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/canonical-tmp-8d2dfcd8-typed-invocation"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
cd "$SOURCE"
export TMPDIR="$RUN/canonical-tmp-8d2dfcd8-typed-invocation" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/canonical-target-8fa84260-aws" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c /run/current-system/sw/bin/bash "$RUN/canonical-checks-8d2dfcd8-typed-invocation-inside.sh"
date -Is
finished=1
printf 'canonical_qualification_complete=1\n'
