#!/usr/bin/env bash
set -euo pipefail
ROOT=/home/brittonr/sbfp-20261113-berdKH
NAME=869a461d-request-workspace-gpt6sol
RUN=/home/brittonr/sbfp-20261113-berdKH/cold-proof-869a461d-request-workspace-gpt6sol
SOURCE=/home/brittonr/sbfp-20261113-berdKH/runner-source-869a461d-request-workspace
COHORT=/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-869a461d-request-workspace
MANTLE=/home/brittonr/sbfp-20261113-berdKH/bin/mantle-869a461d-request-workspace
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
OUT="$RUN/proof"
REVIEW=/home/brittonr/sbfp-20261113-berdKH/launch-review-acceptance-869a461d-request-workspace-gpt6sol.json
GRAPH_REVIEW=/home/brittonr/sbfp-20261113-berdKH/independent-graph-869a461d-request-workspace-gpt6sol.json
IDENTITY_REVIEW=/home/brittonr/sbfp-20261113-berdKH/independent-physical-869a461d-request-workspace-gpt6sol.json
MODEL_AUTHORIZATION=/home/brittonr/sbfp-20261113-berdKH/independent-model-authorization-869a461d-request-workspace-gpt6sol.json
BWRAP=/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap
SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
# Existing sealed launch-review interface; no output authority before acceptance.
test "$#" -eq 1
EXPECTED_REVIEW_BYTES_BLAKE3="$1"
[[ "$EXPECTED_REVIEW_BYTES_BLAKE3" =~ ^[a-f0-9]{64}$ ]]
test "$("$B3" --no-names "$REVIEW")" = "$EXPECTED_REVIEW_BYTES_BLAKE3"
jq -e --arg cohort "$COHORT" --arg graph "$GRAPH_REVIEW" --arg identity "$IDENTITY_REVIEW" --arg authorization "$MODEL_AUTHORIZATION" '.schema == "mantle-cohort-launch-review-v1" and .code_revision == "869a461d6170404e752865a2029092391240b741" and .source_tree_blake3 == "f1c0468c177aa6ccc0cfed3ea950726123f9528cce0b9e6b7c18229e607fa911" and .runner_blake3 == "ef890d46746969a4010632ea28ad138fa916e6f832007b468e12530007cd7f8b" and .cohort == $cohort and .source_profile_blake3 == "79f9c65041788459714b26dd7dccc3dd6463e864a0b2d4feca703b09760da2ce" and .source_profile_bytes_blake3 == "9d26367b89e60c7b980339ddb4da9400d15c3487720dcdee8e57220680f7c442" and .source_preparation_report_bytes_blake3 == "0fbebd51eb6bf87d5b5b0187385bf3a78f10b409d9b34c4bc1e3f47f60211f46" and .independent_review_model == "underclass/gpt-6-sol" and .graph.accepted == true and .graph.review_path == $graph and (.graph.review_bytes_blake3 | type == "string" and test("^[a-f0-9]{64}$")) and .identity.accepted == true and .identity.review_path == $identity and (.identity.review_bytes_blake3 | type == "string" and test("^[a-f0-9]{64}$")) and .model_authorization.path == $authorization and .model_authorization.bytes_blake3 == "27023e6df44ff65d0b4c53f52f4963706d4a1e51e79908457161a338383f0dba" and .profile_transfer_equal == true and .parent_acceptance == true and (.uncovered_criteria | type == "array" and length == 0)' "$REVIEW"
for scope in graph identity; do
  if [ "$scope" = graph ]; then review_file="$GRAPH_REVIEW"; else review_file="$IDENTITY_REVIEW"; fi
  test "$("$B3" --no-names "$review_file")" = "$(jq -er --arg scope "$scope" '.[$scope].review_bytes_blake3' "$REVIEW")"
