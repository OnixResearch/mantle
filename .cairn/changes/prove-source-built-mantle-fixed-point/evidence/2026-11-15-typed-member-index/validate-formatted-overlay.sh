#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
NAME=typed-index-formatted-20261115-bhrtxd
SOURCE=/home/brittonr/sbfp-20261113-berdKH/validation-source-typed-index-formatted-20261115.bhrtxd
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
test ! -e "$RUN/qualification-$NAME.log"
exec > "$RUN/qualification-$NAME.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ] && [ "$finished" -ne 1 ]; then status=1; fi; printf "qualification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/qualification-$NAME.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
hostname
test "$("$B3" --no-names "$RUN/incoming/complete-source-8fa84260-aws.tar.gz")" = 29e4717da23d7fc99d05d4b501f3581f58e262454a3ed5d37de0fa8fe002bd00
cd "$SOURCE"
test "$("$B3" --no-names src/source_built_fixed_point_stagex_actions.rs)" = 32937f01d8199b5a83c0a2af692c1755324de87e65d176a801fe1286346d29fd
test "$("$B3" --no-names src/source_built_fixed_point_rust_actions.rs)" = 03347c19dea92ab80d08b0f65df54ec9cb020443b5dda5f62bb6a9dcbc9bf55f
test "$("$B3" --no-names src/source_built_mantle_programs_graph.rs)" = a07836d408247111138dc22066505a55728859e06a20322af1ccd5621bbcd730
test "$("$B3" --no-names src/source_built_mantle_programs_metadata_helper.rs)" = a3e2e1ae870459019462b11832f37827b489fe371856ee61d02b0a33728ce831
source_digest() { tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names; }
SOURCE_BEFORE=$(source_digest)
printf 'validation_source_before_blake3=%s\n' "$SOURCE_BEFORE"
mkdir "$RUN/runner-tmp-$NAME"
export TMPDIR="$RUN/runner-tmp-$NAME" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo -V; cargo fmt --check -p mantle -v; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings; cargo build --release --locked --offline -p mantle --bin mantle'
SOURCE_AFTER=$(source_digest)
printf 'validation_source_after_blake3=%s\n' "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
install -m 755 "$CARGO_TARGET_DIR/release/mantle" "$RUN/bin/mantle-$NAME"
"$B3" "$RUN/bin/mantle-$NAME"
date -Is
finished=1
printf 'qualification_complete=1\n'
