#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/cold-proof-72f6d21f.log" 2>&1
trap 's=$?; printf "cold_proof_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/cold-proof-72f6d21f.exit"; exit "$s"' EXIT
set -x
date -Is
COHORT="$RUN/prepared-cohort-72f6d21f"
MANTLE="$RUN/bin/mantle-frozen"
OUT="$RUN/proof-72f6d21f"
test ! -e "$OUT"
test "$("$B3" --no-names "$MANTLE")" = e7bfd87c096cb569ce6488710ac7e58be5b981e9e6f3d8fd899d68a81ddd9247
test "$("$B3" --no-names "$COHORT/source-profile.json")" = 0b63f4bcacadf36f904559c63793836d2cf17003aad3faeec66f940573139372
test "$("$B3" --no-names "$COHORT/source-program-preparation.json")" = ccb08afa9a223386dd3a8f8853068fb39f75c26f2ad43e2d9b3736b95ac1398a
mkdir -p "$RUN/cold-proof-home" "$RUN/cold-proof-tmp"
for alias in latest latest-source-built-fixed-point .source-built-fixed-point-current; do
  if [ -L "$RUN/$alias" ]; then
    printf 'alias_before %s -> %s\n' "$alias" "$(readlink "$RUN/$alias")"
  elif [ -e "$RUN/$alias" ]; then
    printf 'alias_before %s is_not_a_symlink\n' "$alias"
  else
    printf 'alias_before %s absent\n' "$alias"
  fi
done
df -B1 "$RUN"
hostname
uname -srm
env -i HOME="$RUN/cold-proof-home" TMPDIR="$RUN/cold-proof-tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json self-build --source-built-fixed-point --source-profile "$COHORT/source-profile.json" --expected-source-profile-blake3 47c446b61809bc1f968a065969431af0bb3d7791227e8d18e0bacf95bcad6d77 --expected-stagex-lineage-blake3 e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d --expected-native-provider-blake3 f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f --proof-bwrap /nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap --proof-sandbox-shell /nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox --proof-elapsed-seconds-max 86400 --proof-disk-bytes-max 200000000000 --proof-exec-events-max 524288 --proof-source-records-max 65536 --jobs 16 --out "$OUT" > "$RUN/cold-proof-result-72f6d21f.json"
env -i HOME="$RUN/cold-proof-home" TMPDIR="$RUN/cold-proof-tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json bootstrap trust-report --proof-root "$OUT" > "$RUN/cold-proof-trust-report-72f6d21f.json"
date -Is
