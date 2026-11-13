#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/preparation-07ccbf78.log" 2>&1
trap 's=$?; printf "preparation_07ccbf78_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/preparation-07ccbf78.exit"; exit "$s"' EXIT
set -x
date -Is
test "$(cat "$RUN/qualification-07ccbf78.exit")" = 0
"$B3" --check "$RUN/runner-blake3-07ccbf78.txt"
SOURCE="$RUN/runner-source-07ccbf78"
COHORT="$RUN/prepared-cohort-07ccbf78"
MANTLE="$RUN/bin/mantle-07ccbf78"
cd "$SOURCE"
mkdir -p "$RUN/preparation-07ccbf78-home" "$RUN/preparation-07ccbf78-tmp"
test ! -e "$COHORT"
PROOF_HOME="$RUN/preparation-07ccbf78-home"
PROOF_TMP="$RUN/preparation-07ccbf78-tmp"
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json source bundle refresh-mantle-source --from "$RUN/incoming/source-profile-baseline.json" --mantle-source "$SOURCE" --to "$RUN/source-profile-07ccbf78.json" > "$RUN/source-refresh-07ccbf78.json"
jq -e '.input_manifest_blake3 == "de163b8deb0b14afd58469f0c66f8dc62714117b4706323d5e419cf37cd03c6c" and .previous_mantle_source_blake3 == "51b3da0e5d3a15f690aec40a41763ff7b656eee03b2e399d97b5d705e6005194" and .preserved_record_count == 65 and .added_record_count == 0' "$RUN/source-refresh-07ccbf78.json"
PROFILE_ID=$(jq -er .output_manifest_blake3 "$RUN/source-refresh-07ccbf78.json")
[[ "$PROFILE_ID" =~ ^[a-f0-9]{64}$ ]]
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json source bundle verify --from "$RUN/source-profile-07ccbf78.json" > "$RUN/source-profile-verification-07ccbf78.json"
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json self-build --source-built-fixed-point --prepare-source-programs --source-profile "$RUN/source-profile-07ccbf78.json" --expected-source-profile-blake3 "$PROFILE_ID" --expected-stagex-lineage-blake3 e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d --expected-native-provider-blake3 f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f --proof-bwrap /nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap --proof-sandbox-shell /nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox --proof-elapsed-seconds-max 86400 --proof-disk-bytes-max 200000000000 --proof-exec-events-max 524288 --proof-source-records-max 65536 --jobs 16 --out "$COHORT" > "$RUN/preparation-report-07ccbf78.json"
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json source bundle verify --from "$COHORT/source-profile.json" > "$RUN/prepared-source-profile-verification-07ccbf78.json"
jq -e '.ready_class == "ready" and ([.missing_records, .stale_records, .unsupported_records, .untrusted_records] | all(length == 0))' "$RUN/prepared-source-profile-verification-07ccbf78.json"
"$B3" "$RUN/source-profile-07ccbf78.json" "$COHORT/source-profile.json" "$COHORT/source-program-preparation.json"
date -Is
