#!/usr/bin/env bash
set -euo pipefail
RUN='/home/brittonr/sbfp-20261113-berdKH'
NAME='70db0868-primary-sources'
B3='/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
SOURCEBASE="$RUN/source-assembly-$NAME/.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-17-immutable-compiler-source"
DEST='/home/brittonr/sbfp-20261113-berdKH/review-tools-70db0868-primary-sources'
test ! -e "$RUN/copy-review-tools-$NAME.log"
exec > "$RUN/copy-review-tools-$NAME.log" 2>&1
trap 'status=$?; printf "review_tools_copy_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
test "$(cat "$RUN/assemble-complete-source-$NAME.exit")" = 0
jq -e '.source_code_revision == "70db08685c0c5fa5c326bfe32db78576d7e2e182" and .code_archive_blake3 == "e40bff16144520416cacb3b865b2eaba01dab4bf194a44c0b0d9cc18b9ec19fe"' "$RUN/complete-source-$NAME.json"
test ! -e "$DEST"
mkdir "$DEST"
rsync -a -- "$SOURCEBASE/graph-review" "$SOURCEBASE/identity-review" "$SOURCEBASE/check-helper-mutations.py" "$SOURCEBASE/check-selector-mutations.py" "$DEST/"
for member in graph-review/graph-audit.py graph-review/controller-template-audit.py graph-review/helper-authority-audit.py graph-review/mantle-selector-audit.py graph-review/run-semantic-root-inspection.sh identity-review/identity-audit.py identity-review/preauthor-role-physical-audit.py identity-review/run-full-identity.sh identity-review/run-preauthor-roles.sh check-helper-mutations.py check-selector-mutations.py; do
  test -f "$DEST/$member"
  test ! -L "$DEST/$member"
  test "$("$B3" --no-names "$SOURCEBASE/$member")" = "$("$B3" --no-names "$DEST/$member")"
  "$B3" "$DEST/$member"
done
printf 'review_tools_copy_complete=1\n'
