#!/usr/bin/env python3
"""Exercise actual newly authored unit facts and selectors; development evidence only."""
import copy
import json
import runpy
import stat
import subprocess
import sys
from pathlib import Path, PurePosixPath
root = Path('/home/brittonr/sbfp-20261113-berdKH')
cohort = root/'prepared-cohort-unit-plan-development-20261116'
inspector = root/'review-tools-unit-plan-development-v10/graph-review/mantle-selector-audit.py'
a = runpy.run_path(str(inspector), run_name='unit_plan_smoke_library')
b3 = '/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
def digest(path):
    return subprocess.check_output([b3, '--no-names', str(path)], text=True).strip()
assert len(sys.argv) == 2
stream_blake3 = sys.argv[1]
stages = a['STAGES']
members = {stage: f'bootstrap/source-built/{stage}.unit-plan.json' for stage in stages}
source_root = cohort/'inputs/mantle-source'
recipes = {stage: json.loads((source_root/f'bootstrap/source-built/{stage}.json').read_bytes()) for stage in stages}
documents = {stage: json.loads((source_root/member).read_bytes()) for stage, member in members.items()}
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
output = root/'unit-plan-selectors-development-v10-20261116.json'
record = {'scope': 'Mutable developer authoring against genuine predecessor inputs only; not frozen qualification, independent acceptance, protected execution or proof.',
          'preparation_task': 1667, 'source_plan_projections': projections,
          'positive': positive, 'inspector_blake3': digest(inspector)}
if not positive['bounded_verdict'] or any(x.get('uncovered_count', len(x.get('uncovered', []))) for x in projections.values()):
    with output.open('x') as stream: json.dump(record, stream, indent=2); stream.write('\n')
    print(json.dumps({'bounded_verdict': False, 'coverage': positive['coverage'], 'projections': projections, 'diagnostics': positive['diagnostics'][:12]}))
    raise SystemExit(1)
profile = cohort/'source-profile.json'
preparation = json.loads((cohort/'source-program-preparation.json').read_bytes())
subject = {'profile_file': str(profile), 'profile_file_bytes': profile.stat().st_size,
           'profile_file_raw_blake3': stream_blake3, 'full_profile_ssh_stream_blake3': stream_blake3,
           'prepared_manifest_blake3': preparation['output_profile_blake3']}
record['authenticated_unit_plans'] = a['member_record_bindings'](cohort, subject, list(members.values()), digest, b3)
record['actual_profile_stream_blake3'] = stream_blake3
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
record['negative_cases'] = cases
record['all_assertions_passed'] = True
with output.open('x') as stream: json.dump(record, stream, indent=2); stream.write('\n')
print(json.dumps({'bounded_verdict': True, 'coverage': positive['coverage'], 'source_plan_projections': projections, 'negative_case_count': len(cases), 'all_assertions_passed': True, 'output': str(output)}))
