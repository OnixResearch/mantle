#!/usr/bin/env python3
"""Exercise complete current seed traversal and immutable compiler-source rejection."""
import collections
import copy
import json
import runpy
import sys
from pathlib import Path

if len(sys.argv) != 6:
    raise SystemExit('usage: check-seed-source-regressions.py COHORT SOURCE GRAPH_SCRATCH SUBJECT_JSON INSPECTOR_ROOT')
cohort, source, scratch, subject_path, tools = map(Path, sys.argv[1:])
sys.argv = [str(tools / 'graph-review/graph-audit.py'), str(cohort), str(source), str(scratch), str(subject_path)]
controllers = runpy.run_path(str(tools / 'graph-review/controller-template-audit.py'), run_name='current_seed_regression_library')
graph = controllers['audit_controller_templates'].__globals__['G']
selectors = runpy.run_path(str(tools / 'graph-review/mantle-selector-audit.py'), run_name='current_source_regression_library')
seed_check = graph['inspect_output_seeds']
subject = graph['load'](subject_path)


def seed_producer(context, name):
    reference = context['inputs'][name]
    action = reference['action_id']
    namespace = action.split('/', 1)[0]
    if namespace == 'controller-validation':
        producer = context['template_owner'][action]
    elif namespace == context['stage']:
        producer = context['owner'][action]
    else:
        producer = context['prior'][namespace][action]
    return producer, producer['outputs'][reference['output_id']]


class SeedTraversal:
    def __init__(self):
        self.scope = 'raw'
        self.counts = {scope: collections.Counter() for scope in ('raw', 'generated')}
        self.examples = {}

    def observe(self, stage, step_id, inputs, outputs, seeds, owner, prior, step_index,
                guard=None, template_owner=None):
        context = dict(stage=stage, step_id=step_id, inputs=inputs, outputs=outputs,
                       seeds=seeds, owner=owner, prior=prior, step_index=step_index,
                       guard=guard, template_owner=template_owner)
        result = seed_check(**context)
        assert result['invalid'] == 0, (self.scope, stage, step_id, result)
        self.counts[self.scope].update({key: result[key] for key in ('total', 'covered', 'invalid')})
        for output, name in seeds.items():
            reference = inputs[name]
            namespace = reference['action_id'].split('/', 1)[0]
            wanted = stage if self.scope == 'raw' else 'controller-validation'
            if self.scope in self.examples or namespace != wanted:
                continue
            _, declaration = seed_producer(context, name)
            if declaration['bytes_max'] <= 0:
                continue
            saved = dict(context, inputs=dict(inputs), outputs=dict(outputs), seeds=dict(seeds),
                         prior=dict(prior), guard=copy.deepcopy(guard),
                         template_owner=dict(template_owner) if template_owner is not None else None)
            self.examples[self.scope] = (saved, output, name)
        return result


