#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/canonical-qualification-4e0f7ad0.log" 2>&1
trap 's=$?; printf "canonical_qualification_4e0f7ad0_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/canonical-qualification-4e0f7ad0.exit"; exit "$s"' EXIT
set -x
test "$(cat "$RUN/qualification-4e0f7ad0.exit")" = 0
ARCHIVE="$RUN/incoming/source-4e0f7ad0f228773710e57ea8938e1d70d73fdc20.tar.gz"
test "$("$B3" --no-names "$ARCHIVE")" = abecc71c89e6a0bba7d7fc19beeb29a062d6f64928ef9b986bf0972739609a7f
test "$(gzip --decompress --stdout "$ARCHIVE" | "$B3" --no-names)" = 88fc87d89af06757004943586df32b910330d745fb57b58b2793c784ec7212b8
SOURCE="$RUN/canonical-source-4e0f7ad0"
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/canonical-tmp-4e0f7ad0"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
cd "$SOURCE"
export TMPDIR="$RUN/canonical-tmp-4e0f7ad0" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/canonical-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c /run/current-system/sw/bin/bash "$RUN/canonical-checks-4e0f7ad0-inside.sh"
