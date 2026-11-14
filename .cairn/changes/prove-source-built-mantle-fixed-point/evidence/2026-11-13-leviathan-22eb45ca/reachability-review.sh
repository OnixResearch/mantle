#!/usr/bin/env bash
# Read-only exact reachability walk: inputs plus ObservePhase edges, matching frozen recipe validator.
# ssh leviathan.cymric-daggertooth.ts.net bash -s < local://source22-reachability-review-script
set -euo pipefail
base=/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed
for stage in 1 2; do
 file="$base/inputs/mantle-source/bootstrap/source-built/mantle-stage${stage}.json"
 /run/current-system/sw/bin/jq -c '.stage_id as $stage|{stage_id:$stage,steps:[.steps[]|{id,prerequisites:([.inputs[]|select(.kind=="produced")|.action_id]+(if .operation.kind=="observe-phase" then [$stage+"/"+.operation.phase] else [] end))}],roots:[.roots[]|select(.kind=="produced")|.action_id]}' "$file" | /run/current-system/sw/bin/perl -MJSON::PP -e '
   use strict; use warnings;
   local $/;
   my $r = decode_json(<STDIN>);
   my $stage = $r->{stage_id};
   my %by_id = map { ("$stage/$_->{id}" => $_) } @{$r->{steps}};
   my @todo = @{$r->{roots}};
   my %reached;
   while (my $id = pop @todo) {
     next unless index($id,"$stage/")==0;
     next if $reached{$id}++;
     my $step = $by_id{$id} or die "unknown reachable local step $id\n";
     push @todo, @{$step->{prerequisites}};
   }
   my @missing;
   for my $n (0..11) {
     my $p="unit-00031-script.sources.perl-$n";
     for my $suffix (qw(authenticate pipe-source generate translate result)) {
       push @missing, "$p.$suffix" unless $reached{"$stage/$p.$suffix"};
     }
     push @missing,"unit-00031-script.sources.reply-$n.reply" unless $reached{"$stage/unit-00031-script.sources.reply-$n.reply"};
   }
   for my $n (0..25) {
     my $p="unit-00037-script.perlasm-$n";
     for my $suffix (qw(pipe-source generate translate)) {
       push @missing, "$p.$suffix" unless $reached{"$stage/$p.$suffix"};
     }
   }
   for my $id (qw(aws-original-source unit-00031-script.generated-package unit-00031-script.sources.complete unit-00037-script.perlasm-source unit-00037-script.assembler-version unit-00037-script.generated-package unit-00037-script.source-data unit-00037-script.unprefixed.complete)) {
     push @missing, $id unless $reached{"$stage/$id"};
   }
   printf "REACHABLE %s local_total=%d/%d ring_auth=%d/12 aws_generate=%d/26 missing=%s\n",
     $stage, scalar(keys %reached), scalar(keys %by_id),
     scalar(grep { $reached{"$stage/unit-00031-script.sources.perl-$_.authenticate"} } (0..11)),
     scalar(grep { $reached{"$stage/unit-00037-script.perlasm-$_.generate"} } (0..25)),
     @missing ? join(",",@missing) : "none";
   exit(@missing ? 3 : 0);
 '
done
