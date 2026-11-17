#!/usr/bin/env bash
set -euo pipefail
if (($# != 4)); then
  printf 'usage: %s COHORT FROZEN_SOURCE REVIEW_SCRATCH SUBJECT_JSON\n' "$0" >&2
  exit 2
fi
COHORT=$1
SOURCE=$2
SCRATCH=$3
SUBJECT=$4
HERE=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
PYTHON=/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14
test -f "$SUBJECT"
test -f "$HERE/graph-audit.py"
test -f "$HERE/controller-template-audit.py"
mkdir -p -- "$SCRATCH"
test ! -e "$SCRATCH/execution.log"
exec > "$SCRATCH/execution.log" 2>&1
trap 'status=$?; printf "semantic_root_inspection_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
export PYTHONDONTWRITEBYTECODE=1
"$PYTHON" "$HERE/graph-audit.py" "$COHORT" "$SOURCE" "$SCRATCH" "$SUBJECT"
test -s "$SCRATCH/graph-audit.json"
test -s "$SCRATCH/source-binding-kind-inventory.json"
"$PYTHON" "$HERE/controller-template-audit.py" "$COHORT" "$SOURCE" "$SCRATCH" "$SUBJECT"
test -s "$SCRATCH/controller-template-audit.json"
"$PYTHON" - "$HERE/graph-audit.py" "$COHORT" "$SOURCE" "$SCRATCH" "$SUBJECT" <<'PY'
import json,runpy,sys
from pathlib import Path
module,cohort,source,scratch,subject=sys.argv[1:]
sys.argv=[module,cohort,source,scratch,subject]
audit=runpy.run_path(module,run_name='inert_semantic_root_regression')
route=json.loads((Path(cohort)/'inputs/mantle-source/bootstrap/source-built-cold-route.json').read_bytes())
cases=[]
for relative in route['mantle_recipes']:
    recipe=json.loads((Path(cohort)/'inputs/mantle-source'/relative).read_bytes())
    stage=recipe['stage_id']; binary=recipe['exports'][stage]
    owners={stage+'/'+step['id']:{'outputs':step['outputs']} for step in recipe['steps']}
    def check(label,candidate,expected_match):
        result=audit['mantle_root_report'](stage,candidate,{stage:owners})
        assert result['binary_output_kind']=='executable',label
        assert result['help_consumer_count']==1,label
        assert result['smoke_export_actual_producer_valid'],label
        assert result['recipe_roots_exact_semantic_binary_smoke_union'],label
        assert result['binary_is_exact_mantle_semantic_product']==expected_match,label
        assert result['root_and_consumer_failure_count']==int(not expected_match),label
        cases.append({'stage':stage,'case':label,'canonical_match':result['binary_is_exact_mantle_semantic_product'],'unit_keys':result['mantle_unit_key_candidates'],'failure_count':result['root_and_consumer_failure_count'],'other_consumer_checks_passed':True})
        return result
    original=check('actual_selected_recipe',recipe,True)
    qualified=original['mantle_unit_key_candidates'][0]
    def candidate(semantic):
        roots={audit['key'](ref):ref for refs in semantic.values() for ref in refs}
        for ref in (binary,recipe['exports']['mantle-smoke']): roots[audit['key'](ref)]=ref
        return {**recipe,'semantic_roots':semantic,'roots':list(roots.values())}
    missing={name:refs for name,refs in recipe['semantic_roots'].items() if name!=qualified}
    check('missing_qualified_unit',candidate(missing),False)
    duplicate_parts=qualified.split(':'); duplicate_parts[2]='0'*64
    duplicate=':'.join(duplicate_parts)
    assert duplicate!=qualified
    check('nonunique_qualified_units',candidate({**recipe['semantic_roots'],duplicate:[binary]}),False)
    replacement=next({'kind':'produced','action_id':action,'output_id':output} for action,owner in owners.items() for output,bound in owner['outputs'].items() if bound['kind']=='executable' and {'kind':'produced','action_id':action,'output_id':output}!=binary)
    check('wrong_produced_binary',candidate({**recipe['semantic_roots'],qualified:[replacement]}),False)
    check('shorthand_unit_not_qualified',candidate({**missing,'rust-unit:mantle':[binary]}),False)
result={'schema':'mantle-semantic-root-regression-v1','inspector_blake3':audit['b3'](Path(module)),'selected_recipes':len(route['mantle_recipes']),'cases':cases,'non_claim':'Inert recipe-data validation only. Mutations are in-memory fixtures; actual source/cohort and expected pins are unchanged. No product, compiler or protected execution.'}
(Path(scratch)/'semantic-root-regression.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
print(json.dumps({'semantic_root_regression_cases':len(cases),'actual_selected_recipes':len(route['mantle_recipes']),'all_assertions_passed':True}))
PY
test -s "$SCRATCH/semantic-root-regression.json"
printf 'semantic_root_inspection_complete=1\n'
