#!/bin/sh
set -eu

HOST=leviathan.cymric-daggertooth.ts.net
LOCAL=/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/source-built-receipt-finalization
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
BASE="$RUN/source-2e67a21e"
REMOTE="$RUN/source-3cccfc11"
LOG="$RUN/source-transfer-v94.txt"
EXPECTED_COMMIT=3cccfc11
COMMIT=$(git -C "$LOCAL" rev-parse HEAD)
DRY_RUN=/tmp/mantle-source-v94-rsync-dry-run.txt

case "$COMMIT" in "$EXPECTED_COMMIT"*) ;; *) echo "unexpected source commit: $COMMIT" >&2; exit 1;; esac
ssh "$HOST" "set -eu; test -d '$BASE'; test ! -e '$REMOTE'; test ! -e '$LOG'; cp -a --reflink=auto '$BASE' '$REMOTE'"
rsync -a --checksum --delete --no-owner --no-group --no-perms --omit-dir-times \
  --exclude='/.git' \
  --exclude='/.jj' \
  --exclude='/target' \
  --exclude='/.pi/worktrees' \
  "$LOCAL/" "$HOST:$REMOTE/"
rsync -a --checksum --delete --no-owner --no-group --no-perms --omit-dir-times --dry-run --itemize-changes \
  --exclude='/.git' \
  --exclude='/.jj' \
  --exclude='/target' \
  --exclude='/.pi/worktrees' \
  "$LOCAL/" "$HOST:$REMOTE/" > "$DRY_RUN"
test ! -s "$DRY_RUN"
remote_counts=$(ssh "$HOST" "find '$REMOTE' -xdev \( -type f -o -type d \) -printf '%y\n' | sort | uniq -c | tr '\n' ';'")
ssh "$HOST" "cat > '$LOG' <<'EOF'
captured_at=$(date -Is)
commit=$COMMIT
source=$REMOTE
binary=$RUN/mantle-3cccfc11-release
binary_roundtrip_parity=pending
rsync_checksum_parity=exact
root_anchored_excludes=true
excluded_roots=/.git,/.jj,/target,/.pi/worktrees
source_counts=$remote_counts
EOF"
ssh "$HOST" "cat '$LOG'"
