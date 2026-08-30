#!/bin/sh
set -eu

HOST=leviathan.cymric-daggertooth.ts.net
LOCAL=/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/source-built-receipt-finalization
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
BASE="$RUN/source-0d4211e5"
REMOTE="$RUN/source-28bfb932"
LOG="$RUN/source-transfer-v83.txt"
COMMIT=28bfb932a5f6c9098054c98841c22d4ce23f5f11
DRY_RUN=/tmp/mantle-source-v83-rsync-dry-run.txt

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
binary=$RUN/mantle-28bfb932-release
binary_roundtrip_parity=pending
rsync_checksum_parity=exact
root_anchored_excludes=true
excluded_roots=/.git,/.jj,/target,/.pi/worktrees
source_counts=$remote_counts
EOF"
ssh "$HOST" "cat '$LOG'"