def traverse_current_seeds():
    base = graph['load'](scratch / 'graph-audit.json')
    complete = graph['load'](scratch / 'controller-template-audit.json')
    assert complete['bounded_verdict'] == 'STATIC_TEMPLATE_REVIEW_CLEAR'
    assert all(base['subject_binding']['observed_claim_checks'].values())
    plan = graph['load'](cohort / 'source-program-preparation-input-plan.json')
    assert [row['stage_id'] for row in plan['stages']] == graph['STAGES']
    route_path, route = graph['selected_route']()
    assert route == base['selectors'] and str(route_path) == base['selected_input_blake3']['cold_route']['path']
    keys = dict(zip(graph['STAGES'], ('stagex_transition_recipe', 'publication_recipe',
                    'native_recipe', 'rust_recipe', 'mantle_stage1', 'mantle_stage2'), strict=True))
    traversal = SeedTraversal()
    graph['inspect_output_seeds'] = traversal.observe
    prior = {}
    for index, row in enumerate(plan['stages']):
        stage = row['stage_id']
        selected = base['selected_input_blake3'][keys[stage]]
        path = Path(selected['path'])
        assert path.stat().st_size == selected['size_bytes'] and graph['b3'](path) == selected['blake3']
        recipe = graph['load'](path)
        issues = []
        owner, _ = graph['make_owner'](recipe, index, prior, index, issues)
        traversal.scope = 'raw'
        for position, step in enumerate(recipe['steps']):
            operation = step['operation']
            traversal.observe(stage, step['id'], step.get('inputs', {}), step['outputs'],
                              operation.get('output_seeds', {}), owner, prior, position,
                              operation.get('guard'))
        words = graph['recipe_words'](recipe, owner, prior, index)
        traversal.scope = 'generated'
        allowed = {item['role'] for item in row['inputs'] if item.get('authority') == 'source'}
        result = controllers['audit_controller_templates'](stage, recipe, owner, prior, index, words, issues, allowed)
        assert result['generated_output_seeds']['invalid'] == 0
        prior[stage] = owner
    for scope, report_key in (('raw', 'raw_output_seeds'), ('generated', 'generated_output_seeds')):
        expected = complete['coverage'][report_key]
        assert dict(traversal.counts[scope]) == {key: expected[key] for key in ('total', 'covered', 'invalid')}
        assert traversal.counts[scope]['total'] > 0 and scope in traversal.examples
    return traversal


def reject_seed_case(cases, scope, context, label, expected, mapping, key, value):
    missing = object()
    original = mapping.get(key, missing)
    mapping[key] = value
    try:
        result = seed_check(**context)
        assert result['total'] > 0 and result['invalid'] > 0
        assert result['failure_reason_counts'].get(expected, 0) > 0, (label, result)
        cases.append(dict(scope=scope, case=label, invalid=result['invalid'],
                          rejection_reasons=result['failure_reason_counts'], passed=True))
    finally:
        if original is missing:
            del mapping[key]
        else:
            mapping[key] = original


def mutate_current_seeds(traversal):
    cases = []
    for scope, (context, output, name) in traversal.examples.items():
        producer, declaration = seed_producer(context, name)
        target = context['outputs'][output]
        reference = context['inputs'][name]
        source_bound = declaration['bytes_max']
        def check(label, reason, mapping, key, value):
            reject_seed_case(cases, scope, context, label, reason, mapping, key, value)
        check('unknown-source-kind', 'source or destination artifact kind is unknown', declaration, 'kind', 'unknown-regression-kind')
        check('mismatched-destination-kind', 'Produced source and destination artifact kinds differ', target, 'kind',
              'text' if declaration['kind'] != 'text' else 'tree')
        check('unresolved-source-bound', 'source or destination bytes_max is unresolved/invalid', declaration, 'bytes_max', None)
        check('undersized-destination', 'Produced source bytes_max exceeds destination bytes_max', target, 'bytes_max', source_bound - 1)
        check('unknown-exact-producer', 'exact seed producer is unknown', context['inputs'], name,
              dict(reference, action_id=reference['action_id'].split('/', 1)[0] + '/negative-seed-never-declared'))
        check('unknown-producer-output', 'exact seed producer output is unknown', context['inputs'], name,
              dict(reference, output_id='negative-seed-output-never-declared'))
        check('undeclared-seed-input', 'seed input is not a genuine nonguard Produced reference', context['seeds'], output,
              'negative-seed-input-never-declared')
        guard = copy.deepcopy(context['guard']) if context['guard'] else {'conditions': []}
        guard.setdefault('conditions', []).append({'status_input': name})
        check('guard-input-as-seed', 'seed input is not a genuine nonguard Produced reference', context, 'guard', guard)
        check('stream-output-as-seed', 'seed target is not a declared ordinary output', context, 'seeds', {'stdout': name})
        if scope == 'raw':
            check('nonprior-local-producer', 'seed producer is not a prior local declaration', producer, 'index', context['step_index'])
    assert len(cases) == 19 and all(case['passed'] for case in cases)
    return cases


