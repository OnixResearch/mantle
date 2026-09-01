#!/bin/sh
set -eu

HOST=leviathan.cymric-daggertooth.ts.net
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
LOCAL=/tmp/mantle-release-a1537652/release/mantle
REMOTE="$RUN/mantle-a1537652-release"
REMOTE_TMP="$REMOTE.tmp"
ROUNDTRIP=/tmp/mantle-a1537652-release-roundtrip
LOG="$RUN/source-transfer-v87.txt"
B3_LOCAL=/run/current-system/sw/bin/b3sum
B3_REMOTE="$RUN/b3sum-1.8.5-operator"

for required in "$LOCAL" "$B3_LOCAL"; do test -x "$required"; done
ssh "$HOST" "set -eu; test -x '$B3_REMOTE'; test -f '$LOG'; test ! -e '$REMOTE'; test ! -e '$REMOTE_TMP'"
local_digest=$("$B3_LOCAL" --no-names "$LOCAL")
rsync -a --checksum "$LOCAL" "$HOST:$REMOTE_TMP"
ssh "$HOST" "set -eu; chmod 0555 '$REMOTE_TMP'; mv '$REMOTE_TMP' '$REMOTE'"
remote_digest=$(ssh "$HOST" "'$B3_REMOTE' --no-names '$REMOTE'")
test "$remote_digest" = "$local_digest"
rm -f "$ROUNDTRIP"
rsync -a --checksum "$HOST:$REMOTE" "$ROUNDTRIP"
roundtrip_digest=$("$B3_LOCAL" --no-names "$ROUNDTRIP")
test "$roundtrip_digest" = "$local_digest"
rm -f "$ROUNDTRIP"
ssh "$HOST" "set -eu; tmp='$LOG.tmp'; awk -v digest='$local_digest' 'BEGIN{done=0} /^binary_roundtrip_parity=/{print \"binary_blake3=\" digest; print \"binary_roundtrip_parity=exact\"; done=1; next} {print} END{if(!done) exit 1}' '$LOG' > \"\$tmp\"; mv \"\$tmp\" '$LOG'; cat '$LOG'"
