#!/usr/bin/env bash
# Read-only stage1/stage2 typed graph and actual original PerlAsm script boundary audit.
set -euo pipefail
r=/home/brittonr/sbfp-20261113-berdKH
c="$r/prepared-cohort-8fa84260-aws"
f="$r/runner-source-8fa84260-aws"
original="$c/inputs/mantle-source/bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9"
ring="$c/inputs/vendor-deps/ring"
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
jq=/run/current-system/sw/bin/jq
fail(){ printf 'BLOCKER %s\n' "$*" >&2; exit 3; }
value(){ "$b3" --no-names "$1"; }
[[ -f "$original/crypto/perlasm/x86_64-xlate.pl" && -f "$ring/crypto/perlasm/x86_64-xlate.pl" ]] || fail 'original translator absent'
filter=/dev/shm/source8-identity-recipes-8fa84260.jq
[[ -f $filter ]] || fail "missing inspection filter $filter"
for n in 1 2; do
  file="$c/inputs/mantle-source/bootstrap/source-built/mantle-stage$n.json"
  "$jq" -c -f "$filter" "$file"
  printf 'RECIPE_RAW stage=%s blake3=%s bytes=%s\n' "$n" "$(value "$file")" "$(stat -c %s "$file")"
  actual=$("$jq" -rj '.steps[]|select(.id=="unit-00037-script.perlasm-0.pipe-source")|.operation.text' "$file" | "$b3" --no-names)
  frozen=$(perl -0777 -ne 'm{const PIPE_SOURCE: &str = r###"(.*?)"###;}s or die "frozen PIPE_SOURCE not found\n";print $1;' "$f/src/source_built_mantle_programs_native.rs" | "$b3" --no-names)
  [[ $actual == "$frozen" ]] || fail "stage$n source pipe wrapper differs from FROZEN: $actual/$frozen"
  distinct=$("$jq" -r '[.steps[]|select(.id|test("^unit-00031-script[.]sources[.]perl-[0-9]+[.]pipe-source$|^unit-00037-script[.]perlasm-[0-9]+[.]pipe-source$"))|.operation.text]|unique|length' "$file")
  [[ $distinct -eq 1 ]] || fail "stage$n selected pipe wrappers not all identical"
  printf 'PIPE_WRAPPER stage=%s frozen_b3=%s distinct=%s\n' "$n" "$frozen" "$distinct"
  "$jq" -c '.stage_id as $stage|{stage_id:$stage,steps:[.steps[]|{id,prerequisites:([.inputs[]|select(.kind=="produced")|.action_id]+(if .operation.kind=="observe-phase" then [$stage+"/"+.operation.phase] else [] end))}],roots:[.roots[]|select(.kind=="produced")|.action_id]}' "$file" | perl -MJSON::PP -e '
    use strict;use warnings;local $/;my $r=decode_json(<STDIN>);my $stage=$r->{stage_id};my %by=map {("$stage/$_->{id}"=>$_)} @{$r->{steps}};
    my @todo=@{$r->{roots}};my %reached;
    while(my $id=pop @todo){next unless index($id,"$stage/")==0;next if $reached{$id}++;my $s=$by{$id} or die "missing reachable step $id\n";push @todo,@{$s->{prerequisites}};}
    my @missing=grep {!$reached{"$stage/$_"}} map {$_->{id}} grep {$_->{id}=~/^(?:unit-00031-script\.sources\.perl-[0-9]+\.|unit-00031-script\.sources\.reply-[0-9]+\.reply$|unit-00037-script\.perlasm-[0-9]+\.|aws-original-source$|unit-00031-script\.generated-package$|unit-00031-script\.complete$|unit-00037-script\.(?:perlasm-source|source-data|generated-package|unprefixed\.complete)$)/} @{$r->{steps}};
    printf "REACHABILITY stage=%s reached=%d total=%d selected_missing=%d\n",$stage,scalar(keys %reached),scalar(keys %by),scalar(@missing);
    die "unreachable selected steps: @missing\n" if @missing;
  '
