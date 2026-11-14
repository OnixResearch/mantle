#!/usr/bin/env bash
set -euo pipefail
umask 0022
CAPTURE=/home/brittonr/sbfp-20261113-berdKH/aws-original-source-02561621.TGISLN
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
REVISION=02561621ffa4cf17c0c4f70bc11a82df36b42ae9
EXPECTED_TREE=969d75db5c0494de4a26bbb2dcbf6837058054c1
test ! -e "$CAPTURE/capture.log"
exec > "$CAPTURE/capture.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; printf "source_capture_exit=%s\n" "$status"; printf "%s\n" "$status" > "$CAPTURE/capture.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
hostname
uname -srm
mkdir "$CAPTURE/home"
git_source() {
  env -i HOME="$CAPTURE/home" PATH=/run/current-system/sw/bin:/usr/bin:/bin GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0 /run/current-system/sw/bin/git -c core.hooksPath=/dev/null "$@"
}
git_source --version
git_source init --quiet --bare "$CAPTURE/aws-lc.git"
git_source -C "$CAPTURE/aws-lc.git" fetch --quiet --depth=1 --no-tags https://github.com/aws/aws-lc.git +$REVISION:refs/mantle-capture
OBSERVED_REVISION=$(git_source -C "$CAPTURE/aws-lc.git" rev-parse refs/mantle-capture^{commit})
OBSERVED_TREE=$(git_source -C "$CAPTURE/aws-lc.git" rev-parse refs/mantle-capture^{tree})
test "$OBSERVED_REVISION" = "$REVISION"
test "$OBSERVED_TREE" = "$EXPECTED_TREE"
MODES=$(git_source -C "$CAPTURE/aws-lc.git" ls-tree -r --format='%(objectmode)' "$REVISION" | sort -u)
[[ "$MODES" != *160000* ]]
FILE_COUNT=$(git_source -C "$CAPTURE/aws-lc.git" ls-tree -r --format='%(objecttype)' "$REVISION" | wc -l)
git_source -C "$CAPTURE/aws-lc.git" archive --format=tar --output="$CAPTURE/aws-lc-original.tar" "$REVISION"
mkdir "$CAPTURE/original"
/run/current-system/sw/bin/tar --extract --no-same-owner --file "$CAPTURE/aws-lc-original.tar" --directory "$CAPTURE/original"
ARCHIVE_BLAKE3=$("$B3" --no-names "$CAPTURE/aws-lc-original.tar")
TREE_BLAKE3=$(/run/current-system/sw/bin/tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$CAPTURE/original" . | "$B3" --no-names)
/run/current-system/sw/bin/jq -n --arg repository 'https://github.com/aws/aws-lc.git' --arg revision "$OBSERVED_REVISION" --arg git_tree "$OBSERVED_TREE" --arg archive_blake3 "$ARCHIVE_BLAKE3" --arg normalized_tar_blake3 "$TREE_BLAKE3" --arg source_root "$CAPTURE/original" --arg modes "$MODES" --argjson tracked_file_count "$FILE_COUNT" '{schema:"mantle-pinned-original-source-capture-v1",repository:$repository,revision:$revision,git_tree_sha1:$git_tree,archive_blake3:$archive_blake3,normalized_tar_blake3:$normalized_tar_blake3,source_root:$source_root,git_object_modes:$modes,tracked_file_count:$tracked_file_count,gitlinks_absent:true,non_claim:"Pinned Git source acquisition outside protected proof execution; no provider, assembler, compiler or repository code executed; not source correctness, a fixed point or release eligibility"}' > "$CAPTURE/original-source-capture.json"
"$B3" "$CAPTURE/original-source-capture.json"
date -Is
finished=1
printf 'source_capture_complete=1\n'
