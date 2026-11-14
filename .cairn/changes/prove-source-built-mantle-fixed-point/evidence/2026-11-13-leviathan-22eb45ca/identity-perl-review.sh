#!/usr/bin/env bash
# Read-only physical program/closure inspection; run:
# ssh leviathan.cymric-daggertooth.ts.net bash -s -- anchors < local://source22-identity-perl-review-script
# Complementary PerlAsm, root traversal, and original Ring source scripts:
# local://source22-perlasm-review-script, local://source22-reachability-review-script,
# local://source22-ring-payload-review-script. No provider, build, generator, or profile parser.
set -euo pipefail
base=/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed
frozen=/home/brittonr/sbfp-20261113-berdKH/runner-source-22eb45ca
jq=/run/current-system/sw/bin/jq
nix=/run/current-system/sw/bin/nix
tar=/run/current-system/sw/bin/tar
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
native="$base/inputs/native-source-authority/native-source-closure.json"
original="$base/native-source-closure.json"
programs="$base/program-sources"
report="$base/source-program-preparation.json"
ids=(fixed-point-native-source-program fixed-point-rust-source-program source-built-mantle-programs source-built-stagex-programs)
fail() { printf 'BLOCKER %s\n' "$*" >&2; exit 1; }
hash_file() { "$b3" "$1" | { read -r value ignored; printf '%s' "$value"; }; }
rec_digest() {
  "$jq" -rj --arg id "$1" '.records[]|select(.identity==$id)|"kind\u0000",(.kind|tojson),"\nmetadata\u0000",(.metadata|tojson),"\nfiles\u0000",(.files[]|tojson,"\n")' "$native" | "$b3" | { read -r value ignored; printf '%s' "$value"; }
}
read_record_lines() {
  "$jq" -r '.records[]|[.identity,.content_blake3]|@tsv' "$original"
}
read_extended_record_lines() {
  "$jq" -r '.records[]|select(.identity|IN("fixed-point-native-source-program","fixed-point-rust-source-program","source-built-mantle-programs","source-built-stagex-programs")|not)|[.identity,.content_blake3]|@tsv' "$native"
}
anchors() {
  printf 'ANCHOR frozen-source-revision=22eb45ca5321416fce3278a228a43224dd0026c5\n'
  for file in "$report" "$base/source-program-preparation-input-plan.json" "$original" "$native" "$base/inputs/mantle-source/bootstrap/source-built-cold-route.json" "$base/inputs/mantle-source/bootstrap/source-built/mantle-stage1.json" "$base/inputs/mantle-source/bootstrap/source-built/mantle-stage2.json" "$base/inputs/stagex-seed/hex0-seed" "$base/inputs/stagex-lineage/stagex-transition-lineage.json" "$base/inputs/stagex-source-bundle/stagex-source-closure.json"; do
    printf 'RAW_BLAKE3 %s %s\n' "${file#"$base"/}" "$(hash_file "$file")"
  done
  diff -u <(read_record_lines) <(read_extended_record_lines) || fail 'older 59 native records changed during append of four programs'
  printf 'PRESERVED native original record identities and content hashes=59\n'
  for id in "${ids[@]}"; do
    row=$("$jq" -r --arg id "$id" '.records[]|select(.identity==$id)|[.identity,.content_blake3,.payload_bytes,.metadata.hash,.metadata.url,(.files|length)]|@tsv' "$native")
    [[ -n $row ]] || fail "missing native closure record $id"
    IFS=$'\t' read -r name digest payload nar url count <<< "$row"
    expected=$("$jq" -r --arg id "$id" '.program_records[$id] // empty' "$report")
    [[ $expected == "$digest" ]] || fail "$id report/record content digest mismatch"
    [[ $url == "file://$programs/$id.tar" ]] || fail "$id archive URL not exact published payload"
    content=$(rec_digest "$id")
    [[ $content == "$digest" ]] || fail "$id canonical record digest $content != $digest"
    measured=$("$nix" --extra-experimental-features blake3-hashes hash path --type blake3 --base16 "$programs/$id")
    [[ $measured == "$nar" ]] || fail "$id recursive NAR $measured != metadata $nar"
    printf 'RECORD %s %s payload_bytes=%s entry_count=%s recursive_blake3=%s archive_blake3=%s archive_bytes=%s\n' "$id" "$digest" "$payload" "$count" "$measured" "$(hash_file "$programs/$id.tar")" "$(stat -c %s "$programs/$id.tar")"
    actual_entries=$("$tar" -tf "$programs/$id.tar" | wc -l)
    expected_entries=$("$jq" -r --arg id "$id" '.records[]|select(.identity==$id)|.files[].path' "$native" | sort -u | wc -l)
    [[ $actual_entries -eq $expected_entries ]] || fail "$id archive has $actual_entries members but record has $expected_entries unique members"
    diff -u <("$jq" -r --arg id "$id" '.records[]|select(.identity==$id)|.files[].path' "$native" | sort -u) <("$tar" -tf "$programs/$id.tar" | while IFS= read -r member; do [[ $member == source/* ]] || fail "unsafe archive member $member"; printf '%s\n' "${member#source/}"; done | sort -u) || fail "$id archive member list differs from declared payload"
    from_tar=$("$tar" -xOf "$programs/$id.tar" | "$b3" | { read -r value ignored; printf '%s' "$value"; })
    from_files=$("$tar" -tf "$programs/$id.tar" | while IFS= read -r member; do cat "$programs/$id/${member#source/}"; done | "$b3" | { read -r value ignored; printf '%s' "$value"; })
    [[ $from_tar == "$from_files" ]] || fail "$id archive content stream differs from physical file stream"
    printf 'ARCHIVE_CONTENT %s matched physical files, concatenated_blake3=%s unique_members=%s\n' "$id" "$from_files" "$expected_entries"
  done
  file_count=0
  chunk_count=0
  while IFS=$'\t' read -r id path size digest index chunks hexlength; do
    localfile="$programs/$id/$path"
    [[ -f $localfile && ! -L $localfile ]] || fail "missing/nonregular $id/$path"
    [[ "$hexlength" -eq "$((size*2))" ]] || fail "embedded source payload length mismatch $id/$path:$index"
    if ((chunks > 0)); then
      [[ $index -ge 0 && $index -lt $chunks ]] || fail "invalid source chunk index $id/$path:$index"
      observed=$(dd if="$localfile" bs=67108864 skip="$index" count=1 status=none | "$b3" | { read -r value ignored; printf '%s' "$value"; })
      chunk_count=$((chunk_count+1))
    else
      [[ "$(stat -c %s "$localfile")" -eq "$size" ]] || fail "physical file length mismatch $id/$path"
      observed=$(hash_file "$localfile")
    fi
    [[ $observed == "$digest" ]] || fail "physical source bytes mismatch record $id/$path:$index"
    file_count=$((file_count+1))
  done < <("$jq" -r '.records[]|select(.identity|IN("fixed-point-native-source-program","fixed-point-rust-source-program","source-built-mantle-programs","source-built-stagex-programs"))|.identity as $id|.files[]|[$id,.path,.size,.blake3,(.chunk_index//-1),(.chunk_count//0),(.content_hex|length)]|@tsv' "$native")
  [[ $file_count -eq 479 && $chunk_count -eq 4 ]] || fail "expected 479 verified record entries/4 chunks, observed $file_count/$chunk_count"
  printf 'FILE_ENTRIES verified_b3sum_and_length=%s rust_chunks=%s\n' "$file_count" "$chunk_count"
  for filename in source-built/rust/program.json cairn/rust/compiler-environment.rs; do
    cmp -s "$programs/fixed-point-rust-source-program/$filename" "$base/inputs/rust-source-archives/$filename" || fail "Rust authority diverges from authored record $filename"
  done
  printf 'RUST_AUTHORITY copied authored recipe and compiler environment byte-identically\n'
  cmp -s "$frozen/src/source_built_mantle_programs_aws_helper.rs" "$programs/source-built-mantle-programs/aws-source-helper.rs" || fail 'AWS helper differs from frozen include_str source'
  cmp -s "$frozen/src/source_built_mantle_programs_native_helper.rs" "$programs/source-built-mantle-programs/source_built_mantle_native_helper.rs" || fail 'Ring native helper differs from frozen include_str source'
  printf 'FROZEN_HELPERS AWS=%s native=%s exact byte copies\n' "$(hash_file "$frozen/src/source_built_mantle_programs_aws_helper.rs")" "$(hash_file "$frozen/src/source_built_mantle_programs_native_helper.rs")"
}
case "${1:-}" in
 anchors) anchors ;;
 *) printf 'usage: %s anchors\n' "$0" >&2; exit 2 ;;
esac
