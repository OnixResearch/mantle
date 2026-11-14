#!/usr/bin/env bash
# Read-only complete AWS-LC original script/utility input audit. Run:
# ssh leviathan.cymric-daggertooth.ts.net bash -s < local://source22-aws-original-input-audit-script
# Do not evaluate any AWS Perl or Go source; only inspect file contents.
set -euo pipefail
original=/home/brittonr/sbfp-20261113-berdKH/aws-original-source-02561621.TGISLN/original
cohort=/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
jq=/run/current-system/sw/bin/jq
fail(){ printf 'BLOCKER %s\n' "$*" >&2; exit 1; }
value(){ "$b3" --no-names "$original/$1"; }
stage1="$cohort/inputs/mantle-source/bootstrap/source-built/mantle-stage1.json"
stage2="$cohort/inputs/mantle-source/bootstrap/source-built/mantle-stage2.json"
filter='[.steps[]|select(.id|test("^unit-00037-script[.]perlasm-[0-9]+[.]generate$"))|[.id,.operation.arguments[2].parts[0].path,.operation.arguments[4].parts[0].text]]|sort_by(.[0]|capture("perlasm-(?<n>[0-9]+)").n|tonumber)|.[]|@tsv'
diff -q <("$jq" -r "$filter" "$stage1") <("$jq" -r "$filter" "$stage2") || fail 'stages differ in selected generators'
count=0; failed=0
while IFS=$'\t' read -r id relative output; do
  [[ "$relative" == crypto/*/*.pl || "$relative" == crypto/*/*/*.pl || "$relative" == crypto/*/*/*/*.pl ]] || fail "unexpected logical source $relative"
  [[ "$output" == *.S && "$output" != */* && -f "$original/$relative" ]] || fail "missing source or invalid output: $id $relative $output"
  count_pipe=$(perl -0777 -e '
    my $path=shift;open my $fh,"<",$path or die $!;my $text=<$fh>;my $count=0;
    for my $line (split /\n/,$text,-1) {
      my $normalized=$line;
      $normalized =~ s/^open OUT,\s*/open OUT,/;
      $normalized =~ s/\s+$//;
      $count++ if $normalized eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\"";};
    }
    print "$count\n";
  ' "$original/$relative")
  if [[ "$count_pipe" != 1 ]]; then failed=$((failed+1)); fi
  printf 'GENERATOR %s source=%s output=%s pipe=%s blake3=%s\n' "$id" "$relative" "$output" "$count_pipe" "$(value "$relative")"
  count=$((count+1))
done < <("$jq" -r "$filter" "$stage1")
[[ "$count" -eq 26 ]] || fail "AWS selections=$count, expected 26"
translator=crypto/perlasm/x86_64-xlate.pl
[[ -f "$original/$translator" ]] || fail 'missing x86_64 Perl translator'
printf 'TRANSLATOR %s blake3=%s\n' "$translator" "$(value "$translator")"
probe='`$ENV{CC} -Wa,-v -c -o /dev/null -x assembler /dev/null 2>&1`'
for name in rsaz-2k-avx512.pl rsaz-3k-avx512.pl rsaz-4k-avx512.pl; do
  relative="crypto/fipsmodule/bn/asm/$name"
  [[ -f "$original/$relative" ]] || fail "missing probe source $relative"
  num=$(perl -0777 -e 'my($path,$literal)=@ARGV;open my $fh,"<",$path or die $!;my $src=<$fh>;my $n=()=$src=~/\Q$literal\E/g;print "$n\n"' "$original/$relative" "$probe")
  [[ "$num" == 1 ]] || fail "changed rsaz assembler-probe cut: $relative count=$num"
  printf 'PROBE %s exact_count=%s blake3=%s\n' "$relative" "$num" "$(value "$relative")"
done
for relative in util/make_prefix_headers.go util/read_symbols.go crypto/err/err_data_generate.go; do
  [[ -f "$original/$relative" ]] || fail "missing original Go input $relative"
  printf 'UTILITY %s blake3=%s\n' "$relative" "$(value "$relative")"