def immutable_source_cases():
    root = graph['source_role_root']('mantle-source')
    documents = {stage: graph['load'](graph['program_member_path'](f'bootstrap/source-built/{stage}.unit-plan.json'))
                 for stage in selectors['STAGES']}
    check = selectors['immutable_root_source_check']
    binary = subject['inert_tools']['b3sum']
    positive = check(documents, root, binary)
    assert positive['all_match'] and positive['root_unit_count'] > 0
    assert positive['actual_immutable_compiler_source']['file_count'] > 0
    cases = []
    for stage in selectors['STAGES']:
        derivations = documents[stage]['unit_graph']['derivations']
        roots = [unit for unit in derivations if unit.get('package_id') == 'path+native#mantle@0.1.0']
        assert roots
        unit = roots[0]
        original = unit['source_digest']
        value = original['value']
        unit['source_digest'] = dict(original, value=('1' if value[0] == '0' else '0') + value[1:])
        try:
            result = check(documents, root, binary)
            mismatches = [item for values in result['pre_lowering_root_units'].values() for item in values
                          if not item['matches_actual_compiler_source']]
            assert not result['all_match'] and result['root_unit_count'] == positive['root_unit_count']
            assert len(mismatches) == 1 and mismatches[0]['unit_id'] == unit['unit_id']
            assert result['actual_immutable_compiler_source'] == positive['actual_immutable_compiler_source']
            cases.append(dict(stage=stage, case='changed-root-unit-source-digest', unit_id=unit['unit_id'], rejected=True))
        finally:
            unit['source_digest'] = original
        documents[stage]['unit_graph']['derivations'] = [item for item in derivations if item not in roots]
        try:
            check(documents, root, binary)
        except ValueError as error:
            assert str(error) == f'{stage}: no pre-lowering root-package derivations'
            cases.append(dict(stage=stage, case='empty-root-unit-set', rejection=str(error), rejected=True))
        else:
            raise AssertionError('empty root-unit set passed immutable source inspection')
        finally:
            documents[stage]['unit_graph']['derivations'] = derivations
    assert len(cases) == 4 and all(case['rejected'] for case in cases)
    return dict(positive=positive, negative_cases=cases)


def main():
    traversal = traverse_current_seeds()
    seed_cases = mutate_current_seeds(traversal)
    immutable = immutable_source_cases()
    result = dict(schema='mantle-current-seed-source-regressions-v1',
                  subject_blake3=graph['b3'](subject_path), source_revision=subject['frozen_product_revision'],
                  complete_seed_coverage={scope: dict(counts) for scope, counts in traversal.counts.items()},
                  seed_negative_cases=seed_cases, immutable_compiler_source=immutable,
                  inspector_hashes={name: graph['b3'](tools / 'graph-review' / name)
                                    for name in ('graph-audit.py', 'controller-template-audit.py', 'mantle-selector-audit.py')},
                  probe_blake3=graph['b3'](Path(__file__)), all_assertions_passed=True,
                  non_claim='Complete actual-current-data inert seed traversal and in-memory rejection cases only. Compiler source was hashed in full with unchanged Rust framing; no sampled source set or file exception. Frozen source/cohort were not modified. No product, helper, compiler, controller, provider, recipe or protected command executed.')
    output = scratch / 'output-seed-immutable-source-regressions.json'
    with output.open('x') as stream:
        json.dump(result, stream, indent=2, sort_keys=True)
        stream.write('\n')
    print(json.dumps(dict(output=str(output), complete_seed_coverage=result['complete_seed_coverage'],
                          seed_negative_cases=len(seed_cases), immutable_source_negative_cases=len(immutable['negative_cases']),
                          all_assertions_passed=True)))


if __name__ == '__main__':
    main()
