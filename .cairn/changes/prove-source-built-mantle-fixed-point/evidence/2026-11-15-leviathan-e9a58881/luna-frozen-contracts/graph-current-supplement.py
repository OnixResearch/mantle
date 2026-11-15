#!/usr/bin/env python3
"""Supplementary static read of guarded/native state and StageX publication, no products run.

Usage: python3 graph-current-supplement.py main-report.json supplement.json
"""
import collections
import json
import pathlib
import sys

ROOT = pathlib.Path('/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index')
B3 = '/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
PATHS = {
    'stagex-transition': 'program-sources/source-built-stagex-programs/stagex-transition.json',
    'stagex-provider-publication': 'program-sources/source-built-stagex-programs/stagex-provider-publication.json',
    'full-source-native-provider': 'program-sources/fixed-point-native-source-program/source-built/native/program.json',
    'mantle-stage1': 'inputs/mantle-source/bootstrap/source-built/mantle-stage1.json',
    'mantle-stage2': 'inputs/mantle-source/bootstrap/source-built/mantle-stage2.json',
}

def key(ref):
    if isinstance(ref, dict) and ref.get('kind') == 'produced':
        return ref['action_id'], ref['output_id']
    return None


def read(stage, baseline):
    path = ROOT / PATHS[stage]
    import subprocess
    size = path.stat().st_size
    hash_ = subprocess.run([B3, '--no-names', str(path)], check=True, text=True,
                           capture_output=True).stdout.strip().split()[0]
    assert {'bytes': size, 'blake3': hash_} == baseline['per_recipe'][stage]['physical_identity']
    return json.loads(path.read_text())


