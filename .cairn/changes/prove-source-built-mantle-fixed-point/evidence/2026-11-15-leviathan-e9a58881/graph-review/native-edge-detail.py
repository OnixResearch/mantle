#!/usr/bin/env python3
"""Detail the one current Native index edge absent from used_inputs."""
import json
import runpy
from pathlib import Path

SCRATCH = Path('/tmp/luna-e9a58881-expansion-edges-20261115')
COHORT = Path('/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index')
ns = runpy.run_path(str(SCRATCH / 'expansion-edge-audit.py'), run_name='native_edge_detail')
original = ns['used_refs']
def exact_used_refs(step):
    if step['operation']['kind'] != 'exec' or 'command_state' not in step['operation']:
        return original(step)
    cloned = dict(step)
    operation = dict(step['operation'])
    operation.pop('command_state')
    cloned['operation'] = operation
    return original(cloned)

index = json.loads((COHORT / 'program-sources/fixed-point-native-source-program/source-built/native/program.expansion.json').read_bytes())
recipe = json.loads((COHORT / 'program-sources/fixed-point-native-source-program/source-built/native/program.json').read_bytes())
steps = {step['id']: step for step in recipe['steps']}
source_drv = '/mantle/store/rksa2xdbf7r5rjfhckkvhpfpqql8is8y-make-4.4.1-full-source-gcc10-v1.drv'
target_drv = '/mantle/store/9bg51ihqk8kj3lsc3cvkgg9b97g507p4-linux-headers-6.6-full-source-gcc10-v3.drv'
reference = index['derivations'][source_drv]['outputs']['out']
node = index['derivations'][target_drv]
matches = []
for step_id in node['step_ids']:
    step = steps[step_id]
    aliases = [name for name, ref in step['inputs'].items() if ref == reference]
    if aliases:
        actual = exact_used_refs(step)
        operation = step['operation']
        matches.append({
            'step_id': step_id,
            'operation_kind': operation['kind'],
            'matching_aliases': aliases,
            'matched_reference_is_actual_used_input': ns['ref_key'](reference) in actual,
            'all_inputs': step['inputs'],
            'operation': operation,
        })
result = {
    'scope': 'Read-only current Native expansion edge detail; no product execution.',
    'dependency_derivation': source_drv,
    'consumer_derivation': target_drv,
    'selected_output': 'out',
    'reference': reference,
    'consumer_node_step_ids': sorted(node['step_ids']),
    'matching_recipe_steps': matches,
}
out = SCRATCH / 'native-edge-detail.json'
out.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
print(json.dumps(result, indent=2, sort_keys=True))
