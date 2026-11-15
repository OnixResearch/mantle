#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
NAME=e9a58881-typed-index
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
COHORT="$RUN/prepared-cohort-$NAME"
MANTLE="$RUN/bin/mantle-$NAME"
OUT="$RUN/proof-$NAME"
REVIEW="$RUN/launch-review-acceptance-$NAME.json"
test ! -e "$RUN/cold-proof-$NAME.log"
exec > "$RUN/cold-proof-$NAME.log" 2>&1
record_aliases() {
 for alias in latest latest-source-built-fixed-point .source-built-fixed-point-current; do
  if [ -L "$RUN/$alias" ]; then
   printf 'alias_%s %s -> %s\n' "$1" "$alias" "$(readlink "$RUN/$alias")"
  elif [ -e "$RUN/$alias" ]; then
   printf 'alias_%s %s is_not_a_symlink\n' "$1" "$alias"
  else
   printf 'alias_%s %s absent\n' "$1" "$alias"
  fi
 done
}
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; record_aliases after; printf "cold_proof_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/cold-proof-$NAME.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
hostname
uname -srm
cat /proc/sys/kernel/random/boot_id
df -B1 "$RUN"
free -b
test ! -e "$OUT"
test ! -L "$OUT"
test "$(cat "$RUN/preparation-$NAME.exit")" = 0
test "$("$B3" --no-names "$MANTLE")" = 76654f00f2c2418e585a1f718da8c8d36ab0d110fbeddc4e69b6dd2dc8116504
test "$("$B3" --no-names "$COHORT/source-profile.json")" = 32b865ac836fa247a6a20b63fd5dd38530c6a455a39dfe84344694be329806cf
test "$("$B3" --no-names "$COHORT/source-program-preparation.json")" = 3c848d614b58613cafb88f2eaf1affad95fa18c642f49eb7b95df4932adc5b96
jq -e '.schema == "mantle-cohort-launch-review-v1" and .code_revision == "e9a58881b9ff491f47864632367c9b8ea4af76e3" and .source_profile_blake3 == "3096052305ed9bc2cf0b23b3291dc98153f5b090d7f9abe726368638f883a946" and .source_profile_bytes_blake3 == "32b865ac836fa247a6a20b63fd5dd38530c6a455a39dfe84344694be329806cf" and .graph.accepted == true and .identity.accepted == true and .profile_transfer_equal == true' "$REVIEW"
test "$("$B3" --no-names /nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap)" = 9979941d82a18db70c4fa1bed4cd3d33504c1d8b35b9f08b6784e494abbdcece
test "$("$B3" --no-names /nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox)" = ac277a3b87ae477a75067b89de22865222d1d01e1cb7f91ca5f5eeca8e8ef6bc
mkdir "$RUN/cold-proof-$NAME-home" "$RUN/cold-proof-$NAME-tmp"
record_aliases before
env -i HOME="$RUN/cold-proof-$NAME-home" TMPDIR="$RUN/cold-proof-$NAME-tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json self-build --source-built-fixed-point --source-profile "$COHORT/source-profile.json" --expected-source-profile-blake3 3096052305ed9bc2cf0b23b3291dc98153f5b090d7f9abe726368638f883a946 --expected-stagex-lineage-blake3 e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d --expected-native-provider-blake3 f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f --proof-bwrap /nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap --proof-sandbox-shell /nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox --proof-elapsed-seconds-max 86400 --proof-disk-bytes-max 200000000000 --proof-exec-events-max 524288 --proof-source-records-max 65536 --jobs 16 --out "$OUT" > "$RUN/cold-proof-result-$NAME.json"
env -i HOME="$RUN/cold-proof-$NAME-home" TMPDIR="$RUN/cold-proof-$NAME-tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json bootstrap trust-report --proof-root "$OUT" > "$RUN/cold-proof-trust-report-$NAME.json"
date -Is
finished=1
printf 'cold_proof_complete=1\n'