done
perl -MFile::Find -e '
my ($original,$template)=@ARGV;
sub raw_strings {
  my ($src,$function)=@_;
  my $start=index($src,"func $function("); die "missing Go function $function\n" if $start<0;
  $src=substr($src,$start); my $end=index($src,"\nfunc "); $src=substr($src,0,$end) if $end>=0;
  my @chunks=($src =~ /`([^`]*)`/gs);
  return scalar @chunks;
}
sub load {my($path)=@_;open my $fh,"<",$path or die "$path: $!"; local $/;return <$fh>}
my $headers=load("$original/util/make_prefix_headers.go");
for my $check ([writeCHeader,3],[writeASMHeader,3],[writeNASMHeader,4]) {
 my ($name,$wanted)=@$check; my $n=raw_strings($headers,$name);die "changed original header template $name: $n\n" unless $n==$wanted;
 print "HEADER_TEMPLATE $name raw_strings=$n\n";
}
my $symbols=load("$original/util/read_symbols.go");
$symbols =~ /var skipSymbols = \[\]string\{(.*?)\}/s or die "missing original symbol exclusions\n";
my @skips=($1 =~ /^\s*"([^"]+)"/mg);
die "empty original symbol exclusions\n" unless @skips;
printf "SYMBOL_EXCLUSIONS original_entries=%d\n",scalar @skips;
my $err=load("$original/crypto/err/err_data_generate.go");
$err =~ /var libraryNames = \[\]string\{(.*?)\}/s or die "missing original library names\n";
my @libraries=($1 =~ /^\s*"([^"]+)"/mg);die "invalid library enum size\n" if !@libraries || @libraries>=64;
my %library=map { $libraries[$_] => $_+1 } 0..$#libraries;
my $chunks=raw_strings($err,"main");die "changed original error output template: $chunks\n" unless $chunks==1;
my @error_data=sort glob("$original/crypto/err/*.errordata");die "missing errordata\n" unless @error_data;
my (%interned,%keys);my $length=0;my $rows=0;
for my $file (@error_data) {
 for my $line (split /\n/,load($file)) {
  next if !length($line);
  my @fields=split /,/,$line,-1;
  die "bad error row in $file\n" unless @fields==3 && exists $library{$fields[0]} && $fields[1]=~/^[0-9]+$/ && $fields[1]<2048;
  my $key=$library{$fields[0]}<<26 | $fields[1]<<15;
  die "duplicate error key: $file\n" if $keys{$key}++;
  if (!exists $interned{$fields[2]}) {
    die "error strings overflow\n" if $length>0x7fff;
    $interned{$fields[2]}=$length;$length+=length($fields[2])+1;
  }
  $rows++;
 }
}
printf "ERROR_DATA libraries=%d files=%d rows=%d unique_keys=%d string_bytes=%d main_raw_strings=%d\n",scalar @libraries,scalar @error_data,$rows,scalar(keys %keys),$length,$chunks;
# source-data.prepare() copies the source filenames in vendor aws-lc package template,
# except generated-src and boringssl_prefix_symbols*; all corresponding original files must exist.
my $count=0;
find({no_chdir=>1,wanted=>sub {
  my $path=$File::Find::name;
  my $rel=substr($path,length($template)+1);
  return unless length($rel);
  if ($rel =~ m{(?:^|/)generated-src(?:/|$)} || $rel =~ m{(?:^|/)boringssl_prefix_symbols[^/]*$}) {
    $File::Find::prune=1 if -d $path;
    return;
  }
  die "unexpected template node $rel\n" unless -d $path || -f $path;
  if (-f $path) {
    die "missing original-native copy input $rel\n" unless -f "$original/$rel" && !-l "$original/$rel";
    $count++;
  }
}},$template);
print "ORIGINAL_NATIVE_TEMPLATE regular_source_inputs=$count missing=0\n";
' "$original" "$cohort/inputs/vendor-deps/aws-lc-sys/aws-lc"
printf 'SUMMARY selected=%s pipe_mismatches=%s probes=3\n' "$count" "$failed"
if (( failed > 0 )); then printf 'BLOCKER frozen PerlAsm PIPE_SOURCE rejects %s selected original AWS scripts\n' "$failed" >&2; exit 3; fi
