#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/cold-proof-4e0f7ad0.log" 2>&1
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
trap 's=$?; record_aliases after; printf "cold_proof_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/cold-proof-4e0f7ad0.exit"; exit "$s"' EXIT
set -x
date -Is
COHORT="$RUN/prepared-cohort-4e0f7ad0"
MANTLE="$RUN/bin/mantle-4e0f7ad0"
OUT="$RUN/proof-4e0f7ad0"
test ! -e "$OUT"
test ! -L "$OUT"
test "$(cat "$RUN/preparation-4e0f7ad0.exit")" = 0
test "$("$B3" --no-names "$MANTLE")" = 57042f2b8048afd51e1151749a9d01aa63b99e958bbd151e035c3a62816abadf
test "$("$B3" --no-names "$COHORT/source-profile.json")" = ffcc050da54bb5a05720e768be84e395759f6b0275e72566a86f3bce3189f3ec
test "$("$B3" --no-names "$COHORT/source-program-preparation.json")" = 00aa1dc381cec5251d8f8853a9c44e8548afa1ad6bba11ac199ec9b66b13cd1a
test "$("$B3" --no-names /nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap)" = 9979941d82a18db70c4fa1bed4cd3d33504c1d8b35b9f08b6784e494abbdcece
test "$("$B3" --no-names /nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox)" = ac277a3b87ae477a75067b89de22865222d1d01e1cb7f91ca5f5eeca8e8ef6bc
mkdir "$RUN/cold-proof-4e0f7ad0-home" "$RUN/cold-proof-4e0f7ad0-tmp"
record_aliases before
df -B1 "$RUN"
hostname
uname -srm
env -i HOME="$RUN/cold-proof-4e0f7ad0-home" TMPDIR="$RUN/cold-proof-4e0f7ad0-tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json self-build --source-built-fixed-point --source-profile "$COHORT/source-profile.json" --expected-source-profile-blake3 4932432f6fddd92e0ecefcb96a4aff872e19af3d2e507f8d6112d1c0251a8e51 --expected-stagex-lineage-blake3 e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d --expected-native-provider-blake3 f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f --proof-bwrap /nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap --proof-sandbox-shell /nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox --proof-elapsed-seconds-max 86400 --proof-disk-bytes-max 200000000000 --proof-exec-events-max 524288 --proof-source-records-max 65536 --jobs 16 --out "$OUT" > "$RUN/cold-proof-result-4e0f7ad0.json"
env -i HOME="$RUN/cold-proof-4e0f7ad0-home" TMPDIR="$RUN/cold-proof-4e0f7ad0-tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json bootstrap trust-report --proof-root "$OUT" > "$RUN/cold-proof-trust-report-4e0f7ad0.json"
date -Is
