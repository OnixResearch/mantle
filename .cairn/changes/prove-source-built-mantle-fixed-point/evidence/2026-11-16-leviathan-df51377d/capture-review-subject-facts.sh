#!/usr/bin/env bash
set -euo pipefail
ROOT=/home/brittonr/sbfp-20261113-berdKH
NAME=df51377d-unit-plans
COHORT="$ROOT/prepared-cohort-$NAME"
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
JQ=/run/current-system/sw/bin/jq
STAT=/run/current-system/sw/bin/stat
MAX_PROFILE_BYTES=21474836480
MAX_METADATA_BYTES=16777216
RESULT="$ROOT/review-subject-facts-$NAME.json"
LOG="$ROOT/review-subject-facts-$NAME.log"
test ! -e "$RESULT"
test ! -e "$LOG"
exec > "$LOG" 2>&1
trap 'status=$?; printf "subject_facts_exit=%s\n" "$status"; printf "%s\n" "$status" > "$ROOT/review-subject-facts-$NAME.exit"; exit "$status"' EXIT
set -x
test "$(cat "$ROOT/preparation-$NAME.exit")" = 0
PROFILE="$COHORT/source-profile.json"
REPORT="$COHORT/source-program-preparation.json"
PLAN="$COHORT/source-program-preparation-input-plan.json"
for path in "$PROFILE" "$REPORT" "$PLAN" "$ROOT/bin/mantle-$NAME" "$ROOT/bin/adopt-source-built-stagex-$NAME" "$ROOT/bin/check-native-row-receipts-$NAME"; do
    test -f "$path"
    test ! -L "$path"
done
PROFILE_BYTES=$("$STAT" -c %s "$PROFILE")
for path in "$REPORT" "$PLAN"; do
    METADATA_BYTES=$("$STAT" -c %s "$path")
    test "$METADATA_BYTES" -gt 0
    test "$METADATA_BYTES" -le "$MAX_METADATA_BYTES"
done
test "$PROFILE_BYTES" -gt 0
test "$PROFILE_BYTES" -le "$MAX_PROFILE_BYTES"
PROFILE_B3=$("$B3" --no-names "$PROFILE")
REPORT_B3=$("$B3" --no-names "$REPORT")
PLAN_B3=$("$B3" --no-names "$PLAN")
RUNNER_B3=$("$B3" --no-names "$ROOT/bin/mantle-$NAME")
ADOPTION_B3=$("$B3" --no-names "$ROOT/bin/adopt-source-built-stagex-$NAME")
CHECKER_B3=$("$B3" --no-names "$ROOT/bin/check-native-row-receipts-$NAME")
"$JQ" -n --arg revision df51377d592bf6a3412b436a10e8fc708e8dbe3d --arg profile "$PROFILE" --argjson profile_bytes "$PROFILE_BYTES" --arg profile_b3 "$PROFILE_B3" --arg report_b3 "$REPORT_B3" --arg plan_b3 "$PLAN_B3" --arg runner_b3 "$RUNNER_B3" --arg adoption_b3 "$ADOPTION_B3" --arg checker_b3 "$CHECKER_B3" --slurpfile preparation "$REPORT" --slurpfile plan "$PLAN" '{source_code_revision:$revision,profile_file:$profile,profile_file_bytes:$profile_bytes,profile_file_raw_blake3:$profile_b3,preparation_report_raw_blake3:$report_b3,input_plan_raw_blake3:$plan_b3,runner_blake3:$runner_b3,adoption_operator_blake3:$adoption_b3,native_row_checker_blake3:$checker_b3,preparation:$preparation[0],input_plan:$plan[0],non_claim:"Exact current prepared-data and executable observations only; no independent review, protected execution, fixed point or native-row acceptance."}' > "$RESULT"
printf 'subject_facts_complete=1\n'