done
test "$("$B3" --no-names "$MODEL_AUTHORIZATION")" = 27023e6df44ff65d0b4c53f52f4963706d4a1e51e79908457161a338383f0dba
jq -e '.authorized_review_model == "underclass/gpt-6-sol" and .submitted_value == "approve-gpt-6-sol" and .coverage_reduced == false and .launch_gates_relaxed == false' "$MODEL_AUTHORIZATION"
test ! -e "$RUN"
test ! -L "$RUN"
mkdir "$RUN"
exec > "$RUN/driver.log" 2>&1
record_aliases() {
  for alias in latest latest-source-built-fixed-point .source-built-fixed-point-current; do
    if [ -L "$RUN/$alias" ]; then printf 'alias_%s %s -> %s\n' "$1" "$alias" "$(readlink "$RUN/$alias")";
    elif [ -e "$RUN/$alias" ]; then printf 'alias_%s %s is_not_a_symlink\n' "$1" "$alias";
    else printf 'alias_%s %s absent\n' "$1" "$alias"; fi
  done
}
finished=0
trap 'status=$?; if [ "$status" -eq 0 ] && [ "$finished" -ne 1 ]; then status=1; fi; record_aliases after; printf "cold_proof_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/driver.exit"; exit "$status"' EXIT
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
for phase in preparation qualification canonical-qualification; do test "$(cat "$ROOT/$phase-869a461d-request-workspace.exit")" = 0; done
test "$("$B3" --no-names "$MANTLE")" = ef890d46746969a4010632ea28ad138fa916e6f832007b468e12530007cd7f8b
test "$(tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names)" = f1c0468c177aa6ccc0cfed3ea950726123f9528cce0b9e6b7c18229e607fa911
test "$("$B3" --no-names "$COHORT/source-profile.json")" = 9d26367b89e60c7b980339ddb4da9400d15c3487720dcdee8e57220680f7c442
test "$("$B3" --no-names "$COHORT/source-program-preparation.json")" = 0fbebd51eb6bf87d5b5b0187385bf3a78f10b409d9b34c4bc1e3f47f60211f46
test "$("$B3" --no-names "$BWRAP")" = 9979941d82a18db70c4fa1bed4cd3d33504c1d8b35b9f08b6784e494abbdcece
test "$("$B3" --no-names "$SANDBOX_SHELL")" = ac277a3b87ae477a75067b89de22865222d1d01e1cb7f91ca5f5eeca8e8ef6bc
"$BWRAP" --unshare-all --unshare-user --ro-bind / / -- "$SANDBOX_SHELL" true
PROFILE_ID=79f9c65041788459714b26dd7dccc3dd6463e864a0b2d4feca703b09760da2ce
jq -e --arg profile "$COHORT/source-profile.json" --arg identity "$PROFILE_ID" '.schema == "mantle-source-built-source-programs-v1" and .source_profile == $profile and .output_profile_blake3 == $identity' "$COHORT/source-program-preparation.json"
mkdir "$RUN/home" "$RUN/tmp"
record_aliases before
for alias in latest latest-source-built-fixed-point .source-built-fixed-point-current; do test ! -e "$RUN/$alias"; test ! -L "$RUN/$alias"; done
env -i HOME="$RUN/home" TMPDIR="$RUN/tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json source bundle verify --from "$COHORT/source-profile.json" > "$RUN/source-verification.json"
jq -e '.ready_class == "ready" and ([.missing_records, .stale_records, .unsupported_records, .untrusted_records] | all(length == 0))' "$RUN/source-verification.json"
env -i HOME="$RUN/home" TMPDIR="$RUN/tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json self-build --source-built-fixed-point --source-profile "$COHORT/source-profile.json" --expected-source-profile-blake3 "$PROFILE_ID" --expected-stagex-lineage-blake3 e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d --expected-native-provider-blake3 f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f --proof-bwrap "$BWRAP" --proof-sandbox-shell "$SANDBOX_SHELL" --proof-elapsed-seconds-max 86400 --proof-disk-bytes-max 200000000000 --proof-exec-events-max 524288 --proof-source-records-max 65536 --jobs 16 --out "$OUT" > "$RUN/report.json"
env -i HOME="$RUN/home" TMPDIR="$RUN/tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json bootstrap trust-report --proof-root "$OUT" > "$RUN/trust-report.json"
jq -e '.verified == true and .status == "verified"' "$RUN/trust-report.json"
date -Is
finished=1
printf 'cold_proof_complete=1\n'
