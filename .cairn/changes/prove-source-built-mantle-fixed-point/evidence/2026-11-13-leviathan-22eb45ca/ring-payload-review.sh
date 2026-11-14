#!/usr/bin/env bash
# Read-only vendor Perl source/pipeline boundary check; never executes generator or assembler.
# ssh leviathan.cymric-daggertooth.ts.net bash -s < local://source22-ring-payload-review-script
set -euo pipefail
base=/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed
jq=/run/current-system/sw/bin/jq
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
vendor="$base/inputs/vendor-deps/ring"
recipe="$base/inputs/mantle-source/bootstrap/source-built/mantle-stage1.json"
rows=0
while IFS=$'\t' read -r id src out; do
 f="$vendor/$src"
 [[ -f "$f" && ! -L "$f" ]] || { printf 'BLOCKER absent Ring original %s\n' "$f"; exit 3; }
 count=$(/run/current-system/sw/bin/perl -e '
  use strict; use warnings;
  my $wanted=q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\"";};
  open my $fh,"<",$ARGV[0] or die $!; my $count=0;
  while (my $line=<$fh>) { chomp $line; $line =~ s/^open OUT,\s*/open OUT,/; $line =~ s/\s+$//; ++$count if $line eq $wanted; }
  print $count;
 ' "$f")
 [[ "$count" -eq 1 ]] || { printf 'BLOCKER Ring pipe boundary count=%s %s\n' "$count" "$src"; exit 3; }
 printf 'RING_SCRIPT %s %s -> %s b3=%s bytes=%s pipe_count=%s\n' "$id" "$src" "$out" "$("$b3" "$f" | { read -r h ignored; printf '%s' "$h"; })" "$(stat -c %s "$f")" "$count"
 rows=$((rows+1))
done < <("$jq" -r '.steps[]|select(.id|test("^unit-00031-script\\.sources\\.perl-[0-9]+\\.authenticate$"))|[.id,.operation.arguments[3].parts[0].path,.operation.arguments[4].parts[0].text]|@tsv' "$recipe")
[[ $rows -eq 12 ]] || { printf 'BLOCKER expected 12 actual Ring selections; got %s\n' "$rows"; exit 3; }
f="$vendor/crypto/perlasm/x86_64-xlate.pl"
[[ -f "$f" && ! -L "$f" ]] || { printf 'BLOCKER absent Ring original translator %s\n' "$f"; exit 3; }
printf 'RING_TRANSLATOR crypto/perlasm/x86_64-xlate.pl b3=%s bytes=%s\n' "$("$b3" "$f" | { read -r h ignored; printf '%s' "$h"; })" "$(stat -c %s "$f")"
