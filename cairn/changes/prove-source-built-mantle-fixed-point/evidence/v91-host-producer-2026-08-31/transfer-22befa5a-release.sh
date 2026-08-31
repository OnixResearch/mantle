#!/bin/sh
set -eu

HOST=leviathan.cymric-daggertooth.ts.net
LOCAL=/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/source-built-receipt-finalization
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
BASE="$RUN/source-8346c02d"
REMOTE="$RUN/source-22befa5a"
LOG="$RUN/source-transfer-v91.txt"
EXPECTED_COMMIT=22befa5a
COMMIT=$(git -C "$LOCAL" rev-parse HEAD)
DRY_RUN=/tmp/mantle-source-v91-rsync-dry-run.txt

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
binary=$RUN/mantle-22befa5a-release
binary_roundtrip_parity=pending
rsync_checksum_parity=exact
root_anchored_excludes=true
excluded_roots=/.git,/.jj,/target,/.pi/worktrees
source_counts=$remote_counts
EOF"
ssh "$HOST" "cat '$LOG'"
