#!/usr/bin/env bash
set -euo pipefail
umask 0022
RUN=/home/brittonr/sbfp-20261113-berdKH
CAPTURE=/home/brittonr/sbfp-20261113-berdKH/aws-original-source-02561621.TGISLN
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
ASSEMBLY="$RUN/source-assembly-71c2bd4d-typed-invocation"
ARCHIVE="$RUN/incoming/complete-source-71c2bd4d-typed-invocation.tar"
CODE_ARCHIVE="$RUN/incoming/source-71c2bd4de8ec66e545b42314ba95bf050396a36d.tar.gz"
DEST="$ASSEMBLY/bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9"
test ! -e "$RUN/assemble-complete-source-71c2bd4d-typed-invocation.log"
exec > "$RUN/assemble-complete-source-71c2bd4d-typed-invocation.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; printf "assembly_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/assemble-complete-source-71c2bd4d-typed-invocation.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
test "$("$B3" --no-names "$CODE_ARCHIVE")" = 2892fd29b7f6d137bcbf951e773d7ace2950bc122a021fa91995aba29bc99d78
test "$("$B3" --no-names "$CAPTURE/aws-lc-original.tar")" = 282f13d4bb302e497915301156539943fd3e1e45f6a60f36678ae13e40636a88
test "$(/run/current-system/sw/bin/jq -r .revision "$CAPTURE/original-source-capture.json")" = 02561621ffa4cf17c0c4f70bc11a82df36b42ae9
test "$(/run/current-system/sw/bin/jq -r .git_tree_sha1 "$CAPTURE/original-source-capture.json")" = 969d75db5c0494de4a26bbb2dcbf6837058054c1
test "$(gzip --decompress --stdout "$CODE_ARCHIVE" | "$B3" --no-names)" = 3c7a49f4e4e353a37eb5eff5b69057e4751164870f89f8a43ca909bd62b4fdc2
test ! -e "$ASSEMBLY"
test ! -e "$ARCHIVE"
test ! -e "$ARCHIVE.gz"
mkdir "$ASSEMBLY"
/run/current-system/sw/bin/tar --extract --gzip --no-same-owner --file "$CODE_ARCHIVE" --directory "$ASSEMBLY"
CODE_TREE=$(/run/current-system/sw/bin/tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$ASSEMBLY" . | "$B3" --no-names)
test ! -e "$DEST"
mkdir -p "$DEST"
/run/current-system/sw/bin/tar --extract --no-same-owner --file "$CAPTURE/aws-lc-original.tar" --directory "$DEST"
ORIGINAL_TREE=$(/run/current-system/sw/bin/tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$DEST" . | "$B3" --no-names)
test "$ORIGINAL_TREE" = 8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89
/run/current-system/sw/bin/tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file "$ARCHIVE" --directory "$ASSEMBLY" .
RAW_BLAKE3=$("$B3" --no-names "$ARCHIVE")
gzip --no-name --keep "$ARCHIVE"
COMPRESSED_BLAKE3=$("$B3" --no-names "$ARCHIVE.gz")
/run/current-system/sw/bin/jq -n --arg source_code_revision '71c2bd4de8ec66e545b42314ba95bf050396a36d' --arg code_archive_blake3 '2892fd29b7f6d137bcbf951e773d7ace2950bc122a021fa91995aba29bc99d78' --arg code_tree_blake3 "$CODE_TREE" --arg original_git_commit '02561621ffa4cf17c0c4f70bc11a82df36b42ae9' --arg original_git_tree '969d75db5c0494de4a26bbb2dcbf6837058054c1' --arg original_source_archive_blake3 '282f13d4bb302e497915301156539943fd3e1e45f6a60f36678ae13e40636a88' --arg original_source_tree_blake3 "$ORIGINAL_TREE" --arg complete_archive_blake3 "$RAW_BLAKE3" --arg compressed_archive_blake3 "$COMPRESSED_BLAKE3" --arg archive "$ARCHIVE" --arg assembly_root "$ASSEMBLY" '{schema:"mantle-complete-source-snapshot-v1",source_code_revision:$source_code_revision,code_archive_blake3:$code_archive_blake3,code_tree_blake3:$code_tree_blake3,original_git_commit:$original_git_commit,original_git_tree_sha1:$original_git_tree,original_source_archive_blake3:$original_source_archive_blake3,original_source_normalized_tar_blake3:$original_source_tree_blake3,complete_archive_blake3:$complete_archive_blake3,compressed_archive_blake3:$compressed_archive_blake3,archive:$archive,assembly_root:$assembly_root,non_claim:"Composite source snapshot only: frozen Mantle code plus the declared original source input; no runner build, provider execution, source correctness or fixed-point claim"}' > "$RUN/complete-source-71c2bd4d-typed-invocation.json"
"$B3" "$ARCHIVE" "$ARCHIVE.gz" "$RUN/complete-source-71c2bd4d-typed-invocation.json"
date -Is
finished=1
printf 'complete_source_assembly=1\n'
