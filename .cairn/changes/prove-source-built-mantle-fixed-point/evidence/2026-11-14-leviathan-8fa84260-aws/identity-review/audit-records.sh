#!/usr/bin/env bash
# Read-only, streaming original-record preservation and bounded one-program-at-a-time inspections.
set -euo pipefail
r=/home/brittonr/sbfp-20261113-berdKH
base="$r/prepared-cohort-8fa84260-aws"
orig="$base/native-source-closure.json"
native="$base/inputs/native-source-authority/native-source-closure.json"
report="$base/source-program-preparation.json"
plan="$base/source-program-preparation-input-plan.json"
programs="$base/program-sources"
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
jq=/run/current-system/sw/bin/jq
tar=/run/current-system/sw/bin/tar
nix=/run/current-system/sw/bin/nix
ids=(fixed-point-native-source-program fixed-point-rust-source-program source-built-mantle-programs source-built-stagex-programs)
fail(){ printf 'BLOCKER %s\n' "$*" >&2; exit 3; }
value(){ "$b3" --no-names "$1"; }
record_rows() {
  perl -MDigest::SHA -ne '
  BEGIN {$inside=0;$count=0;}
  if (!$inside){if (/^  "records": \[/) {$inside=1;} next;}
  if (!$record) {if (/^    \{\s*$/) {$record=1;$hash=Digest::SHA->new(256);$hash->add($_);$id="";next;} last if /^  \],?\s*$/;next;}
  if (/^    \},?\s*$/) {
    $hash->add("    }\n");die "record without identity\n" unless length $id;
    print "$id\t",$hash->hexdigest,"\n"; $record=0;$count++;next;
  }
  $id=$1 if /^      "identity": "([^"]+)"/;
  $hash->add($_);
  END {die "unterminated or absent record inventory\n" if $record || !$count;}
  ' "$1"
}
record_json() {
  perl -ne '
  BEGIN {$wanted=shift @ARGV;$inside=0;$matched=0;$found=0;}
  if (!$inside){if (/^  "records": \[/) {$inside=1;}next;}
  if (!$record){if (/^    \{\s*$/) {$record=1;$head=$_;$matched=0;}next;}
  if (/^    \},?\s*$/) {if($matched){print "}\n";$found++;}$record=0;$head="";next;}
  if (!$matched && /^      "identity": "([^"]+)"/ && $1 eq $wanted) {$matched=1;print $head;}
  if ($matched) {print $_;} elsif (length $head) {$head.=$_ if length($head)<4096;}
  END {die "expected exactly one record $wanted, got $found\n" if $found!=1;}
  ' "$1" "$2"
}
[[ $(value "$orig") == $("$jq" -r '.source_inputs[]|select(.role=="native-source-bundle")|.digest_blake3' "$plan") ]] || fail 'original native manifest raw hash differs from input plan'
[[ $("$jq" -r .plan_digest_blake3 "$plan") == $("$jq" -r .input_plan_blake3 "$report") ]] || fail 'report/plan identity differs'
[[ $("$jq" -r .source_profile "$report") == "$base/source-profile.json" ]] || fail 'report source profile path differs'
printf 'RAW original_native=%s extended_native=%s report=%s input_plan=%s\n' "$(value "$orig")" "$(value "$native")" "$(value "$report")" "$(value "$plan")"
printf 'BINDING plan=%s input_profile=%s output_profile=%s\n' "$("$jq" -r .plan_digest_blake3 "$plan")" "$("$jq" -r .input_profile_blake3 "$report")" "$("$jq" -r .output_profile_blake3 "$report")"
# Each original record's entire pretty-JSON object is SHA-256 hashed while streaming; only its final array comma is normalized.
# Sort small identity/hash projections; never materialize either ~1 GB native manifest in memory.
diff -u <(record_rows "$orig" | sort) <(record_rows "$native" | perl -ne 'print unless /^(?:fixed-point-native-source-program|fixed-point-rust-source-program|source-built-mantle-programs|source-built-stagex-programs)\t/' | sort) || fail 'an original native record changed during append'
printf 'PRESERVED original_native_records_full_byte_identity=true\n'
for id in "${ids[@]}"; do
  got=$(record_json "$id" "$native" | "$jq" -r .content_blake3)
  expected=$("$jq" -r --arg id "$id" '.program_records[$id]' "$report")
  [[ $expected == "$got" && $got =~ ^[0-9a-f]{64}$ ]] || fail "$id report/record digest mismatch"
  canonical=$(record_json "$id" "$native" | "$jq" -rj '"kind\u0000",(.kind|tojson),"\nmetadata\u0000",(.metadata|tojson),"\nfiles\u0000",(.files[]|tojson,"\n")' | "$b3" --no-names)
  [[ $canonical == "$got" ]] || fail "$id canonical source-record framing differs: $canonical"
  row=$(record_json "$id" "$native" | "$jq" -r '[.identity,.kind,.metadata.hash,.metadata.url,.payload_bytes,(.files|length),([.files[].path]|unique|length)]|@tsv')
  IFS=$'\t' read -r name kind nar url payload entries unique <<< "$row"
  [[ $name == "$id" && $kind == fixed-url && $url == "file://$programs/$id.tar" ]] || fail "$id fixed-url metadata does not identify actual program archive"
  measured=$("$nix" --extra-experimental-features blake3-hashes hash path --type blake3 --base16 "$programs/$id")
  [[ $measured == "$nar" ]] || fail "$id physical NAR hash differs from record metadata: $measured"
  tar_count=$("$tar" -tf "$programs/$id.tar" | wc -l)
  [[ $tar_count -eq $unique ]] || fail "$id archive member count mismatch $tar_count/$unique"
  diff -u <(record_json "$id" "$native" | "$jq" -r '.files[].path' | sort -u) <("$tar" -tf "$programs/$id.tar" | perl -ne 'chomp;die "unsafe archive member $_\n" unless m{^source/[^/]} && !m{(?:^|/)\.\.(?:/|$)};s{^source/}{};print "$_\n";' | sort -u) || fail "$id archive members differ from declared members"
  checked=0; chunks=0; physicalbytes=0
  while IFS=$'\t' read -r path size digest index count hexlen filetype executable; do
    target="$programs/$id/$path"
    [[ -f $target && ! -L $target && $filetype == regular && $executable == false && $hexlen -eq $((size*2)) ]] || fail "$id/$path:$index missing, wrong type, or invalid embedded length"
    [[ $(stat -c %a "$target") == 644 ]] || fail "$id/$path has nonstandard physical mode"
    if (( count > 0 )); then
      [[ $index -ge 0 && $index -lt $count ]] || fail "$id/$path:$index invalid chunk index"
      observed=$(dd if="$target" bs=67108864 skip="$index" count=1 status=none | "$b3" --no-names)
      chunks=$((chunks+1))
    else
      [[ $(stat -c %s "$target") -eq $size ]] || fail "$id/$path physical length differs"
      observed=$(value "$target")
    fi
    [[ $observed == "$digest" ]] || fail "$id/$path:$index physical BLAKE3 differs"
    physicalbytes=$((physicalbytes+size)); checked=$((checked+1))
  done < <(record_json "$id" "$native" | "$jq" -r '.files[]|[.path,.size,.blake3,(.chunk_index//-1),(.chunk_count//0),(.content_hex|length),.file_type,.executable]|@tsv')
  [[ $checked -eq $entries && $physicalbytes -eq $payload ]] || fail "$id verified entry count/total bytes differ: $checked/$entries $physicalbytes/$payload"
  tar_digest=$("$tar" -xOf "$programs/$id.tar" | "$b3" --no-names)
  file_digest=$("$tar" -tf "$programs/$id.tar" | while IFS= read -r member; do cat "$programs/$id/${member#source/}"; done | "$b3" --no-names)
  [[ $tar_digest == "$file_digest" ]] || fail "$id archive bytes differ from physical members"
  printf 'PROGRAM %s record=%s kind=%s payload_bytes=%s file_entries=%s unique_files=%s chunks=%s NAR=%s archive_b3=%s archive_bytes=%s concatenated_b3=%s\n' "$id" "$got" "$kind" "$payload" "$checked" "$unique" "$chunks" "$nar" "$(value "$programs/$id.tar")" "$(stat -c %s "$programs/$id.tar")" "$tar_digest"
done
printf 'SOURCE_ROLES %s\n' "$("$jq" -r '[.source_inputs[].role]|join(",")' "$plan")"
printf 'NON_CLAIM %s\n' "$("$jq" -r .non_claim "$report")"
