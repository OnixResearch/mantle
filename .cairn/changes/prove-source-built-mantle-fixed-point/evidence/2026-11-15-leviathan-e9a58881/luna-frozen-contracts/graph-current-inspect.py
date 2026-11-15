#!/usr/bin/env python3
"""Trusted inert-JSON/source-byte launch gate; no recipe or product is executed.

Usage: python3 graph-current-inspect.py /absolute/remote/scratch/output.json
Run on leviathan using the pinned Python and b3sum binaries in procedure.md.
"""
import collections
import json
import pathlib
import subprocess
import sys

RUN = pathlib.Path('/home/brittonr/sbfp-20261113-berdKH')
COHORT = RUN / 'prepared-cohort-e9a58881-typed-index'
FROZEN = RUN / 'runner-source-e9a58881-typed-index'
B3 = '/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
STAGES = {
    'stagex-transition': 'program-sources/source-built-stagex-programs/stagex-transition.json',
    'stagex-provider-publication': 'program-sources/source-built-stagex-programs/stagex-provider-publication.json',
    'full-source-native-provider': 'program-sources/fixed-point-native-source-program/source-built/native/program.json',
    'full-source-rust-provider': 'program-sources/fixed-point-rust-source-program/source-built/rust/program.json',
    'mantle-stage1': 'inputs/mantle-source/bootstrap/source-built/mantle-stage1.json',
    'mantle-stage2': 'inputs/mantle-source/bootstrap/source-built/mantle-stage2.json',
}
ORDER = {stage: number for number, stage in enumerate(STAGES)}
OPS = {'controller', 'exec', 'write-text', 'copy-text', 'extract-archive', 'materialize-record',
       'materialize-fetcher', 'observe-phase', 'assemble-tree'}
WORD_PARTS = {'literal', 'literal-data', 'input', 'output', 'runtime', 'workspace', 'cwd_relative',
              'directory_separator', 'text_input', 'text_slice'}
CONTROLLER_IMAGES = {'fixed', 'adapted', 'inventory', 'request-output'}
ARTIFACT_KINDS = {'text', 'object', 'archive', 'executable', 'tree'}
SOURCE_ROLES = {'stagex-seed', 'stagex-lineage', 'stagex-source-bundle', 'native-source-bundle', 'rust-source-archive-set', 'mantle-source', 'vendor-inputs', 'imported-native-provider', 'imported-rust-provider', 'prior-mantle-output'}
STATE_ALIASES = {'script', 'original-script', 'native-request', 'native-request-invocation',
                 'native-environment-data', 'native-compiler-files', 'native-temporary'}
TREE_ALIASES = {'cc-work', 'package', 'source', 'native-files', 'native-compiler-files', 'native-temporary'}


def digest(path):
    return {'bytes': path.stat().st_size,
            'blake3': subprocess.run([B3, '--no-names', str(path)], check=True, text=True,
                                     capture_output=True).stdout.strip().split()[0]}


def produced(ref):
    return isinstance(ref, dict) and ref.get('kind') == 'produced' and isinstance(ref.get('action_id'), str) and isinstance(ref.get('output_id'), str) and set(ref) == {'kind', 'action_id', 'output_id'}


def key(ref):
    return ref['action_id'], ref['output_id']


def owner(action):
    return next((stage for stage in STAGES if action.startswith(stage + '/')), None) if isinstance(action, str) else None


def specimen(value, maxlen=250):
    return str(value)[:maxlen]


