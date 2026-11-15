#!/usr/bin/env bash
# Bounded read-only projection of current stage1/stage2 selected PerlAsm sources.
set -euo pipefail
r=/home/brittonr/sbfp-20261113-berdKH
test ! -e "$r/selected-source-inspection-8fa84260-aws-qualified.log"
exec > "$r/selected-source-inspection-8fa84260-aws-qualified.log" 2>&1
base="$r/prepared-cohort-8fa84260-aws"
original="$base/inputs/mantle-source/bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9"
ring="$base/inputs/vendor-deps/ring"
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
jq=/run/current-system/sw/bin/jq
WORK=$(mktemp -d /dev/shm/mantle-selected-source-inspection.XXXXXX)
scratch="$WORK/selection"
fail(){ printf 'BLOCKER %s\n' "$*" >&2; exit 3; }
value(){ "$b3" --no-names "$1"; }
trap 'status=$?; rm -f "$scratch.stage1.json" "$scratch.stage2.json" "$scratch.aws1.json" "$scratch.aws2.json" "$scratch.ring1.json" "$scratch.ring2.json"; rmdir "$WORK"; printf "inspection_exit=%s\n" "$status"; printf "%s\n" "$status" > "$r/selected-source-inspection-8fa84260-aws-qualified.exit"; exit "$status"' EXIT
for n in 1 2; do
  file="$base/inputs/mantle-source/bootstrap/source-built/mantle-stage$n.json"
  "$jq" -c '
    . as $doc | (.steps|INDEX(.id)) as $m |
    {
      aws: ([.steps[] | select(.id|test("^unit-00037-script[.]perlasm-[0-9]+[.]generate$")) |
        [.id,.inputs["original-script"].action_id,.operation.arguments[2].parts[0].path,
         .operation.arguments[4].parts[0].text]] | sort_by(.[0]|capture("perlasm-(?<n>[0-9]+)").n|tonumber)),
      ring: ([.steps[] | select(.id|test("^unit-00031-script[.]sources[.]perl-[0-9]+[.]authenticate$")) as $a |
        ($a.id|sub("[.]authenticate$";"")) as $p | $m[$p+".generate"] as $g |
        [$a.id,$a.inputs.request.kind,$a.inputs.request.action_id,$a.inputs.package.kind,
         $a.inputs.package.role,$a.inputs.package.path,$a.operation.arguments[3].parts[0].path,
         $a.operation.arguments[4].parts[0].text,$g.inputs["original-script"].action_id]] |
        sort_by(.[0]|capture("perl-(?<n>[0-9]+)").n|tonumber))
    }
  ' "$file" > "$scratch.stage$n.json"
  [[ $("$jq" -r '.aws|length' "$scratch.stage$n.json") -eq 26 ]] || fail "stage$n AWS generator count differs"
  [[ $("$jq" -r '.ring|length' "$scratch.stage$n.json") -eq 12 ]] || fail "stage$n Ring authentication count differs"
  "$jq" -e --arg owner "mantle-stage$n/" '(.aws | all(.[]; .[1] | startswith($owner))) and (.ring | all(.[]; (.[2] | startswith($owner)) and (.[8] | startswith($owner))))' "$scratch.stage$n.json" > /dev/null || fail "stage$n selected source producer belongs to another stage"
  printf 'CURRENT_STAGE stage=%s raw_b3=%s bytes=%s aws_generators=26 ring_authentications=12\n' "$n" "$(value "$file")" "$(stat -c %s "$file")"
done
for family in aws ring; do
  "$jq" -S --arg family "$family" '.[$family] | map(map(if type == "string" then sub("^mantle-stage[12]/"; "") else . end))' "$scratch.stage1.json" > "$scratch.$family"1.json
  "$jq" -S --arg family "$family" '.[$family] | map(map(if type == "string" then sub("^mantle-stage[12]/"; "") else . end))' "$scratch.stage2.json" > "$scratch.$family"2.json
  cmp -s "$scratch.$family"1.json "$scratch.$family"2.json || fail "stage1/stage2 $family selections differ"
