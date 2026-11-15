def ref($stage;$step;$out): {kind:"produced", action_id:($stage+"/"+$step), output_id:$out};
def iw($word;$name;$path): $word.parts == [{kind:"input",name:$name,path:$path}];
def lw($word;$text): $word.parts == [{kind:"literal",text:$text}];
def used: [.operation.executable_input,.operation.cwd_input,.operation.stdin_input,.operation.invocation_input,
  (.operation.runtime_mounts//{}|.[]),(.operation.output_seeds//{}|.[]),
  ((.operation.arguments//[])[]?.parts[]?|select(.kind=="input" or .kind=="text_input" or .kind=="text_slice")|.name),
  (.operation.environment//{}|.[]|.parts[]?|select(.kind=="input" or .kind=="text_input" or .kind=="text_slice")|.name)]|map(select(.!=null))|unique;
. as $doc | .stage_id as $stage | (.steps|INDEX(.id)) as $m |
[$doc.steps[]|select(.id|test("^unit-00031-script[.]sources[.]perl-[0-9]+[.]generate$"))] as $ring |
[$doc.steps[]|select(.id|test("^unit-00037-script[.]perlasm-[0-9]+[.]generate$"))] as $aws |
[$ring[] | . as $g | (.id|sub("[.]generate$";"")) as $p |
  ($g.id|capture("perl-(?<n>[0-9]+)[.]generate$").n) as $n |
  $m[$p+".authenticate"] as $a | $m[$p+".translate"] as $t | $m[$p+".result"] as $result |
  $m["unit-00031-script.sources.reply-"+$n+".reply"] as $reply |
  select((
    $a.inputs.package == {kind:"source",role:"vendor-inputs",path:"ring"} and
    ($a.inputs["native-perl"].action_id|startswith("full-source-native-provider/")) and
    $a.inputs.request.kind=="produced" and
    lw($a.operation.arguments[0];"--ring-request") and
    iw($a.operation.arguments[1];"request";"") and
    iw($a.operation.arguments[2];"native-perl";"") and
    $a.operation.arguments[3].parts == [{kind:"input",name:"package",path:$a.operation.arguments[3].parts[0].path}] and
    ($a.operation.arguments[3].parts[0].path|endswith(".pl")) and
    ($a.operation.arguments[4].parts[0].text|endswith(".S")) and
    $g.inputs["original-script"]==ref($stage;$p+".authenticate";"authenticated") and
    $g.inputs.source==$a.inputs.package and $g.inputs.perl==$a.inputs["native-perl"] and
    $g.inputs["pipe-source"]==ref($stage;$p+".pipe-source";"source") and
    iw($g.operation.arguments[0];"pipe-source";"") and iw($g.operation.arguments[1];"original-script";"") and
    iw($g.operation.arguments[2];"source";$a.operation.arguments[3].parts[0].path) and
    lw($g.operation.arguments[4];$a.operation.arguments[4].parts[0].text) and $g.operation.cwd_input=="source" and
    $t.inputs.raw==ref($stage;$p+".generate";"raw") and $t.inputs.source==$g.inputs.source and $t.inputs.perl==$g.inputs.perl and
    $t.operation.stdin_input=="raw" and iw($t.operation.arguments[0];"source";"crypto/perlasm/x86_64-xlate.pl") and lw($t.operation.arguments[1];"elf") and
    $result.inputs.assembly==ref($stage;$p+".translate";"assembly") and lw($result.operation.arguments[0];"--perlasm-result") and
    $reply.inputs.files==ref($stage;$p+".result";"files") and
    $m["unit-00031-script.sources.complete.replies"].inputs["reply-"+$n]==ref($stage;"unit-00031-script.sources.reply-"+$n+".reply";"reply")
  )|not)|$p] as $ring_fail |
[$aws[] | . as $g | (.id|sub("[.]generate$";"")) as $p |
  ($g.id|capture("perlasm-(?<n>[0-9]+)[.]generate$").n) as $n |
  $m[$p+".translate"] as $t | $m["unit-00037-script.generated-package"] as $pack |
  select((
    $g.inputs["original-script"]==ref($stage;"unit-00037-script.perlasm-source";"source") and
    $g.inputs.source==$g.inputs["original-script"] and
    ($g.inputs.perl.action_id|startswith("full-source-native-provider/")) and
    $g.inputs["pipe-source"]==ref($stage;$p+".pipe-source";"source") and
    $g.inputs["aws-as-probe"]==ref($stage;"unit-00037-script.assembler-version";"stderr") and
    iw($g.operation.environment.MANTLE_AWS_AS_PROBE;"aws-as-probe";"") and
    iw($g.operation.arguments[0];"pipe-source";"") and
    $g.operation.arguments[1].parts == [{kind:"input",name:"original-script",path:$g.operation.arguments[2].parts[0].path}] and
    ($g.operation.arguments[2].parts[0].name=="source") and
    ($g.operation.arguments[2].parts[0].path|endswith(".pl")) and
    ($g.operation.arguments[4].parts[0].text|endswith(".S")) and
    $g.operation.cwd_input=="source" and
    $t.inputs.raw==ref($stage;$p+".generate";"raw") and
    $t.inputs.source==$g.inputs.source and $t.inputs.perl==$g.inputs.perl and
    $t.operation.stdin_input=="raw" and
    iw($t.operation.arguments[0];"source";"crypto/perlasm/x86_64-xlate.pl") and lw($t.operation.arguments[1];"elf") and
    $t.inputs["aws-as-probe"]==$g.inputs["aws-as-probe"] and iw($t.operation.environment.MANTLE_AWS_AS_PROBE;"aws-as-probe";"") and
    $pack.inputs["assembly-"+$n]==ref($stage;$p+".translate";"assembly") and
    ($pack.operation.arguments|map(.parts[0])|any(.kind=="output" and .name=="package" and (.path|endswith("/"+$g.operation.arguments[4].parts[0].text))))
  )|not)|$p] as $aws_fail |
[$doc.steps[]|select(.id|test("^unit-00031-script[.]sources[.]perl-[0-9]+[.]|^unit-00037-script[.]perlasm-[0-9]+[.]"))|. as $s|($s.inputs|keys-($s|used))|select(length>0)|{id:$s.id,unused:.}] as $unused |
[$doc.steps[]|select(.id|test("^unit-00031-script[.]sources[.]perl-[0-9]+[.]pipe-source$|^unit-00037-script[.]perlasm-[0-9]+[.]pipe-source$"))|select((.operation.text|contains("$0 = $source;") and contains("changed original PerlAsm pipe boundary") and contains("$count == 1"))|not)|.id] as $pipe_fail |
{
 stage_id:$stage,steps:($doc.steps|length),roots:($doc.roots|length),semantic_roots:($doc.semantic_roots|length),
 ring_generators:($ring|length),aws_generators:($aws|length),
 ring_fail:$ring_fail,aws_fail:$aws_fail,unused_declared_inputs:$unused,pipe_fail:$pipe_fail,
 aws_source:$m["aws-original-source"].inputs.original,
 aws_original_edges:($m["unit-00037-script.perlasm-source"].inputs["aws-original"]==ref($stage;"aws-original-source";"source") and
   $m["unit-00037-script.source-data"].inputs["aws-original"]==ref($stage;"aws-original-source";"source") and
   $m["unit-00037-script.source-data"].inputs["original-package"]=={kind:"source",role:"vendor-inputs",path:"aws-lc-sys"}),
 downstream_edges:($m["unit-00031-script.complete"].inputs.package==ref($stage;"unit-00031-script.generated-package";"package") and
   $m["unit-00037-script.unprefixed.complete"].inputs.package==ref($stage;"unit-00037-script.generated-package";"package"))
}
