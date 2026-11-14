#!/usr/bin/env bash
# Read-only inspection of independently captured pinned AWS-LC Git source.
# Run: ssh leviathan.cymric-daggertooth.ts.net bash -s < local://source22-aws-original-capture-review-script
# Git plumbing verifies source bytes/modes; no PerlAsm generator, compiler, provider, build, test or source-profile parser is run.
set -euo pipefail
capture=/home/brittonr/sbfp-20261113-berdKH/aws-original-source-02561621.TGISLN
original="$capture/original"
db="$capture/aws-lc.git"
cohort=/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed
report="$capture/original-source-capture.json"
rev=02561621ffa4cf17c0c4f70bc11a82df36b42ae9
tree=969d75db5c0494de4a26bbb2dcbf6837058054c1
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
git=/run/current-system/sw/bin/git
jq=/run/current-system/sw/bin/jq
tar=/run/current-system/sw/bin/tar
fail() { printf 'BLOCKER %s\n' "$*" >&2; exit 1; }
value() { "$b3" --no-names "$1"; }
reported() { "$jq" -r --arg field "$1" '.[$field]' "$report"; }
[[ "$(reported revision)" == "$rev" && "$(reported git_tree_sha1)" == "$tree" && "$(reported tracked_file_count)" == 10427 && "$(reported source_root)" == "$original" ]] || fail 'capture report does not bind pinned source identity'
[[ "$("$git" --git-dir="$db" cat-file -t "$rev")" == commit && "$("$git" --git-dir="$db" rev-parse "$rev^{tree}")" == "$tree" ]] || fail 'captured Git object or tree differs from required revision'
printf 'PIN revision=%s tree=%s\n' "$rev" "$tree"
# lstat every tracked file, preserving Git executable modes; reject links and extra files.
"$git" --git-dir="$db" ls-tree -r -z "$rev" | perl -MFile::Find -0ne '
BEGIN { $root=$ARGV[0]; shift @ARGV; $expected_count=0; }
chomp;
my ($mode,$type,$oid,$rel)=/^([0-7]{6}) (\S+) ([a-f0-9]{40})\t(.*)$/s or die "malformed Git tree entry\n";
die "unexpected object type or mode: $rel: $type $mode\n" unless $type eq "blob" && ($mode eq "100644" || $mode eq "100755");
die "newline in Git path: $rel\n" if $rel =~ /[\r\n]/;
die "duplicate Git path: $rel\n" if exists $paths{$rel};
$paths{$rel}=1; $expected_count++;
my $path="$root/$rel";
my @s=lstat($path); die "missing source: $rel\n" unless @s;
die "not regular source: $rel\n" unless -f _ && !-l _;
my $permissions=$s[2]&07777;
my $required=$mode eq "100755" ? 0755 : 0644;
die sprintf("wrong mode: %s actual=%04o expected=%04o\n",$rel,$permissions,$required) unless $permissions==$required;
END {
  die "wrong tracked count: $expected_count\n" unless $expected_count==10427;
  my $seen=0;
  find({no_chdir=>1,wanted=>sub {
      my $path=$File::Find::name;
      my @s=lstat($path); die "cannot stat $path: $!\n" unless @s;
      if (-d _) { return; }
      my $rel=substr($path,length($root)+1);
      die "unexpected or nonregular source: $rel\n" unless -f _ && !-l _ && exists $paths{$rel};
      $seen++;
  }},$root);
  die "source count $seen != tracked $expected_count\n" unless $seen==$expected_count;
  printf "TREE_FILES checked=%d modes=100644,100755 extra=0 symlinks=0\n",$seen;
}
' "$original"
# One Git process hashes every captured file as a blob; preserve ls-tree order.
diff -q \
  <("$git" --git-dir="$db" ls-tree -r --format='%(objectname)' "$rev") \
  <("$git" --git-dir="$db" ls-tree -r --name-only -z "$rev" | perl -0ne 'BEGIN { $root=shift @ARGV; } chomp; die "newline path" if /[\r\n]/; print "$root/$_\n"' "$original" | "$git" hash-object --stdin-paths) \
  || fail 'captured source bytes do not match pinned Git blob object IDs'
