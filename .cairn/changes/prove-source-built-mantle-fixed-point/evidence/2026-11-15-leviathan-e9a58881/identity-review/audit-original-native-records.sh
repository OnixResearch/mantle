#!/usr/bin/env bash
# Stream and compare original native-source record bodies in the published e9 cohort.
# Reads only metadata and the two closure JSON files; writes only to SCRATCH.
set -euo pipefail
umask 077
RUN=/home/brittonr/sbfp-20261113-berdKH
COHORT="$RUN/prepared-cohort-e9a58881-typed-index"
ORIGINAL="$COHORT/native-source-closure.json"
AUGMENTED="$COHORT/inputs/native-source-authority/native-source-closure.json"
PLAN="$COHORT/source-program-preparation-input-plan.json"
REPORT="$COHORT/source-program-preparation.json"
PROFILE="$COHORT/source-profile.json"
SCRATCH=/tmp/luna-e9a58881-identity-preservation-gpt6luna
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
JQ=/run/current-system/sw/bin/jq
PERL=/run/current-system/sw/bin/perl
fail(){ printf 'BLOCKER %s\n' "$*" >&2; exit 3; }
value(){ "$B3" --no-names "$1"; }
record_rows(){
  "$PERL" -MDigest::SHA -ne '
  BEGIN {$inside=0;$count=0;}
  if (!$inside){if (/^  "records": \[/) {$inside=1;} next;}
  if (!$record) {if (/^    \{\s*$/) {$record=1;$hash=Digest::SHA->new(256);$hash->add($_);$id="";next;} last if /^  \],?\s*$/;next;}
  if (/^    \},?\s*$/) {
    $hash->add("    }\n");die "record without identity\n" unless length $id;
    print "$id\t",$hash->hexdigest,"\n"; $record=0;$count++;next;
  }
  $id=$1 if /^      "identity": "([^\"]+)"/;
  $hash->add($_);
  END {die "unterminated or absent record inventory\n" if $record || !$count;}
  ' "$1"
}
[[ -f $ORIGINAL && ! -L $ORIGINAL && -f $AUGMENTED && ! -L $AUGMENTED ]] || fail 'required source closure file is missing or not regular'
[[ -f $PLAN && ! -L $PLAN && -f $REPORT && ! -L $REPORT && -f $PROFILE && ! -L $PROFILE ]] || fail 'plan, report, or profile path is missing or not regular'
mkdir -p "$SCRATCH"
expected_digest=$("$JQ" -r '.source_inputs[]|select(.role=="native-source-bundle")|.digest_blake3' "$PLAN")
expected_size=$("$JQ" -r '.source_inputs[]|select(.role=="native-source-bundle")|.size_bytes' "$PLAN")
report_plan=$("$JQ" -r .input_plan_blake3 "$REPORT")
plan_digest=$("$JQ" -r .plan_digest_blake3 "$PLAN")
input_profile=$("$JQ" -r .input_profile_blake3 "$REPORT")
output_profile=$("$JQ" -r .output_profile_blake3 "$REPORT")
report_profile=$("$JQ" -r .source_profile "$REPORT")
[[ $report_plan == "$plan_digest" ]] || fail 'preparation report does not bind exact source-program input plan'
[[ $input_profile == 5974bc464d7bb0f357a89a60dd18f147413500a8533ccd7d0007644ee987c147 ]] || fail 'current input profile identity differs from pinned review subject'
[[ $output_profile == 3096052305ed9bc2cf0b23b3291dc98153f5b090d7f9abe726368638f883a946 ]] || fail 'current output profile identity differs from pinned review subject'
[[ $report_profile == "$PROFILE" ]] || fail 'preparation report does not identify this profile path'
orig_size=$(stat -c %s "$ORIGINAL")
orig_b3=$(value "$ORIGINAL")
[[ $orig_size == "$expected_size" && $orig_b3 == "$expected_digest" ]] || fail "original native-source bundle differs from plan: size=$orig_size b3=$orig_b3"
profile_size=$(stat -c %s "$PROFILE")
[[ $profile_size == 15790943073 ]] || fail "current output profile byte size differs from supplied immutable subject: $profile_size"
printf 'BINDING input_profile=%s plan_digest=%s output_profile=%s report_b3=%s plan_raw_b3=%s\n' "$input_profile" "$plan_digest" "$output_profile" "$(value "$REPORT")" "$(value "$PLAN")"
printf 'SOURCE_ROLE native-source-bundle path=%s bytes=%s b3=%s profile_bytes=%s profile_raw_b3=parent-stream-parity:%s\n' "$ORIGINAL" "$orig_size" "$orig_b3" "$profile_size" 32b865ac836fa247a6a20b63fd5dd38530c6a455a39dfe84344694be329806cf
record_rows "$ORIGINAL" > "$SCRATCH/original-record-rows.tsv"
record_rows "$AUGMENTED" > "$SCRATCH/augmented-record-rows.tsv"
# Reject duplicate identities in either complete manifest projection.
for rows in "$SCRATCH/original-record-rows.tsv" "$SCRATCH/augmented-record-rows.tsv"; do
  duplicate=$("$PERL" -F'\t' -ane '$seen{$F[0]}++ and print "$F[0]\n"' "$rows")
  [[ -z $duplicate ]] || fail "duplicate source record identities: $duplicate"
done
original_count=$(wc -l < "$SCRATCH/original-record-rows.tsv")
augmented_count=$(wc -l < "$SCRATCH/augmented-record-rows.tsv")
ids='fixed-point-native-source-program fixed-point-rust-source-program source-built-mantle-programs source-built-stagex-programs'
"$PERL" -F'\t' -ane 'BEGIN {@want=split / /, $ARGV[0]; %want=map {$_=>1} @want; shift @ARGV;} $count{$F[0]}++ if $want{$F[0]}; END {for $id (@want) {die "expected exactly one new record $id, got ".($count{$id}//0)."\n" if ($count{$id}//0)!=1; print "NEW_PROGRAM_RECORD $id rows=1\n"}}' "$ids" "$SCRATCH/augmented-record-rows.tsv" > "$SCRATCH/new-program-rows.txt"
[[ $augmented_count -eq $((original_count + 4)) ]] || fail "augmented record count $augmented_count != original $original_count + four program records"
"$PERL" -F'\t' -ane 'BEGIN {@ids=split / /, $ARGV[0]; %ids=map {$_=>1} @ids; shift @ARGV;} print unless $ids{$F[0]}' "$ids" "$SCRATCH/augmented-record-rows.tsv" | sort > "$SCRATCH/augmented-original-rows.sorted.tsv"
sort "$SCRATCH/original-record-rows.tsv" > "$SCRATCH/original-record-rows.sorted.tsv"
diff -u "$SCRATCH/original-record-rows.sorted.tsv" "$SCRATCH/augmented-original-rows.sorted.tsv" || fail 'one or more original native source record bodies changed or disappeared'
cat "$SCRATCH/new-program-rows.txt"
printf 'ORIGINAL_RECORD_BODIES exact_sha256_stream_match=true original_records=%s augmented_records=%s appended_program_records=4 original_json_b3=%s augmented_json_bytes=%s augmented_json_b3=%s\n' "$original_count" "$augmented_count" "$orig_b3" "$(stat -c %s "$AUGMENTED")" "$(value "$AUGMENTED")"
printf 'LIMITATION output-profile raw bytes were not reparsed or rehashed; parent retained full stream transfer parity. No generated program archive, NAR, or embedded payload was inspected by this body-preservation pass.\n'
