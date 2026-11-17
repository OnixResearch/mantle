#!/usr/bin/env python3
"""Full-current-data negative helper-authority exercise; never execute candidates."""
import copy
import json
import runpy
import sys
from pathlib import Path
if len(sys.argv) != 5:
    raise SystemExit('usage: check-helper-mutations.py COHORT FROZEN_SOURCE GRAPH_SCRATCH SUBJECT_JSON')
cohort, source, scratch, subject_path = map(Path, sys.argv[1:])
root = cohort.parent
inspector = Path(__file__).resolve().parent/'graph-review/helper-authority-audit.py'
sys.argv = [str(inspector), str(cohort), str(source), str(scratch), str(subject_path)]
a = runpy.run_path(str(inspector), run_name='helper_mutation_library')
g = runpy.run_path(str(inspector.with_name('graph-audit.py')), run_name='helper_mutation_graph_library')
subject = json.loads(subject_path.read_bytes())
graph_report = json.loads((scratch/'graph-audit.json').read_bytes())
selected = graph_report['selected_input_blake3']
positive = json.loads((scratch/'helper-authority-audit.json').read_bytes())
assert positive['bounded_verdict'] == 'CLEAR'
assert positive['inspector_binding']['blake3'] == g['b3'](inspector)
assert positive['input_bindings']['subject_json']['blake3'] == g['b3'](subject_path)
recipes = {stage: json.loads(Path(selected[key]['path']).read_bytes()) for stage, key in zip(a['STAGES'], a['RECIPE_KEYS'], strict=True)}
plan = json.loads(Path(selected['plan']['path']).read_bytes())

def own_source(stage):
    record = a['_author_record'](source, stage, subject)
    for step in recipes[stage]['steps']:
        if step['operation']['kind'] == 'exec':
            for name, ref in step.get('inputs', {}).items():
                if ref.get('kind') == 'manifest-source' and ref.get('record_identity') == record:
                    return step, name, ref
    raise AssertionError('no real owning source operand for ' + stage)

stage = 'mantle-stage1'
row = next(s for s in plan['stages'] if s['stage_id'] == stage)
before = len(row['inputs'])
row['inputs'] = [x for x in row['inputs'] if not (x.get('authority') == 'source' and x.get('role') == 'native-source-bundle')]
assert len(row['inputs']) == before - 1
step, name, ref = own_source('mantle-stage2')
step['inputs'][name] = dict(ref, record_identity='fixed-point-rust-source-program')
step, name, ref = own_source('full-source-native-provider')
other = next(r for s in recipes['full-source-native-provider']['steps'] for r in s.get('inputs', {}).values()
             if r.get('kind') == 'manifest-source' and r.get('record_identity') == ref['record_identity'] and r.get('member') != ref['member'])
step['inputs'][name] = copy.deepcopy(other)
phase_map = next(s for s in recipes['full-source-rust-provider']['steps'] if s['id'] == 'rust-source-phase-helper.compile.paths-0')
old = phase_map['operation']['text']
phase_map['operation']['text'] = old.replace('/inputs/helper-source\t/inputs/helper-source\n', '/inputs/helper-source\t/inputs/native-provider\n')
assert phase_map['operation']['text'] != old
stage = 'stagex-provider-publication'
removed, _, _ = own_source(stage)
recipes[stage]['steps'] = [s for s in recipes[stage]['steps'] if s['id'] != removed['id']]
stdin_step = next(s for s in recipes['stagex-transition']['steps'] if s['id'] == 'tinycc27-source-patch-0')
assert stdin_step['operation']['stdin_input'] == 'patch'
stdin_step['operation']['stdin_input'] = None

result = a['audit_recipes'](recipes, plan, subject, source, cohort, g)
assert result['bounded_verdict'] == 'HOLD'
checks = [
    ('missing-owning-source-role', 'mantle-stage1', 'invalid'),
    ('foreign-source-record', 'mantle-stage2', 'invalid'),
    ('different-authenticated-own-member', 'full-source-native-provider', 'invalid'),
    ('gcc-map-wrong-destination', 'full-source-rust-provider', 'invalid'),
    ('missing-helper-source-producer', 'stagex-provider-publication', 'uncovered'),
    ('missing-stdin-source-consumer', 'stagex-transition', 'invalid'),
]
cases = []
for label, stage, classification in checks:
    observation = result['stages'][stage]
    assert observation[classification] > 0, (label, observation)
    cases.append({'case': label, 'stage': stage, 'classification': classification,
                  'rejected_count': observation[classification], 'passed': True})
output = scratch/'helper-authority-negative.json'
with output.open('x') as stream:
    json.dump({'cases': cases, 'all_assertions_passed': True, 'mutated_data_verdict': result['bounded_verdict'],
               'scope': 'Six independent in-memory mutations over all six genuine current recipes; immutable inputs unchanged; no candidate execution.',
               'inspector_blake3': g['b3'](inspector)}, stream, indent=2)
    stream.write('\n')
print(json.dumps({'case_count': len(cases), 'all_assertions_passed': True, 'output': str(output)}))
