#!/usr/bin/env python3
"""Exercise actual newly authored unit facts and selectors; development evidence only."""
import copy
import json
import runpy
import stat
import subprocess
import sys
from pathlib import Path, PurePosixPath
if len(sys.argv) != 5:
    raise SystemExit('usage: check-selector-mutations.py COHORT FROZEN_SOURCE GRAPH_SCRATCH SUBJECT_JSON')
cohort, source, scratch, subject_path = map(Path, sys.argv[1:])
root = cohort.parent
subject = json.loads(subject_path.read_bytes())
inspector = Path(__file__).resolve().parent/'graph-review/mantle-selector-audit.py'
graph_script = inspector.with_name('graph-audit.py')
sys.argv = [str(graph_script), str(cohort), str(source), str(scratch), str(subject_path)]
g = runpy.run_path(str(graph_script), run_name='selector_mutation_graph_library')
a = runpy.run_path(str(inspector), run_name='unit_plan_smoke_library')
b3 = subject['inert_tools']['b3sum']
def digest(path):
    return subprocess.check_output([b3, '--no-names', str(path)], text=True).strip()
stages = a['STAGES']
members = {stage: f'bootstrap/source-built/{stage}.unit-plan.json' for stage in stages}
source_root = cohort/'inputs/mantle-source'
_, route = g['selected_route']()
recipes = {stage: json.loads(g['program_member_path'](member).read_bytes()) for stage, member in zip(stages, g['MANTLE_MEMBERS'], strict=True)}
documents = {stage: json.loads(g['program_member_path'](member).read_bytes()) for stage, member in members.items()}
facts, projections = a['unit_plan_facts'](documents, recipes)
role_roots = {'mantle-source': source_root, 'vendor-inputs': cohort/'inputs/vendor-deps'}
def source_member(role, member):
    if role not in role_roots or not isinstance(member, str): return False
    rel = PurePosixPath(member)
    if not member or rel.is_absolute() or rel.as_posix() != member or any(x in ('.', '..') for x in rel.parts): return False
    path = role_roots[role]
    try:
        for part in rel.parts:
            path /= part
            mode = path.lstat().st_mode
            if stat.S_ISLNK(mode) or not (stat.S_ISREG(mode) or stat.S_ISDIR(mode)): return False
        return path.is_file()
    except OSError:
        return False
positive = a['audit_data'](recipes, facts, source_member)
output = scratch/'selector-mutations.json'
record = {'scope': 'Authenticated exact-current-cohort inert selector comparison and mutations only; no product/helper/compiler/provider/controller execution, protected proof or release.',
          'source_revision': subject['frozen_product_revision'], 'source_plan_projections': projections,
          'positive': positive, 'inspector_blake3': digest(inspector)}
if not positive['bounded_verdict'] or any(x.get('uncovered_count', len(x.get('uncovered', []))) for x in projections.values()):
    with output.open('x') as stream: json.dump(record, stream, indent=2); stream.write('\n')
    print(json.dumps({'bounded_verdict': False, 'coverage': positive['coverage'], 'projections': projections, 'diagnostics': positive['diagnostics'][:12]}))
    raise SystemExit(1)
record['complete_profile_observation'] = a['profile_claim_observation'](cohort, subject, digest)
record['authenticated_program_members'] = a['member_record_bindings'](subject, [g['ROUTE_MEMBER'], *g['MANTLE_MEMBERS'], *members.values()], g)
record['actual_profile_stream_blake3'] = subject['full_profile_ssh_stream_blake3']
cases = []
def rejected(label, category):
    result = a['audit_data'](recipes, facts, source_member)
    bucket = result['coverage'][category]
    assert result['bounded_verdict'] is False and bucket['invalid'] + bucket['uncovered'] > 0, (label, bucket)
    cases.append({'case': label, 'category': category, 'invalid': bucket['invalid'], 'uncovered': bucket['uncovered'], 'passed': True})
recipe = recipes[stages[0]]
original = next(s for s in recipe['steps'] if s['id'].endswith('-script-original-package'))
args = original['operation']['arguments']
original['operation']['arguments'] = args[:-1]
try: rejected('missing-package-exclusion', 'package_exclusions')
finally: original['operation']['arguments'] = args
word = args[3]
args[3] = {'parts': [{'kind': 'literal-data', 'text': 'src/grammar.rs'}]}
try: rejected('untyped-grammar-exclusion', 'package_exclusions')
finally: args[3] = word
compile_step = next(s for s in recipe['steps'] if s['id'].endswith('-compile') and any(isinstance(a['literal'](w), str) and a['literal'](w).startswith('feature=') for w in s.get('operation', {}).get('arguments', [])))
feature_word = next(w for w in compile_step['operation']['arguments'] if isinstance(a['literal'](w), str) and a['literal'](w).startswith('feature='))
text = feature_word['parts'][0]['text']
feature_word['parts'][0]['text'] = 'feature="unowned-negative-feature"'
try: rejected('changed-compiler-feature-cfg', 'feature_cfg')
finally: feature_word['parts'][0]['text'] = text
script = next(s for s in recipe['steps'] if s['id'].endswith('-script') and any(k.startswith('CARGO_FEATURE_') for k in s.get('operation', {}).get('environment', {})))
env = script['operation']['environment']
key = next(k for k in env if k.startswith('CARGO_FEATURE_'))
value = env[key]
env[key] = {'parts': [{'kind': 'literal', 'text': '0'}]}
try: rejected('changed-cargo-feature-environment', 'feature_environment')
finally: env[key] = value
nested = {'id': 'negative-unowned-static-selector', 'inputs': {}, 'outputs': {'template': {'kind': 'text', 'bytes_max': 4096}},
          'operation': {'kind': 'write-text', 'output': 'template', 'text': json.dumps({'invocation': {'arguments': [{'parts': [{'kind': 'literal', 'text': '--cfg'}]}, {'parts': [{'kind': 'literal', 'text': 'feature="unowned-negative-feature"'}]}], 'environment': {}}})}}
