#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/canonical-qualification-4653ddef.log" 2>&1
trap 's=$?; printf "canonical_qualification_4653ddef_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/canonical-qualification-4653ddef.exit"; exit "$s"' EXIT
set -x
test "$(cat "$RUN/qualification-4653ddef.exit")" = 0
ARCHIVE="$RUN/incoming/source-4653ddef4edd196620f0a3337b2c7045c8cf4efd.tar.gz"
test "$("$B3" --no-names "$ARCHIVE")" = 0030d50e5c78a90d325e969eec8848174c06cacbae561f1a3768a905c8792c14
test "$(gzip --decompress --stdout "$ARCHIVE" | "$B3" --no-names)" = 75845ae712c685ca9a9aa2c2120c5f13934551845b65de9b329aec2868fae4a4
SOURCE="$RUN/canonical-source-4653ddef"
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/canonical-tmp-4653ddef"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
cd "$SOURCE"
export TMPDIR="$RUN/canonical-tmp-4653ddef" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/canonical-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c /run/current-system/sw/bin/bash "$RUN/canonical-checks-4653ddef-inside.sh"