def inspect(baseline):
    problems = {}
    report = {'scope': 'Supplement to six-recipe inert static graph review', 'issues': problems,
              'per_recipe': {}, 'stagex_publication': {}, 'native_provider': {}}

    def issue(kind, context):
        bucket = problems.setdefault(kind, {'count': 0, 'samples': []})
        bucket['count'] += 1
        if len(bucket['samples']) < 8:
            bucket['samples'].append(context)

    transition = read('stagex-transition', baseline)
    publication = read('stagex-provider-publication', baseline)
    index_path = ROOT / 'program-sources/source-built-stagex-programs/stagex-provider-publication.json.expansion.json'
    index = json.loads(index_path.read_text())
    assert index['schema'] == 'mantle-stagex-source-expansion-v1' and index['recipe_member'] == 'stagex-provider-publication.json'
    binutils = index['binutils_admitted']
    expected_tools = ['tcc', 'tcc-0.9.26', 'tcc-selfhost']
    if binutils:
        expected_tools += ['ar', 'as', 'ld', 'nm', 'objcopy', 'objdump', 'readelf', 'size', 'strings', 'strip']
    steps = {'stagex-provider-publication/' + s['id']: s for s in publication['steps']}
    transition_output = transition['exports']['stagex-transition']
    canonical = publication['exports']['stagex-provider']

    def used_refs(step):
        op = step['operation']
        names = set()
        kind = op['kind']
        if kind == 'exec':
            names.add(op['executable_input'])
            names.update(name for name in (op.get('cwd_input'), op.get('stdin_input'), op.get('invocation_input')) if name)
            names.update(op.get('runtime_mounts', {}).values())
            names.update(op.get('output_seeds', {}).values())
            names.update(c['status_input'] for c in (op.get('guard') or {}).get('conditions', []))
            words = op.get('arguments', []) + list(op.get('environment', {}).values())
            if op.get('invocation_input') or any(word.get('parts', [])[:1] == [{'kind': 'literal', 'text': '@'}]
                    and len(word.get('parts', [])) == 2 and word['parts'][1]['kind'] == 'input' for word in words):
                names.update(step['inputs'])
            for word in words:
                for part in word['parts']:
                    if part['kind'] in ('input', 'text_input', 'text_slice'):
                        names.add(part['name'])
                    elif part['kind'] == 'runtime' and part['mount'] in op.get('runtime_mounts', {}):
                        names.add(op['runtime_mounts'][part['mount']])
        elif kind == 'assemble-tree':
            names.update(op['entries'].values())
        else:
            issue('unknown_publication_operation', {'step': step['id'], 'kind': kind})
        return [step['inputs'][name] for name in names if name in step['inputs']]

    def depends_on(product, prerequisite):
        if product == prerequisite:
            return True
        pending = [product]
        visited = set()
        while pending:
            ref = pending.pop()
            reference_key = key(ref)
            if reference_key is None or reference_key[0] in visited:
                continue
            visited.add(reference_key[0])
            producer = steps.get(reference_key[0])
            if producer:
                for dependency in used_refs(producer):
                    if dependency == prerequisite:
                        return True
                    pending.append(dependency)
        return False

    exp_validation = {'runtime-supplement', 'tcc-positive', 'tcc-negative', 'tcc-execution'}
    if binutils:
        exp_validation.update(('as-positive', 'ld-positive', 'as-negative', 'positive-execution'))
    validation = index['validation_commands']
    if set(validation) != exp_validation or len(set(validation.values())) != len(validation):
        issue('stagex_validation_command_inventory', {'expected': sorted(exp_validation), 'actual': validation})
    for launch, id_ in validation.items():
        action = 'stagex-provider-publication/' + id_
        step = steps.get(action)
        if not step or step['operation']['kind'] != 'exec':
            issue('stagex_validation_command_not_exec', {'launch': launch, 'action': action})
            continue
        op = step['operation']
        code = 42 if launch in ('tcc-execution', 'positive-execution') else (1 if launch in ('tcc-negative', 'as-negative') else 0)
        policy = op.get('exit_policy', {})
        if policy.get('kind') != 'exact' or (policy.get('code') == 0 if code == 1 else policy.get('code') != code):
            issue('stagex_validation_exit_status', {'launch': launch, 'action': action, 'policy': policy})
        image = step['inputs'].get(op.get('executable_input'))
        if launch in ('tcc-execution', 'positive-execution'):
            producer = validation['tcc-positive' if launch == 'tcc-execution' else 'ld-positive']
            if key(image) is None or key(image)[0] != 'stagex-provider-publication/' + producer:
                issue('stagex_execution_does_not_use_positive_build', {'launch': launch, 'image': image})
        elif launch not in ('runtime-supplement',):
            role = 'as' if launch.startswith('as-') else 'ld' if launch == 'ld-positive' else 'tcc'
            if image != publication['exports'].get(role):
                issue('stagex_validation_not_exported_tool', {'launch': launch, 'role': role, 'image': image})
    export_kinds = {}
    for role in expected_tools + ['musl.headers', 'musl.libraries']:
        ref = publication['exports'].get(role)
        product = steps.get(key(ref)[0]) if key(ref) else None
        kind = product['outputs'].get(key(ref)[1], {}).get('kind') if product else None
        export_kinds[role] = kind
        if kind != ('tree' if role.startswith('musl.') else 'executable'):
            issue('stagex_export_kind', {'role': role, 'reference': ref, 'kind': kind})
    reach_transition = depends_on(canonical, transition_output)
    reach_tools = {role: depends_on(canonical, publication['exports'][role]) for role in expected_tools if role in publication['exports']}
    if not reach_transition or not all(reach_tools.values()):
        issue('stagex_canonical_missing_prerequisite', {'transition': reach_transition, 'tool_dependencies': reach_tools})
    report['stagex_publication'] = {'binutils_admitted': binutils, 'validation_commands': validation,
                                   'tool_export_kinds': export_kinds, 'canonical_depends_on_transition': reach_transition,
                                   'canonical_depends_on_tool_exports': reach_tools, 'expansion_index_path': str(index_path)}
    del transition, publication, steps

    native = read('full-source-native-provider', baseline)
    provider = native['exports'].get('native.provider')
    canonical_roles = ['native.provider', 'native.sysroot', 'native.runtime']
    native_actions = {'full-source-native-provider/' + step['id']: step for step in native['steps']}
    provider_kind = native_actions.get(key(provider)[0], {}).get('outputs', {}).get(key(provider)[1], {}).get('kind') if key(provider) else None
    native_ok = (key(provider) and key(provider)[1] == 'full-source-native-provider' and
                 provider in native['roots'] and provider_kind == 'tree' and
                 all(native['exports'].get(role) == provider for role in canonical_roles))
    report['native_provider'] = {'canonical_roles': canonical_roles, 'canonical_tree': provider,
                                 'producer_kind': provider_kind, 'valid': bool(native_ok)}
    if not native_ok:
        issue('native_provider_not_canonical', report['native_provider'])
    del native, native_actions

    for stage in ('mantle-stage1', 'mantle-stage2'):
        recipe = read(stage, baseline)
        steps = {stage + '/' + step['id']: step for step in recipe['steps']}
        counts = collections.Counter()
        examples = {'guard_conditions': [], 'optional_direct_inputs': [], 'observations': [], 'perlasm_authenticated': []}
        for action, step in steps.items():
            op = step['operation']
            guard = op.get('guard')
            if guard is not None:
                counts['guarded_commands'] += 1
                if len(examples['guard_conditions']) < 4:
                    examples['guard_conditions'].append({'action': action, 'conditions': guard['conditions']})
            if op['kind'] == 'observe-phase':
                counts['phase_observations'] += 1
                if len(examples['observations']) < 4:
                    examples['observations'].append({'action': action, 'phase': stage + '/' + op['phase'], 'state': op.get('state')})
            if step['id'].endswith('.authenticate') and len(examples['perlasm_authenticated']) < 3:
                consumers = [{'action': target, 'reference': input_ref} for target, other in steps.items()
                             for alias, input_ref in other['inputs'].items() if alias == 'original-script' and
                             key(input_ref) == (action, 'authenticated')]
                examples['perlasm_authenticated'].append({'action': action, 'consumers': consumers[:2]})
            native_script = 'script.native' in step['id']
            if not native_script:
                continue
            for field, expect_kind in [('cwd_input', None), ('invocation_input', 'text')]:
                alias = op.get(field)
                if alias:
                    bound = step['inputs'].get(alias)
                    if not bound:
                        issue('native_script_unbound_operation_input', {'action': action, 'field': field, 'alias': alias})
                    elif key(bound) is not None:
                        producer = steps.get(key(bound)[0])
                        if producer is None or producer['outputs'].get(key(bound)[1], {}).get('kind') != (expect_kind or producer['outputs'].get(key(bound)[1], {}).get('kind')):
                            issue('native_script_nonlocal_or_wrong_kind_operation_input', {'action': action, 'field': field, 'alias': alias, 'reference': bound})
                    counts[field + '_bindings'] += 1
            prior = (op.get('command_state') or {}).get('prior_times_input')
            if prior:
                bound = step['inputs'].get(prior)
                if key(bound) is None or key(bound)[0] not in steps or steps[key(bound)[0]]['outputs'].get(key(bound)[1], {}).get('kind') != 'text':
                    issue('native_script_history_not_owner_text', {'action': action, 'alias': prior, 'reference': bound})
                counts['history_text_bindings'] += 1
            for env, word in op.get('environment', {}).items():
                for part in word.get('parts', []):
                    name = part.get('name') if part.get('kind') in ('input', 'text_input', 'text_slice') else None
                    if name in ('native-environment-data', 'native-request-invocation') or (name or '').startswith('gcc-phase-environment-'):
                        bound = step['inputs'].get(name)
                        if key(bound) is None or key(bound)[0] not in steps or steps[key(bound)[0]]['outputs'].get(key(bound)[1], {}).get('kind') != 'text':
                            issue('native_script_environment_not_owner_text', {'action': action, 'env': env, 'name': name, 'reference': bound})
                        counts['environment_text_parts'] += 1
            for alias, reference in step['inputs'].items():
                target = steps.get(key(reference)[0]) if key(reference) else None
                if alias in ('script', 'original-script') or alias.startswith('gcc-phase-environment-'):
                    if target and target['outputs'].get(key(reference)[1], {}).get('kind') != 'text':
                        issue('native_script_text_not_text', {'action': action, 'alias': alias, 'reference': reference})
                    counts['native_script_text_bindings'] += 1
        report['per_recipe'][stage] = {'counts': dict(counts), 'samples': examples}
        print(json.dumps({'stage': stage, 'supplement_counts': dict(counts), 'issues': {k: v['count'] for k, v in problems.items()}}, sort_keys=True), flush=True)
        del recipe, steps
    stage = 'stagex-transition'
    source = read(stage, baseline)
    ordinary = []
    by_action = {stage + '/' + s['id']: s for s in source['steps']}
    for ref in source['roots']:
        if key(ref) is None:
            continue
        step = by_action[key(ref)[0]]
        policy = step['operation'].get('exit_policy', {})
        if key(ref)[1] not in ('stdout', 'stderr', 'exit-status') and policy.get('kind') in ('one-of', 'normal') and 'exit-status' in step['outputs']:
            ordinary.append(ref)
    report['per_recipe'][stage] = {'optional_failure_ordinary_roots_allowed': len(ordinary),
                                   'optional_failure_ordinary_root_samples': ordinary[:6]}
    if len(ordinary) != baseline['per_recipe'][stage]['optional_failure_roots_exports_semantic']:
        issue('optional_failure_root_count_mismatch', {'stage': stage, 'roots': len(ordinary)})
    del source, by_action
    report['supplement_gate'] = 'accept' if not problems else 'reject'
    report['boundary'] = ('Static publication/export/guard/native owner checks, not a re-execution of product recipes, '
                          'source expansion lineage, generated invocation requests or actual provider validation.')
    return report


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: graph-current-supplement.py graph-current-result.json graph-current-supplement.json')
    baseline = json.loads(pathlib.Path(sys.argv[1]).read_text())
    if baseline['advisory_static_launch_gate'] != 'accept':
        raise SystemExit('main graph did not accept')
    report = inspect(baseline)
    target = pathlib.Path(sys.argv[2])
    target.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'supplement_gate': report['supplement_gate'], 'issues': report['issues'], 'result': str(target)}, sort_keys=True))