printf 'BLOB_CONTENT matched=10427 pinned_tree=%s\n' "$tree"
actual_archive=$(value "$capture/aws-lc-original.tar")
[[ "$actual_archive" == "$(reported archive_blake3)" ]] || fail 'physical Git archive hash differs from capture report'
printf 'GIT_ARCHIVE blake3=%s\n' "$actual_archive"
actual_tar=$("$tar" --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$original" . | "$b3" --no-names)
[[ "$actual_tar" == "$(reported normalized_tar_blake3)" ]] || fail 'normalized physical source tree tar hash differs from capture report'
printf 'NORMALIZED_TAR blake3=%s\n' "$actual_tar"
stage1="$cohort/inputs/mantle-source/bootstrap/source-built/mantle-stage1.json"
stage2="$cohort/inputs/mantle-source/bootstrap/source-built/mantle-stage2.json"
filter='[.steps[]|select(.id|test("^unit-00037-script[.]perlasm-[0-9]+[.]generate$"))|[.id,.operation.arguments[2].parts[0].path,.operation.arguments[4].parts[0].text]]|sort_by(.[0]|capture("perlasm-(?<n>[0-9]+)").n|tonumber)|.[]|@tsv'
diff -q <("$jq" -r "$filter" "$stage1") <("$jq" -r "$filter" "$stage2") || fail 'stage1/stage2 AWS generator source selections differ'
count=0
while IFS=$'\t' read -r id relative output; do
  [[ "$relative" == crypto/*/*.pl || "$relative" == crypto/*/*/*.pl || "$relative" == crypto/*/*/*/*.pl ]] || fail "unexpected AWS logical .pl path: $relative"
  [[ "$output" == *.S && "$output" != */* && -f "$original/$relative" ]] || fail "missing AWS generator or invalid basename: $id $relative $output"
  # Exact line-normalizing comparison performed by frozen PIPE_SOURCE wrapper; do not evaluate Perl source.
  count_pipe=$(perl -0777 -e '
    my $path=shift; open my $fh,"<",$path or die "$path: $!";
    my $text=<$fh>; my $count=0;
    for my $line (split /\n/,$text,-1) {
      my $normalized=$line;
      $normalized =~ s/^open OUT,\s*/open OUT,/;
      $normalized =~ s/\s+$//;
      $count++ if $normalized eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\"";};
    }
    print "$count\n";
  ' "$original/$relative")
  [[ "$count_pipe" == 1 ]] || fail "PIPE_SOURCE will reject $id $relative count=$count_pipe"
  printf 'GENERATOR %s source=%s output=%s pipe=%s blake3=%s\n' "$id" "$relative" "$output" "$count_pipe" "$(value "$original/$relative")"
  count=$((count+1))
done < <("$jq" -r "$filter" "$stage1")
[[ "$count" -eq 26 ]] || fail "expected 26 AWS selections, got $count"
translator=crypto/perlasm/x86_64-xlate.pl
[[ -f "$original/$translator" ]] || fail 'missing PerlAsm translator'
printf 'TRANSLATOR %s blake3=%s\n' "$translator" "$(value "$original/$translator")"
probe='`$ENV{CC} -Wa,-v -c -o /dev/null -x assembler /dev/null 2>&1`'
for name in rsaz-2k-avx512.pl rsaz-3k-avx512.pl rsaz-4k-avx512.pl; do
  relative="crypto/fipsmodule/bn/asm/$name"
  [[ -f "$original/$relative" ]] || fail "missing probe script $relative"
  num=$(perl -0777 -e 'my($path,$literal)=@ARGV;open my $fh,"<",$path or die $!;my $src=<$fh>;my $n=()=$src=~/\Q$literal\E/g;print "$n\n"' "$original/$relative" "$probe")
  [[ "$num" == 1 ]] || fail "changed original assembler-probe cut: $relative count=$num"
  printf 'PROBE %s exact_count=%s blake3=%s\n' "$relative" "$num" "$(value "$original/$relative")"
done
# Inspect original Go utility inputs and native copy paths, not the Go generators themselves.
for relative in util/make_prefix_headers.go util/read_symbols.go crypto/err/err_data_generate.go; do
  [[ -f "$original/$relative" ]] || fail "missing utility generator input: $relative"
  printf 'UTILITY %s blake3=%s\n' "$relative" "$(value "$original/$relative")"
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
# source-data.prepare() recursively takes original filenames from vendored package template,
# except original generated-src and boringssl_prefix_symbols*; check every required original source exists.
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
printf 'SUITABILITY source_acquisition_only=1 cold_proof=0 old_cohort_still_rejected=1\n'