recipe['steps'].append(nested)
try: rejected('unowned-nested-feature-selector', 'feature_cfg')
finally: recipe['steps'].pop()
index = original['id'][5:10]
consumer = next(s for s in recipe['steps'] if s['id'] == f'unit-{index}-compile')
ref = consumer['inputs']['source-package']
consumer['inputs']['source-package'] = dict(ref, role='mantle-source')
try: rejected('foreign-original-package-source', 'package_exclusions')
finally: consumer['inputs']['source-package'] = ref
continuation = next(s for s in recipe['steps'] if a['SCRIPT_SCOPE'].match(s['id']) and not s['id'].endswith('-script') and any(k.startswith('CARGO_FEATURE_') for k in s.get('operation', {}).get('environment', {})))
environment = continuation['operation']['environment']
removed = {k: environment.pop(k) for k in list(environment) if k.startswith('CARGO_FEATURE_')}
try: rejected('missing-continuation-feature-environment', 'feature_environment')
finally: environment.update(removed)
for template in recipe['steps']:
    op = template.get('operation', {})
    if op.get('kind') != 'write-text' or not isinstance(op.get('text'), str): continue
    try: document = json.loads(op['text'])
    except ValueError: continue
    contexts = [nested for _, nested in a['nested_operations'](document) if any(k.startswith('CARGO_FEATURE_') for k in nested.get('environment', {}))]
    if contexts: break
else: raise AssertionError('actual selected recipe has no feature-bearing static invocation template')
original_text = op['text']
environment = contexts[0]['environment']
for key in list(environment):
    if key.startswith('CARGO_FEATURE_'): del environment[key]
op['text'] = json.dumps(document)
try: rejected('missing-template-feature-environment', 'feature_environment')
finally: op['text'] = original_text
unit_id = facts[stages[0]][index]['unit_id']
root_key = 'build-script:' + unit_id
saved_root = recipe['semantic_roots'].pop(root_key)
try: rejected('missing-source-selected-build-script-root', 'feature_environment')
finally: recipe['semantic_roots'][root_key] = saved_root
primary_index = next(index for index, fact in facts[stages[0]].items()
                     if fact['package_id'] == 'path+native#snix-build@0.1.0' and fact['target_kind'] == 'custom-build')
primary_compile = next(step for step in recipe['steps'] if step['id'] == f'unit-{primary_index}-compile')
primary_arguments = primary_compile['operation']['arguments']
primary_fact = facts[stages[0]][primary_index]
primary_word_index = next(i for i, word in enumerate(primary_arguments)
                          if a['exact_primary_source_consumed']([word], primary_compile['inputs'], primary_fact))
primary_word = primary_arguments[primary_word_index]
primary_arguments[primary_word_index] = {'parts': [{'kind': 'literal', 'text': primary_fact['primary_path']}]}
try: rejected('literal-primary-source-has-no-input-authority', 'primary_sources')
finally: primary_arguments[primary_word_index] = primary_word
primary_arguments[primary_word_index] = {'parts': [*primary_word['parts'], {'kind': 'literal', 'text': ''}]}
try: rejected('concatenated-primary-source-is-not-exact-input', 'primary_sources')
finally: primary_arguments[primary_word_index] = primary_word
primary_name = primary_word['parts'][0]['name']
primary_reference = primary_compile['inputs'][primary_name]
primary_compile['inputs'][primary_name] = dict(primary_reference, role='vendor-inputs')
try: rejected('foreign-primary-source-role', 'primary_sources')
finally: primary_compile['inputs'][primary_name] = primary_reference
planned_unit = next(unit for unit in documents[stages[0]]['unit_graph']['derivations']
                    if unit['unit_id'] == primary_fact['unit_id'])
planned_arguments = planned_unit['derivation']['args']
planned_primary_index = next(i for i, arg in enumerate(planned_arguments) if not arg.startswith('-') and arg.endswith('.rs'))
planned_primary = planned_arguments[planned_primary_index]
planned_path = PurePosixPath(planned_primary)
assert planned_path.parent.name, 'actual snix-build primary must identify its owning source directory'
planned_arguments[planned_primary_index] = (planned_path.parent / '..' / planned_path.parent.name / planned_path.name).as_posix()
saved_facts = facts
try:
    facts, mutated_projection = a['unit_plan_facts'](documents, recipes)
    assert not any(row.get('uncovered_count', len(row.get('uncovered', []))) for row in mutated_projection.values())
    assert '..' in PurePosixPath(facts[stages[0]][primary_index]['primary_path']).parts
    rejected('source-plan-parent-path-alias-cannot-pass-as-canonical-primary', 'primary_sources')
finally:
    facts = saved_facts
    planned_arguments[planned_primary_index] = planned_primary
record['negative_cases'] = cases
record['all_assertions_passed'] = True
with output.open('x') as stream: json.dump(record, stream, indent=2); stream.write('\n')
print(json.dumps({'bounded_verdict': True, 'coverage': positive['coverage'], 'source_plan_projections': projections, 'negative_case_count': len(cases), 'all_assertions_passed': True, 'output': str(output)}))