def inspect():
    issues = {}
    actions = {}  # all six exact physical producers; value: output kinds, stage, ordinal, operation/guard status
    edges = {}  # every actual step's declared dependencies, including implicit observe-phase edges
    result = {'scope': 'read-only static six physical recipe launch gate', 'cohort': str(COHORT),
              'frozen_source': str(FROZEN), 'frozen_code_revision': 'e9a58881b9ff491f47864632367c9b8ea4af76e3',
              'consumer_source_blake3': {name: digest(FROZEN / 'src' / name)['blake3'] for name in (
                  'source_built_fixed_point_stagex_actions.rs', 'source_built_fixed_point_rust_actions.rs',
                  'source_built_recipe.rs', 'source_built_mantle_programs_graph.rs',
                  'source_built_mantle_programs_metadata_helper.rs', 'source_built_recipe_controller.rs',
                  'source_built_recipe_exec.rs', 'source_built_fixed_point_cold.rs',
                  'source_built_fixed_point_native_actions.rs', 'source_built_fixed_point_action_trust.rs',
                  'source_built_mantle_programs.rs')}, 'per_recipe': {}, 'issues': issues}
    totals = collections.Counter()

    def issue(category, context):
        bucket = issues.setdefault(category, {'count': 0, 'samples': []})
        bucket['count'] += 1
        if len(bucket['samples']) < 8:
            bucket['samples'].append(context)

    def ref(stage, where, value, step=None, root=False, nested=False):
        if not isinstance(value, dict) or value.get('kind') not in ('produced', 'source', 'manifest-source'):
            issue('unknown_or_malformed_artifact_variant', {'stage': stage, 'where': where, 'value': specimen(value)})
            return
        if value['kind'] != 'produced':
            expect = {'source': {'kind', 'role', 'path'},
                      'manifest-source': {'kind', 'role', 'record_identity', 'member'}}[value['kind']]
            if set(value) != expect:
                issue('malformed_import_reference', {'stage': stage, 'where': where, 'value': specimen(value)})
            if value.get('role') not in SOURCE_ROLES:
                issue('unknown_source_role_variant', {'stage': stage, 'where': where, 'value': specimen(value)})
            if root:
                issue('nonproduced_root_or_export', {'stage': stage, 'where': where, 'value': specimen(value)})
            return
        if not produced(value) or not value['action_id'] or not value['output_id']:
            issue('malformed_produced_reference', {'stage': stage, 'where': where, 'value': specimen(value)})
            return
        target, output = key(value)
        dest_stage = owner(target)
        info = result['per_recipe'][stage]
        info['produced_reference_occurrences'] += 1
        if nested:
            info['embedded_controller_reference_occurrences'] += 1
        if step:
            edges[step].add(target)
        if root and dest_stage != stage:
            issue('nonlocal_root_export_semantic', {'stage': stage, 'where': where, 'target': target})
        if dest_stage is None:
            issue('unknown_stage_producer', {'stage': stage, 'where': where, 'target': target})
        elif dest_stage != stage:
            info['cross_stage_reference_occurrences'] += 1
            info['cross_stage_by_owner'][dest_stage] += 1
            if ORDER[dest_stage] >= ORDER[stage]:
                issue('cross_stage_not_predecessor', {'stage': stage, 'where': where, 'target': target})
        producer = actions.get(target)
        if producer is None:
            issue('missing_or_late_producer', {'stage': stage, 'where': where, 'target': target, 'output': output})
        elif output not in producer['outputs']:
            issue('unknown_producer_output', {'stage': stage, 'where': where, 'target': target, 'output': output})
        elif step and producer['stage'] == stage and producer['ordinal'] >= actions[step]['ordinal']:
            issue('late_or_cyclic_local_reference', {'stage': stage, 'where': where, 'target': target, 'source': step})
        if step and producer and producer['stage'] != stage and producer['guard'] is not None:
            issue('cross_stage_guarded_input', {'stage': stage, 'where': where, 'target': target})

    def collect_embedded(obj, path='operation.specification'):
        if isinstance(obj, dict):
            if 'kind' in obj and obj['kind'] in ('produced', 'source', 'manifest-source'):
                yield path, obj
            elif 'action_id' in obj or 'output_id' in obj:
                issue('unknown_nested_artifact_variant', {'where': path, 'value': specimen(obj)})
            else:
                for field, value in obj.items():
                    yield from collect_embedded(value, path + '.' + field)
        elif isinstance(obj, list):
            for i, value in enumerate(obj):
                yield from collect_embedded(value, f'{path}[{i}]')

    def guard(stage, step, action):
        op = step['operation']
        value = op.get('guard')
        if value is None:
            return None
        if op.get('kind') not in ('exec', 'controller'):
            issue('invalid_guarded_operation', {'action': action, 'kind': op.get('kind')})
        conditions = value.get('conditions') if isinstance(value, dict) else None
        if not isinstance(conditions, list) or not (1 <= len(conditions) <= 8):
            issue('invalid_guard_condition_count', {'action': action, 'value': specimen(value)})
            return ()
        predicates = []
        seen = set()
        for condition in conditions:
            alias = condition.get('status_input') if isinstance(condition, dict) else None
            codes = condition.get('accepted_codes') if isinstance(condition, dict) else None
            if (not isinstance(condition, dict) or set(condition) != {'status_input', 'accepted_codes'} or
                not isinstance(codes, list) or not 1 <= len(codes) <= 8 or
                not all(type(code) is int and 0 <= code <= 255 for code in codes) or
                any(a >= b for a, b in zip(codes, codes[1:]))):
                issue('invalid_guard_codes', {'action': action, 'condition': specimen(condition)})
            reference = step['inputs'].get(alias)
            if not produced(reference) or reference['output_id'] != 'exit-status':
                issue('invalid_guard_status_reference', {'action': action, 'alias': alias, 'reference': specimen(reference)})
                continue
            producer = actions.get(reference['action_id'])
            if (reference['action_id'] in seen or producer is None or producer['stage'] != stage or
                producer['ordinal'] >= actions[action]['ordinal'] or producer['kind'] not in ('exec', 'controller') or
                producer['outputs'].get('exit-status') != 'text'):
                issue('invalid_guard_status_producer', {'action': action, 'alias': alias, 'producer': reference['action_id']})
            seen.add(reference['action_id'])
            predicates.append((reference['action_id'], tuple(codes) if isinstance(codes, list) else ()))
        return tuple(predicates)

    def check_word(stage, action, word, where):
        if not isinstance(word, dict) or not isinstance(word.get('parts'), list):
            issue('malformed_recipe_word', {'stage': stage, 'action': action, 'where': where})
            return
        for i, part in enumerate(word['parts']):
            if not isinstance(part, dict) or part.get('kind') not in WORD_PARTS:
                issue('unknown_word_part_variant', {'stage': stage, 'action': action, 'where': f'{where}[{i}]', 'value': specimen(part)})

    def local_state(stage, action, alias, reference, category, kind=None):
        if not produced(reference):
            if not isinstance(reference, dict) or reference.get('kind') not in ('source', 'manifest-source'):
                issue('invalid_native_script_state_reference', {'action': action, 'alias': alias, 'category': category})
            return
        result['per_recipe'][stage]['native_state_categories'][category] += 1
        target = actions.get(reference['action_id'])
        if not target or target['stage'] != stage or reference['output_id'] not in target['outputs'] or (kind and target['outputs'][reference['output_id']] != kind):
            issue('nonowning_or_wrong_kind_native_script_state', {'action': action, 'alias': alias, 'category': category,
                                                               'reference': reference, 'expected_kind': kind})

    for stage, relative in STAGES.items():
        file = COHORT / relative
        stats = {'physical_source': str(file), 'physical_identity': digest(file), 'steps': 0, 'roots': 0,
                 'distinct_roots': 0, 'exports': 0, 'semantic_root_keys': 0, 'semantic_root_references': 0,
                 'produced_reference_occurrences': 0, 'embedded_controller_reference_occurrences': 0,
                 'cross_stage_reference_occurrences': 0, 'cross_stage_by_owner': collections.Counter(),
                 'operation_variants': collections.Counter(), 'controller_executable_variants': collections.Counter(),
                 'native_state_categories': collections.Counter(), 'guarded_commands': 0, 'optional_failure_commands': 0,
                 'guarded_direct_inputs': 0, 'optional_failure_direct_inputs': 0, 'phase_observations': 0,
                 'guarded_root_export_semantic': 0, 'optional_failure_roots_exports_semantic': 0,
                 'authenticated_perlasm_steps': 0, 'authenticated_perlasm_consumed': 0,
                 'timeout_groups': 0}
        result['per_recipe'][stage] = stats
        with file.open(encoding='utf-8') as f:
            recipe = json.load(f)
        if recipe.get('schema') != 'mantle-source-recipe-v1' or recipe.get('stage_id') != stage:
            issue('wrong_recipe_schema_or_stage', {'file': str(file), 'schema': recipe.get('schema'), 'stage_id': recipe.get('stage_id')})
        if set(recipe) != {'schema', 'stage_id', 'preparation', 'steps', 'roots', 'exports', 'semantic_roots', 'timeout_groups'}:
            issue('unknown_recipe_fields', {'stage': stage, 'fields': list(recipe)})
        steps = recipe['steps']
        stats['steps'], stats['roots'] = len(steps), len(recipe['roots'])
        stats['distinct_roots'] = len({key(r) for r in recipe['roots'] if produced(r)})
        stats['exports'], stats['semantic_root_keys'] = len(recipe['exports']), len(recipe['semantic_roots'])
        stats['semantic_root_references'] = sum(len(refs) for refs in recipe['semantic_roots'].values())
        stats['timeout_groups'] = len(recipe.get('timeout_groups', []))
        if not 0 < len(steps) <= 100000 or not recipe['roots']:
            issue('recipe_bounds_or_empty_roots', {'stage': stage, 'steps': len(steps), 'roots': len(recipe['roots'])})
        # Producer inventory is derived from the physical file, not a profile or authored plan.
        for n, step in enumerate(steps):
            action = stage + '/' + step['id']
            op = step['operation']
            if action in actions:
                issue('duplicate_action_id', {'action': action})
            outputs = {name: declaration.get('kind') for name, declaration in step['outputs'].items()}
            for name, kind in outputs.items():
                if kind not in ARTIFACT_KINDS:
                    issue('unknown_output_kind', {'action': action, 'output': name, 'kind': kind})
            kind = op.get('kind')
            if kind not in OPS:
                issue('unknown_operation_variant', {'action': action, 'kind': kind})
            stats['operation_variants'][kind] += 1
            policy = op.get('exit_policy') or {}
            if kind == 'exec' and policy.get('kind') not in ('exact', 'one-of', 'normal'):
                issue('unknown_exit_policy_variant', {'action': action, 'policy': specimen(policy)})
            actions[action] = {'stage': stage, 'ordinal': n, 'outputs': outputs, 'kind': kind,
                               'guard': None, 'optional': kind == 'exec' and policy.get('kind') in ('one-of', 'normal')
                               and 'exit-status' in outputs}
            edges[action] = set()
        # Recheck actual input order, source authority and operation/guard edges.
        for step in steps:
            action = stage + '/' + step['id']
            op = step['operation']
            g = guard(stage, step, action)
            actions[action]['guard'] = g
            if g is not None:
                stats['guarded_commands'] += 1
            if actions[action]['optional']:
                stats['optional_failure_commands'] += 1
            for alias, reference in step['inputs'].items():
                ref(stage, 'steps.' + step['id'] + '.inputs.' + alias, reference, action)
                if produced(reference):
                    producer = actions.get(reference['action_id'])
                    if producer and producer['stage'] == stage:
                        ordinary = reference['output_id'] not in ('stdout', 'stderr', 'exit-status')
                        optional = producer['optional'] and ordinary
                        if producer['guard'] is not None or optional:
                            stats['guarded_direct_inputs' if producer['guard'] is not None else 'optional_failure_direct_inputs'] += 1
                            ix = next((n for n, (target, codes) in enumerate(g or ()) if target == reference['action_id']), None)
                            if (g is None or ix is None or (producer['guard'] is not None and
                                    (g[:ix][:len(producer['guard'])] != producer['guard']))):
                                issue('unguarded_or_nondominating_direct_input', {'consumer': action, 'alias': alias, 'reference': reference})
            if op.get('kind') == 'controller':
                spec = op.get('specification', {})
                embedded = list(collect_embedded(spec))
                catalog = {tuple(sorted(value.items())) for where, value in embedded}
                declared = {tuple(sorted(value.items())) for value in step['inputs'].values() if isinstance(value, dict)}
                for where, reference in embedded:
                    ref(stage, step['id'] + '.' + where, reference, action, nested=True)
                    if tuple(sorted(reference.items())) not in declared:
                        issue('unbound_controller_reference', {'action': action, 'where': where, 'reference': reference})
                catalog_refs = {tuple(sorted(value.items())) for name, value in step['inputs'].items()
                                if name.startswith('controller-input-') and isinstance(value, dict)}
                guard_names = {condition.get('status_input') for condition in (op.get('guard') or {}).get('conditions', [])}
                catalog_names = {'controller-input-' + str(n) for n in range(len(catalog))}
                if catalog_refs != catalog or set(step['inputs']) != catalog_names | guard_names:
                    issue('controller_inputs_not_exact_catalog', {'action': action, 'actual_count': len(step['inputs']), 'catalog_count': len(catalog)})
                controller_policy = spec.get('exit_policy') or {}
                if controller_policy.get('kind') not in ('exact', 'one-of', 'normal'):
                    issue('unknown_controller_exit_policy_variant', {'action': action, 'policy': specimen(controller_policy)})
                for slot in spec.get('slots', []):
                    exe = slot.get('executable') if isinstance(slot, dict) else None
                    variant = exe.get('kind') if isinstance(exe, dict) else None
                    stats['controller_executable_variants'][variant] += 1
                    if variant not in CONTROLLER_IMAGES:
                        issue('unknown_controller_executable_variant', {'action': action, 'slot': slot.get('id'), 'variant': variant})
                for where, word in enumerate(spec.get('start_arguments', [])):
                    check_word(stage, action, word, 'controller.start_arguments.' + str(where))
                for where, word in spec.get('environment', {}).items():
                    check_word(stage, action, word, 'controller.environment.' + where)
            elif op.get('kind') == 'exec':
                for where, word in enumerate(op.get('arguments', [])):
                    check_word(stage, action, word, 'arguments.' + str(where))
                for where, word in op.get('environment', {}).items():
                    check_word(stage, action, word, 'environment.' + where)
            if op.get('kind') == 'observe-phase':
                stats['phase_observations'] += 1
                observed = stage + '/' + op['phase']
                producer = actions.get(observed)
                edges[action].add(observed)
                if (not producer or producer['stage'] != stage or producer['ordinal'] >= actions[action]['ordinal'] or
                    producer['guard'] is None):
                    issue('invalid_phase_observation', {'action': action, 'phase': observed})
                state = op.get('state')
                if state is not None:
                    skipped = step['inputs'].get(state.get('skipped_input'))
                    if (not producer or producer['outputs'].get(state.get('output')) != 'tree' or
                        not produced(skipped) or skipped['action_id'] == observed):
                        issue('invalid_phase_observation_state', {'action': action, 'phase': observed, 'state': specimen(state)})
            if stage.startswith('mantle-'):
                native = 'script.native' in step['id']
                perlasm = '.sources.perl-' in step['id']
                for alias, reference in step['inputs'].items():
                    if alias in ('script', 'original-script') or alias.startswith('gcc-phase-environment-'):
                        local_state(stage, action, alias, reference, 'script_text')
                    if native and alias in (STATE_ALIASES - {'script', 'original-script'}):
                        local_state(stage, action, alias, reference, 'native_state')
                    if (native or perlasm) and alias in TREE_ALIASES:
                        local_state(stage, action, alias, reference, 'script_tree', 'tree' if alias == 'cc-work' else None)
                    if native and alias == op.get('cwd_input'):
                        local_state(stage, action, alias, reference, 'script_cwd')
                    if native and alias == op.get('invocation_input'):
                        local_state(stage, action, alias, reference, 'script_invocation', 'text')
                    if native and alias == (op.get('command_state') or {}).get('prior_times_input'):
                        local_state(stage, action, alias, reference, 'script_history', 'text')
                if step['id'].endswith('.authenticate'):
                    stats['authenticated_perlasm_steps'] += 1
        for category, references in (('roots', recipe['roots']), ('exports', list(recipe['exports'].values())),
                                     ('semantic_roots', [r for values in recipe['semantic_roots'].values() for r in values])):
            for i, reference in enumerate(references):
                ref(stage, f'{category}[{i}]', reference, root=True)
                if produced(reference):
                    producer = actions.get(reference['action_id'])
                    if producer and producer['stage'] == stage:
                        if producer['guard'] is not None:
                            stats['guarded_root_export_semantic'] += 1
                            issue('guarded_root_export_semantic', {'stage': stage, 'category': category, 'reference': reference})
                        if producer['optional'] and reference['output_id'] not in ('stdout', 'stderr', 'exit-status'):
                            stats['optional_failure_roots_exports_semantic'] += 1  # permitted by source validator
        for semantic_key, references in recipe['semantic_roots'].items():
            if not references:
                issue('empty_semantic_root', {'stage': stage, 'key': semantic_key})
        queue = [r['action_id'] for r in recipe['roots'] if produced(r)]
        reachable = set()
        while queue:
            current = queue.pop()
            if current not in reachable and current in actions and actions[current]['stage'] == stage:
                reachable.add(current)
                queue.extend(edges[current])
        omitted = sorted(set(stage + '/' + step['id'] for step in steps) - reachable)
        stats['reachable'], stats['unreachable'], stats['unreachable_samples'] = len(reachable), len(omitted), omitted[:8]
        if omitted:
            issue('unreachable_producers', {'stage': stage, 'count': len(omitted), 'sample': omitted[:8]})
        if stage == 'full-source-rust-provider':
            canonical = recipe['exports'].get(stage)
            canonical_ok = (produced(canonical) and canonical['output_id'] == stage and
                            canonical in recipe['roots'] and recipe['semantic_roots'].get(stage) == [canonical] and
                            actions.get(canonical['action_id'], {}).get('outputs', {}).get(stage) == 'tree')
            stats['canonical_rust_provider'] = {'valid': canonical_ok, 'reference': canonical}
            if not canonical_ok:
                issue('noncanonical_rust_provider_export', {'stage': stage, 'reference': canonical})
        if stage in ('stagex-transition', 'stagex-provider-publication'):
            export_name = 'stagex-transition' if stage == 'stagex-transition' else 'stagex-provider'
            canonical = recipe['exports'].get(export_name)
            canonical_ok = (produced(canonical) and canonical['output_id'] == export_name and
                            actions.get(canonical['action_id'], {}).get('outputs', {}).get(export_name) == 'tree')
            stats['canonical_stagex_tree'] = {'valid': canonical_ok, 'reference': canonical}
            if not canonical_ok:
                issue('noncanonical_stagex_tree', {'stage': stage, 'reference': canonical})
        if stage.startswith('mantle-'):
            semantic = recipe['semantic_roots']
            invalid_semantic = [k for k in semantic if not k.startswith(('rust-unit:', 'build-script:')) or
                                (k.startswith('rust-unit:') and len(semantic[k]) != 1) or
                                (k.startswith('build-script:') and {r.get('output_id') for r in semantic[k]} != {'stdout', 'out-dir'})]
            sem_refs = {key(r) for refs in semantic.values() for r in refs if produced(r)}
            binary = recipe['exports'].get(stage)
            smoke = recipe['exports'].get('mantle-smoke')
            roots = [key(r) for r in recipe['roots'] if produced(r)]
            expected = sem_refs | ({key(binary)} if produced(binary) else set()) | ({key(smoke)} if produced(smoke) else set())
            exact = set(roots) == expected and len(roots) == len(set(roots)) and not invalid_semantic
            smoke_step = next((s for s in steps if s['id'] == 'mantle-help'), None)
            smoke_op = smoke_step['operation'] if smoke_step else {}
            smoke_ok = (produced(smoke) and key(smoke) == (stage + '/mantle-help', 'stdout') and
                        smoke_op.get('kind') == 'exec' and smoke_op.get('exit_policy') == {'kind': 'exact', 'code': 0} and
                        smoke_step['inputs'].get(smoke_op.get('executable_input')) == binary and
                        smoke_op.get('arguments') == [{'parts': [{'kind': 'literal', 'text': '--help'}]}])
            binary_ok = (produced(binary) and binary['output_id'] == stage and
                         actions.get(binary['action_id'], {}).get('outputs', {}).get(stage) == 'executable')
            stats['mantle_semantic_contract'] = {'rust_unit_keys': sum(k.startswith('rust-unit:') for k in semantic),
                  'build_script_keys': sum(k.startswith('build-script:') for k in semantic),
                  'distinct_semantic_products': len(sem_refs), 'exact_roots': exact, 'binary_executable': binary_ok,
                  'smoke_executes_binary_help_exact_zero': smoke_ok, 'binary': binary, 'smoke': smoke,
                  'invalid_semantic_key_examples': invalid_semantic[:5],
                  'extra_roots': list(set(roots) - expected)[:5], 'missing_roots': list(expected - set(roots))[:5]}
            if not (exact and binary_ok and smoke_ok):
                issue('invalid_mantle_roots_binary_or_help', {'stage': stage, 'details': stats['mantle_semantic_contract']})
            authenticators = {stage + '/' + s['id'] for s in steps if s['id'].endswith('.authenticate')}
            auth_consumers = {r['action_id'] for s in steps for alias, r in s['inputs'].items()
                              if alias == 'original-script' and produced(r) and r['output_id'] == 'authenticated'}
            stats['authenticated_perlasm_consumed'] = len(authenticators & auth_consumers)
            stats['authenticated_perlasm_reachable'] = len(authenticators & reachable)
            stats['authenticated_perlasm_samples'] = sorted(authenticators)[:3]
            if authenticators - auth_consumers or authenticators - reachable:
                issue('perlasm_authenticator_not_reachable_and_consumed', {'stage': stage,
                      'missing_consumers': sorted(authenticators - auth_consumers)[:5],
                      'unreachable': sorted(authenticators - reachable)[:5]})
        stats['operation_variants'] = dict(stats['operation_variants'])
        stats['controller_executable_variants'] = dict(stats['controller_executable_variants'])
        stats['native_state_categories'] = dict(stats['native_state_categories'])
        stats['cross_stage_by_owner'] = dict(stats['cross_stage_by_owner'])
        stats['outcome'] = 'accept' if not any(True for bucket in issues.values() for sample in bucket['samples']
              if sample.get('stage') == stage or sample.get('action', '').startswith(stage + '/') or
              sample.get('consumer', '').startswith(stage + '/')) else 'reject'
        for field in ('steps', 'roots', 'reachable', 'unreachable', 'produced_reference_occurrences',
                      'embedded_controller_reference_occurrences', 'cross_stage_reference_occurrences',
                      'guarded_commands', 'optional_failure_commands', 'guarded_direct_inputs',
                      'optional_failure_direct_inputs', 'phase_observations', 'guarded_root_export_semantic'):
            totals[field] += stats[field]
        print(json.dumps({'stage': stage, 'identity': stats['physical_identity'], 'steps': len(steps),
                          'reachable': stats['reachable'], 'issues_so_far': {k: v['count'] for k, v in issues.items()}}, sort_keys=True), flush=True)
        del recipe, steps
    # Whole-graph Kahn traversal catches cycles even if a bad cross-stage edge is otherwise masked.
    degree = {source: sum(target in actions for target in dependencies) for source, dependencies in edges.items()}
    successors = collections.defaultdict(list)
    for source, dependencies in edges.items():
        for target in dependencies:
            if target in actions:
                successors[target].append(source)
    queue = collections.deque(source for source, count in degree.items() if not count)
    visited = 0
    while queue:
        node = queue.popleft()
        visited += 1
        for consumer in successors[node]:
            degree[consumer] -= 1
            if not degree[consumer]:
                queue.append(consumer)
    cyclic = sorted(source for source, count in degree.items() if count)
    if cyclic:
        issue('cycle', {'unprocessed_actions': len(cyclic), 'sample': cyclic[:8]})
    totals['cycle_candidates'] = len(cyclic)
    result['total'] = dict(totals)
    result['total']['issues'] = {name: bucket['count'] for name, bucket in issues.items()}
    result['advisory_static_launch_gate'] = 'accept' if not issues else 'reject'
    result['boundary'] = ('A static admission of current physical recipe graph/guard/root/owner references only; '
        'does not authenticate a proof run, actual exit codes, dynamic controller turns/requests/cwd/env, '
        'source-package/lineage correctness, provider behavior, compiler correctness, reconciliation, '
        'fixed-point equality or release eligibility.')
    return result


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: python3 graph-current-inspect.py /remote/scratch/result.json')
    target = pathlib.Path(sys.argv[1])
    data = inspect()
    target.write_text(json.dumps(data, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    print(json.dumps({'report_path': str(target), 'gate': data['advisory_static_launch_gate'],
                      'totals': data['total']}, sort_keys=True), flush=True)
