#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
NAME=787784fb-helper-feature
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
SOURCE="$RUN/runner-source-787784fb-helper-feature"
COHORT="$RUN/prepared-cohort-$NAME"
MANTLE="$RUN/bin/mantle-$NAME"
OUT="$RUN/proof-$NAME"
REVIEW="$RUN/launch-review-acceptance-$NAME.json"
BWRAP=/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap
SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
test "$#" -eq 1
EXPECTED_REVIEW_BYTES_BLAKE3="$1"
[[ "$EXPECTED_REVIEW_BYTES_BLAKE3" =~ ^[a-f0-9]{64}$ ]]
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
test "$(cat "$RUN/qualification-$NAME.exit")" = 0
test "$(cat "$RUN/canonical-qualification-$NAME.exit")" = 0
test "$("$B3" --no-names "$REVIEW")" = "$EXPECTED_REVIEW_BYTES_BLAKE3"
jq -e --arg cohort "$COHORT" '.schema == "mantle-cohort-launch-review-v1" and .code_revision == "787784fbb6bf2141fce5aa2be9f1741ac9f2f013" and .source_tree_blake3 == "dcfcf2de5825e24aec564fee9f3bc064a50965046c86013eac7a3ff4edd4ec0c" and .runner_blake3 == "6b7c840975dd68631c267458f3360ce46f84fe839af176aa077fb4d9bdccf7ef" and .cohort == $cohort and (.source_profile_blake3 | type == "string" and test("^[a-f0-9]{64}$")) and (.source_profile_bytes_blake3 | type == "string" and test("^[a-f0-9]{64}$")) and (.source_preparation_report_bytes_blake3 | type == "string" and test("^[a-f0-9]{64}$")) and .graph.accepted == true and .identity.accepted == true and .profile_transfer_equal == true and .parent_acceptance == true and (.uncovered_criteria | type == "array" and length == 0)' "$REVIEW"
test "$("$B3" --no-names "$MANTLE")" = 6b7c840975dd68631c267458f3360ce46f84fe839af176aa077fb4d9bdccf7ef
test "$(tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names)" = dcfcf2de5825e24aec564fee9f3bc064a50965046c86013eac7a3ff4edd4ec0c
test "$("$B3" --no-names "$COHORT/source-profile.json")" = "$(jq -er .source_profile_bytes_blake3 "$REVIEW")"
test "$("$B3" --no-names "$COHORT/source-program-preparation.json")" = "$(jq -er .source_preparation_report_bytes_blake3 "$REVIEW")"
PROFILE_ID=$(jq -er .source_profile_blake3 "$REVIEW")
jq -e --arg profile "$COHORT/source-profile.json" --arg identity "$PROFILE_ID" '.schema == "mantle-source-built-source-programs-v1" and .source_profile == $profile and .output_profile_blake3 == $identity' "$COHORT/source-program-preparation.json"
jq -e '.ready_class == "ready" and ([.missing_records, .stale_records, .unsupported_records, .untrusted_records] | all(length == 0))' "$RUN/prepared-source-profile-verification-$NAME.json"
test "$("$B3" --no-names "$BWRAP")" = 9979941d82a18db70c4fa1bed4cd3d33504c1d8b35b9f08b6784e494abbdcece
test "$("$B3" --no-names "$SANDBOX_SHELL")" = ac277a3b87ae477a75067b89de22865222d1d01e1cb7f91ca5f5eeca8e8ef6bc
mkdir "$RUN/cold-proof-$NAME-home" "$RUN/cold-proof-$NAME-tmp"
record_aliases before
env -i HOME="$RUN/cold-proof-$NAME-home" TMPDIR="$RUN/cold-proof-$NAME-tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json self-build --source-built-fixed-point --source-profile "$COHORT/source-profile.json" --expected-source-profile-blake3 "$PROFILE_ID" --expected-stagex-lineage-blake3 e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d --expected-native-provider-blake3 f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f --proof-bwrap "$BWRAP" --proof-sandbox-shell "$SANDBOX_SHELL" --proof-elapsed-seconds-max 86400 --proof-disk-bytes-max 200000000000 --proof-exec-events-max 524288 --proof-source-records-max 65536 --jobs 16 --out "$OUT" > "$RUN/cold-proof-report-$NAME.json"
env -i HOME="$RUN/cold-proof-$NAME-home" TMPDIR="$RUN/cold-proof-$NAME-tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json bootstrap trust-report --proof-root "$OUT" > "$RUN/cold-proof-trust-report-$NAME.json"
date -Is
finished=1
printf 'cold_proof_complete=1\n'