done
printf 'STAGE_SELECTION_PARITY aws=26 ring=12 validated stage-local tuples identical across stage1/stage2\n'
while IFS=$'\t' read -r id source_action source_member output; do
  [[ $source_member == crypto/* && $source_member == *.pl && $source_member != /* ]] || fail "unsafe AWS original member $source_member"
  case "/$source_member/" in *"/../"*) fail "AWS path traversal $source_member";; esac
  [[ $source_action == mantle-stage1/unit-00037-script.perlasm-source ]] || fail "AWS original is not selected source-tree output: $source_action"
  [[ $output == *.S && $output != */* ]] || fail "AWS output is not an assembly basename $output"
  path="$original/$source_member"
  [[ -f $path && ! -L $path ]] || fail "missing/nonregular AWS original $source_member"
  style=$(/nix/store/phnk1lwy8xs0yrbrcs6l2mb9yr9c2knp-perl-5.42.0/bin/perl -ne '
    BEGIN{$standalone=0;$guarded=0;$bad=0;}
    chomp;my $norm=$_;$norm=~s/^open OUT,\s*/open OUT,/;$norm=~s/\s+$//;
    if($norm eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\"";}){$standalone++;}
    if($norm eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\""}){my $continuation=<>//"";$continuation=~s/^\s+|\s+$//g;$guarded++ if $continuation eq (q{or die "can}.chr(39).q{t call $xlate: $!";});$bad++ if $continuation ne (q{or die "can}.chr(39).q{t call $xlate: $!";});}
    END{die "unexpected original pipe cut\n" unless $standalone+$guarded==1 && !$bad;print $guarded ? "guarded" : "single-line";}
  ' "$path") || fail "AWS complete boundary check failed: $source_member"
  printf 'AWS_ORIGINAL %s %s -> %s bytes=%s blake3=%s boundary=%s\n' "$id" "$source_member" "$output" "$(stat -c %s "$path")" "$(value "$path")" "$style"
done < <("$jq" -r '.aws[]|@tsv' "$scratch.stage1.json")
while IFS=$'\t' read -r id request_kind request_action package_kind role package_path member output auth_action; do
  [[ $request_kind == produced && -n $request_action ]] || fail "Ring source request is not a produced typed item $id"
  [[ $package_kind == source && $role == vendor-inputs && $package_path == ring ]] || fail "Ring source role is not vendor-inputs/ring $id"
  [[ $auth_action == "mantle-stage1/$id" ]] || fail "Ring source did not consume selected authenticated file output $id"
  [[ $member == crypto/* && $member == *.pl && $member != /* ]] || fail "unsafe Ring original member $member"
  case "/$member/" in *"/../"*) fail "Ring path traversal $member";; esac
  [[ $output == *.S && $output != */* ]] || fail "Ring output is not an assembly basename $output"
  path="$ring/$member"
  [[ -f $path && ! -L $path ]] || fail "missing/nonregular Ring original $member"
  expected=$("$jq" -r --arg p "$member" '.files[$p] // empty' "$ring/.cargo-checksum.json")
  observed=$(sha256sum "$path" | cut -d ' ' -f1)
  [[ $expected =~ ^[0-9a-f]{64}$ && $observed == "$expected" ]] || fail "Ring package checksum mismatch $member"
  style=$(/nix/store/phnk1lwy8xs0yrbrcs6l2mb9yr9c2knp-perl-5.42.0/bin/perl -ne '
    BEGIN{$standalone=0;$guarded=0;$bad=0;}
    chomp;my $norm=$_;$norm=~s/^open OUT,\s*/open OUT,/;$norm=~s/\s+$//;
    if($norm eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\"";}){$standalone++;}
    if($norm eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\""}){my $continuation=<>//"";$continuation=~s/^\s+|\s+$//g;$guarded++ if $continuation eq (q{or die "can}.chr(39).q{t call $xlate: $!";});$bad++ if $continuation ne (q{or die "can}.chr(39).q{t call $xlate: $!";});}
    END{die "unexpected original pipe cut\n" unless $standalone+$guarded==1 && !$bad;print $guarded ? "guarded" : "single-line";}
  ' "$path") || fail "Ring complete boundary check failed: $member"
  printf 'RING_ORIGINAL %s %s -> %s request=%s bytes=%s sha256=%s blake3=%s boundary=%s\n' "$id" "$member" "$output" "$request_action" "$(stat -c %s "$path")" "$observed" "$(value "$path")" "$style"
done < <("$jq" -r '.ring[]|@tsv' "$scratch.stage1.json")
for rel in crypto/perlasm/x86_64-xlate.pl; do
  path="$original/$rel"
  [[ -f $path && ! -L $path ]] || fail "missing AWS-LC translator $path"
  printf 'TRANSLATOR AWS-LC %s bytes=%s blake3=%s pinned_git=verified-by-original-audit\n' "$rel" "$(stat -c %s "$path")" "$(value "$path")"
  path="$ring/$rel"
  [[ -f $path && ! -L $path ]] || fail "missing Ring translator $path"
  expected=$("$jq" -r --arg p "$rel" '.files[$p] // empty' "$ring/.cargo-checksum.json")
  observed=$(sha256sum "$path" | cut -d ' ' -f1)
  [[ $observed == "$expected" ]] || fail "Ring translator checksum mismatch"
  printf 'TRANSLATOR Ring %s bytes=%s sha256=%s blake3=%s\n' "$rel" "$(stat -c %s "$path")" "$observed" "$(value "$path")"
done
for name in rsaz-2k-avx512.pl rsaz-3k-avx512.pl rsaz-4k-avx512.pl; do
  path="$original/crypto/fipsmodule/bn/asm/$name"
  count=$(/nix/store/phnk1lwy8xs0yrbrcs6l2mb9yr9c2knp-perl-5.42.0/bin/perl -0777 -ne '$literal=q{`$ENV{CC} -Wa,-v -c -o /dev/null -x assembler /dev/null 2>&1`};$n=()=/\Q$literal\E/g;print "$n\n"' "$path")
  [[ $count -eq 1 ]] || fail "expected exact original assembler-probe literal once: $name count=$count"
  printf 'ASSEMBLER_PROBE original=%s exact_literal_count=%s bytes=%s blake3=%s\n' "$name" "$count" "$(stat -c %s "$path")" "$(value "$path")"
done
printf 'NON_CLAIM static source inspection only; no original Perl/Go source, helper, recipe, build, provider or product execution\n'
