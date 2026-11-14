#!/usr/bin/env bash
# Frozen-source, read-only stage audit; does not execute Perl, a builder or source-profile verification.
# ssh leviathan.cymric-daggertooth.ts.net bash -s < local://source22-perlasm-review-script
set -euo pipefail
base=/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed
jq=/run/current-system/sw/bin/jq
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
aws_original="$base/inputs/mantle-source/bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9"
for ordinal in 1 2; do
  file="$base/inputs/mantle-source/bootstrap/source-built/mantle-stage${ordinal}.json"
  "$jq" -c --arg ordinal "$ordinal" '
    def ref($stage;$step;$out): {kind:"produced",action_id:($stage+"/"+$step),output_id:$out};
    def iw($word;$name;$path): $word.parts == [{kind:"input",name:$name,path:$path}];
    def lw($word;$text): $word.parts == [{kind:"literal",text:$text}];
    def used:
      [.operation.executable_input,.operation.cwd_input,.operation.stdin_input,.operation.invocation_input,
       (.operation.runtime_mounts//{}|.[]),(.operation.output_seeds//{}|.[]),
       ((.operation.arguments//[])[]?.parts[]?|select(.kind=="input" or .kind=="text_input" or .kind=="text_slice")|.name),
       (.operation.environment//{}|.[]|.parts[]?|select(.kind=="input" or .kind=="text_input" or .kind=="text_slice")|.name)]|map(select(.!=null))|unique;
    . as $doc | .stage_id as $stage |
    ($doc.steps|INDEX(.id)) as $m |
    [$doc.steps[]|select(.id|test("^unit-00031-script\\.sources\\.perl-[0-9]+\\.generate$"))] as $ring |
    [$doc.steps[]|select(.id|test("^unit-00037-script\\.perlasm-[0-9]+\\.generate$"))] as $aws |
    [$ring[] | . as $g | (.id|sub("\\.generate$";"")) as $prefix |
      ($g.id|capture("perl-(?<n>[0-9]+)\\.generate$").n) as $n |
      $m[$prefix+".authenticate"] as $a | $m[$prefix+".translate"] as $t |
      $m[$prefix+".result"] as $r | $m["unit-00031-script.sources.reply-"+$n+".reply"] as $reply |
      select((
        $a.inputs.package=={kind:"source",role:"vendor-inputs",path:"ring"} and
        ($a.inputs["native-perl"].action_id|startswith("full-source-native-provider/")) and
        ($a.inputs.request.kind=="produced") and lw($a.operation.arguments[0];"--ring-request") and
        iw($a.operation.arguments[1];"request";"") and iw($a.operation.arguments[2];"native-perl";"") and
        ($a.operation.arguments[3].parts[0].name=="package") and
        $a.operation.arguments[5].parts==[{kind:"output",name:"authenticated",path:""}] and
        $g.inputs["original-script"]==ref($stage;$prefix+".authenticate";"authenticated") and
        $g.inputs.source==$a.inputs.package and $g.inputs.perl==$a.inputs["native-perl"] and
        $g.inputs["pipe-source"]==ref($stage;$prefix+".pipe-source";"source") and
        iw($g.operation.arguments[0];"pipe-source";"") and iw($g.operation.arguments[1];"original-script";"") and
        iw($g.operation.arguments[2];"source";$a.operation.arguments[3].parts[0].path) and
        lw($g.operation.arguments[4];$a.operation.arguments[4].parts[0].text) and $g.operation.cwd_input=="source" and
        $t.inputs.raw==ref($stage;$prefix+".generate";"raw") and $t.inputs.source==$g.inputs.source and
        $t.inputs.perl==$g.inputs.perl and $t.operation.stdin_input=="raw" and
        iw($t.operation.arguments[0];"source";"crypto/perlasm/x86_64-xlate.pl") and
        lw($t.operation.arguments[1];"elf") and
        $r.inputs.assembly==ref($stage;$prefix+".translate";"assembly") and
        lw($r.operation.arguments[0];"--perlasm-result") and
        $reply.inputs.files==ref($stage;$prefix+".result";"files") and
        $m["unit-00031-script.sources.complete.replies"].inputs["reply-"+$n]==ref($stage;"unit-00031-script.sources.reply-"+$n+".reply";"reply")
      )|not) | $prefix
    ] as $ring_fail |
    [$aws[] | . as $g | (.id|sub("\\.generate$";"")) as $prefix |
      ($g.id|capture("perlasm-(?<n>[0-9]+)\\.generate$").n) as $n |
      $m[$prefix+".translate"] as $t | $m["unit-00037-script.generated-package"] as $pack |
      select((
        $g.inputs["original-script"]==ref($stage;"unit-00037-script.perlasm-source";"source") and
        $g.inputs.source==$g.inputs["original-script"] and
        ($g.inputs.perl.action_id|startswith("full-source-native-provider/")) and
        $g.inputs["pipe-source"]==ref($stage;$prefix+".pipe-source";"source") and
        $g.inputs["aws-as-probe"]==ref($stage;"unit-00037-script.assembler-version";"stderr") and
        iw($g.operation.environment.MANTLE_AWS_AS_PROBE;"aws-as-probe";"") and
        iw($g.operation.arguments[0];"pipe-source";"") and
        ($g.operation.arguments[1].parts[0].name=="original-script") and
        ($g.operation.arguments[1].parts[0].path==$g.operation.arguments[2].parts[0].path) and
        ($g.operation.arguments[2].parts[0].name=="source") and
        ($g.operation.arguments[2].parts[0].path|endswith(".pl")) and
        ($g.operation.arguments[4].parts[0].text|endswith(".S")) and $g.operation.cwd_input=="source" and
        $t.inputs.raw==ref($stage;$prefix+".generate";"raw") and $t.inputs.source==$g.inputs.source and
        $t.inputs.perl==$g.inputs.perl and $t.operation.stdin_input=="raw" and
        iw($t.operation.arguments[0];"source";"crypto/perlasm/x86_64-xlate.pl") and
        $t.inputs["aws-as-probe"]==$g.inputs["aws-as-probe"] and
        iw($t.operation.environment.MANTLE_AWS_AS_PROBE;"aws-as-probe";"") and
        $pack.inputs["assembly-"+$n]==ref($stage;$prefix+".translate";"assembly") and
        ($pack.operation.arguments|map(.parts[0])|any(.kind=="output" and .name=="package" and (.path|endswith("/"+$g.operation.arguments[4].parts[0].text))))
      )|not) | $prefix
    ] as $aws_fail |
    [$doc.steps[]|select(.id|test("^unit-00031-script\\.sources\\.perl-[0-9]+\\.|^unit-00037-script\\.perlasm-[0-9]+\\."))|. as $s|($s.inputs|keys - ($s|used))|select(length>0)|{id:$s.id,unused:.}] as $unused |
    [$doc.steps[]|select(.id|test("^unit-00031-script\\.sources\\.perl-[0-9]+\\.pipe-source$|^unit-00037-script\\.perlasm-[0-9]+\\.pipe-source$"))|select((.operation.text | (contains("authenticated PerlAsm source") and contains("$0 = $source;") and contains("changed original PerlAsm pipe boundary")))|not)|.id] as $pipe_fail |
    {ordinal:$ordinal,stage_id:$stage,recipe_steps:($doc.steps|length),semantic_roots:($doc.semantic_roots|length),
      ring_generators:($ring|length),ring_authentications:([$doc.steps[]|select(.id|test("^unit-00031-script\\.sources\\.perl-[0-9]+\\.authenticate$"))]|length),
      aws_generators:($aws|length),aws_original:$m["aws-original-source"].inputs.original,
      aws_source_tree:$m["unit-00037-script.perlasm-source"].inputs["aws-original"],
      aws_source_data_original:$m["unit-00037-script.source-data"].inputs["aws-original"],
      aws_probe_executable:$m["unit-00037-script.assembler-version"].inputs.as,
      ring_downstream_package:$m["unit-00031-script.complete"].inputs.package,
      ring_generated_package:$m["unit-00031-script.generated-package"].inputs.generated,
      aws_downstream_package:$m["unit-00037-script.unprefixed.complete"].inputs.package,
      ring_semantic:([$doc.semantic_roots|to_entries[]|select(.key|contains("#ring@"))|.value]|length),
      aws_semantic:([$doc.semantic_roots|to_entries[]|select(.key|contains("#aws-lc-sys@"))|.value]|length),
      ring_fail:$ring_fail,aws_fail:$aws_fail,unused_declared_inputs:$unused,pipe_fail:$pipe_fail}
  ' "$file"
  printf 'RECIPE_RAW_BLAKE3 mantle-stage%s %s\n' "$ordinal" "$("$b3" "$file" | { read -r val ignored; printf '%s' "$val"; })"
done
if [[ ! -d "$aws_original" ]]; then
  printf 'BLOCKER AWS original Mantle source directory absent: %s\n' "$aws_original"
  exit 3
fi
printf 'AWS_SOURCE present directory: %s\n' "$aws_original"
