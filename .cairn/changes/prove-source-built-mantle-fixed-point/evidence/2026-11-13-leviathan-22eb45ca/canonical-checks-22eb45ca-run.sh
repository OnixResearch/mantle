#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/canonical-qualification-22eb45ca.log" 2>&1
trap 's=$?; printf "canonical_qualification_22eb45ca_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/canonical-qualification-22eb45ca.exit"; exit "$s"' EXIT
set -x
test "$(cat "$RUN/qualification-22eb45ca.exit")" = 0
ARCHIVE="$RUN/incoming/source-22eb45ca5321416fce3278a228a43224dd0026c5.tar.gz"
test "$("$B3" --no-names "$ARCHIVE")" = ecdd35a1099f16746496811c57d90115e3c57411828f9737173d98ce29cff7db
test "$(gzip --decompress --stdout "$ARCHIVE" | "$B3" --no-names)" = 75622e1209102d6f1cdbaf5187688a05710e0380c9525d18e9d759ea8d93f0f0
SOURCE="$RUN/canonical-source-22eb45ca"
test ! -e "$SOURCE"
mkdir "$SOURCE" "$RUN/canonical-tmp-22eb45ca"
tar --extract --gzip --no-same-owner --file "$ARCHIVE" --directory "$SOURCE"
cd "$SOURCE"
export TMPDIR="$RUN/canonical-tmp-22eb45ca" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/canonical-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c /run/current-system/sw/bin/bash "$RUN/canonical-checks-22eb45ca-inside.sh"