done
selector='[.steps[]|select(.id|test("^unit-00037-script[.]perlasm-[0-9]+[.]generate$"))|[.id,.operation.arguments[2].parts[0].path,.operation.arguments[4].parts[0].text]]|sort_by(.[0]|capture("perlasm-(?<n>[0-9]+)").n|tonumber)|.[]|@tsv'
aws1="$c/inputs/mantle-source/bootstrap/source-built/mantle-stage1.json"
aws2="$c/inputs/mantle-source/bootstrap/source-built/mantle-stage2.json"
diff -q <("$jq" -r "$selector" "$aws1") <("$jq" -r "$selector" "$aws2") || fail 'AWS original script selection differs across stages'
ring_selector='[.steps[]|select(.id|test("^unit-00031-script[.]sources[.]perl-[0-9]+[.]authenticate$"))|[.id,.operation.arguments[3].parts[0].path,.operation.arguments[4].parts[0].text]]|sort_by(.[0]|capture("perl-(?<n>[0-9]+)").n|tonumber)|.[]|@tsv'
diff -q <("$jq" -r "$ring_selector" "$aws1") <("$jq" -r "$ring_selector" "$aws2") || fail 'Ring original script selection differs across stages'
checked=0;standalone=0;guarded=0
while IFS=$'\t' read -r id relative output; do
  path="$original/$relative"
  [[ $relative == crypto/*.pl || $relative == crypto/*/*.pl || $relative == crypto/*/*/*.pl ]] || fail "invalid AWS script member $relative"
  [[ $output == *.S && $output != */* && -f $path && ! -L $path ]] || fail "missing AWS script $relative"
  style=$(perl -ne '
    BEGIN{$standalone=0;$guarded=0;$bad=0;}
    chomp;my $norm=$_;$norm=~s/^open OUT,\s*/open OUT,/;$norm=~s/\s+$//;
    if($norm eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\";}){$standalone++;}
    if($norm eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\"}){my $continuation=<>//"";$continuation=~s/^\s+|\s+$//g;$guarded++ if $continuation eq (q{or die "can}.chr(39).q{t call $xlate: $!";});$bad++ if $continuation ne (q{or die "can}.chr(39).q{t call $xlate: $!";});}
    END{die "unexpected original pipe cut\n" unless $standalone+$guarded==1 && !$bad;print $guarded ? "guarded" : "standalone";}
  ' "$path") || fail "AWS source boundary rejected: $relative"
  [[ $style == guarded || $style == standalone ]] || fail "AWS source boundary style invalid $relative"
  if [[ $style == guarded ]]; then guarded=$((guarded+1)); else standalone=$((standalone+1)); fi
  printf 'AWS_ORIGINAL %s %s -> %s bytes=%s b3=%s cut=%s\n' "$id" "$relative" "$output" "$(stat -c %s "$path")" "$(value "$path")" "$style"
  checked=$((checked+1))
done < <("$jq" -r "$selector" "$aws1")
printf 'AWS_SELECTION scripts=%s standalone=%s guarded=%s translator_b3=%s\n' "$checked" "$standalone" "$guarded" "$(value "$original/crypto/perlasm/x86_64-xlate.pl")"
checked=0
while IFS=$'\t' read -r id relative output; do
  path="$ring/$relative"
  [[ $relative == crypto/*.pl || $relative == crypto/*/*.pl || $relative == crypto/*/*/*.pl ]] || fail "invalid Ring script member $relative"
  [[ $output == *.S && $output != */* && -f $path && ! -L $path ]] || fail "missing Ring script $relative"
  count=$(perl -ne 'BEGIN{$n=0}chomp;my $norm=$_;$norm=~s/^open OUT,\s*/open OUT,/;$norm=~s/\s+$//;$n++ if $norm eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\""} || $norm eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\"";};END{print $n}' "$path")
  [[ $count -eq 1 ]] || fail "Ring source boundary count=$count: $relative"
  printf 'RING_ORIGINAL %s %s -> %s bytes=%s b3=%s cut=%s\n' "$id" "$relative" "$output" "$(stat -c %s "$path")" "$(value "$path")" "$count"
  checked=$((checked+1))
done < <("$jq" -r "$ring_selector" "$aws1")
printf 'RING_SELECTION scripts=%s translator_b3=%s\n' "$checked" "$(value "$ring/crypto/perlasm/x86_64-xlate.pl")"
for name in rsaz-2k-avx512.pl rsaz-3k-avx512.pl rsaz-4k-avx512.pl; do
 path="$original/crypto/fipsmodule/bn/asm/$name"
 count=$(perl -0777 -ne '$literal=q{`$ENV{CC} -Wa,-v -c -o /dev/null -x assembler /dev/null 2>&1`};$n=()=/\Q$literal\E/g;print "$n\n"' "$path")
 [[ $count -eq 1 ]] || fail "rsaz probe source changed: $name count=$count"
 printf 'ASSEMBLER_PROBE %s literal_count=%s\n' "$name" "$count"
done
