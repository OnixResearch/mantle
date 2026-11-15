#!/usr/bin/env python3
"""Trace encoded Rust RecipeInvocation data through its immediate frozen consumer."""
import collections
import json
import re
import subprocess
from pathlib import Path

ROOT = Path('/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index')
RECIPE = ROOT / 'inputs/rust-source-archives/source-built/rust/program.json'
TRACE_PATH = Path('/tmp/luna-e9a58881-expansion-edges-20261115/encoded-trace.json')
B3 = '/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
STAGE = 'full-source-rust-provider'

def blake3(path):
    return subprocess.check_output([B3, '--no-names', str(path)], text=True).strip()

def main():
    trace = json.loads(TRACE_PATH.read_text())
    observed = blake3(RECIPE)
    if observed != trace['recipe_blake3']:
        raise RuntimeError(f'actual route recipe hash differs from parent trace: {observed}')
    recipe = json.loads(RECIPE.read_bytes())
    if recipe['stage_id'] != STAGE:
        raise RuntimeError('unexpected routed Rust recipe stage')
    steps = {step['id']: step for step in recipe['steps']}
    if len(steps) != len(recipe['steps']):
        raise RuntimeError('duplicate step ids in routed recipe')
    issues = collections.Counter()
    producer_kinds = collections.Counter()
    output_kinds = collections.Counter()
    consumer_kinds = collections.Counter()
    guarded_consumers = 0
    consumer_count = 0
    samples = []
    producer_map = trace['producers']
    for full_id, item in producer_map.items():
        prefix = STAGE + '/'
        if not full_id.startswith(prefix):
            raise RuntimeError(f'cross-stage candidate producer {full_id}')
        step_id = full_id[len(prefix):]
        producer = steps[step_id]
        operation = producer['operation']
        if operation.get('kind') != 'write-text':
            raise RuntimeError(f'candidate producer is not WriteText: {step_id}')
        producer_kinds[operation['kind']] += 1
        output_id = operation['output']
        if producer['outputs'][output_id]['kind'] != 'text':
            raise RuntimeError(f'encoded request producer is not Text: {step_id}')
        output_kinds[producer['outputs'][output_id]['kind']] += 1
        request = json.loads(operation['text'])
        consumers = item['immediate_consumers']
        if not consumers:
            raise RuntimeError(f'candidate has no immediate Produced consumer: {step_id}')
        verified_consumers = []
        for consumer_info in consumers:
            consumer = steps[consumer_info['step_id']]
            coperation = consumer['operation']
            consumer_kinds[coperation['kind']] += 1
            consumer_count += 1
            if coperation['kind'] != 'exec' or coperation.get('invocation_input') != consumer_info['alias']:
                raise RuntimeError(f'candidate is not the exact Exec invocation_input: {step_id} -> {consumer_info["step_id"]}')
            reference = consumer['inputs'].get(consumer_info['alias'])
            expected = {'kind': 'produced', 'action_id': f'{STAGE}/{step_id}', 'output_id': output_id}
            if reference != expected:
                raise RuntimeError(f'consumer does not read exact Produced request: {step_id} -> {consumer_info["step_id"]}')
            guarded = coperation.get('guard') is not None
            guarded_consumers += int(guarded)
            verified_consumers.append({'step_id': consumer_info['step_id'], 'kind': coperation['kind'], 'alias': consumer_info['alias'], 'guarded': guarded})
        for candidate in item['candidate_locations']:
            location = candidate['where']
            match = re.fullmatch(r'\$\.arguments\[(\d+)\]', location)
            if not match:
                raise RuntimeError(f'unsupported candidate location {location}')
            word = request['arguments'][int(match.group(1))]
            part = word['parts'][0]
            if candidate['issue'] == 'literal_filesystem_path':
                if part['kind'] != 'literal' or not any(ch in part['text'] for ch in ('/', '\\', '\x00')):
                    raise RuntimeError(f'candidate lexical finding does not match encoded word: {step_id} {location}')
            elif candidate['issue'] == 'noncanonical_member':
                if part['kind'] != 'input' or not any(component in ('', '.', '..') for component in part['path'].split('/')):
                    raise RuntimeError(f'candidate member finding does not match encoded word: {step_id} {location}')
            else:
                raise RuntimeError(f'unknown lexical finding {candidate["issue"]}')
            issues[candidate['issue']] += 1
            if len(samples) < 24:
                samples.append({
                    'producer': full_id,
                    'consumers': verified_consumers,
                    'issue': candidate['issue'],
                    'word': word,
                })
    expected_issues = collections.Counter(trace['issues'])
    if issues != expected_issues:
        raise RuntimeError(f'issue totals differ from encoded trace: {issues} != {expected_issues}')
    result = {
        'scope': 'Read-only static trace from current Rust WriteText outputs through exact Produced request input to frozen Exec invocation parser; no product execution.',
        'routed_recipe': str(RECIPE),
        'routed_recipe_blake3': observed,
        'parent_trace_recipe': trace['recipe_path'],
        'parent_trace_recipe_blake3': trace['recipe_blake3'],
        'recipe_bytes': RECIPE.stat().st_size,
        'step_count': len(steps),
        'candidate_producer_count': len(producer_map),
        'immediate_consumer_count': consumer_count,
        'producer_operation_kinds': dict(producer_kinds),
        'producer_output_kinds': dict(output_kinds),
        'consumer_operation_kinds': dict(consumer_kinds),
        'guarded_consumer_count': guarded_consumers,
        'candidate_issue_counts': dict(issues),
        'candidate_word_count': sum(issues.values()),
        'samples': samples,
        'consumer_source_blake3': blake3(Path('/home/brittonr/sbfp-20261113-berdKH/runner-source-e9a58881-typed-index/src/source_built_recipe.rs')),
    }
    out = Path('/tmp/luna-e9a58881-expansion-edges-20261115/dynamic-request-trace.json')
    out.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps(result, indent=2, sort_keys=True))

if __name__ == '__main__':
    main()
