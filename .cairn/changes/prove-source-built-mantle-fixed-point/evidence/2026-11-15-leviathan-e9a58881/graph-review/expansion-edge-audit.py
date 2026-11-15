#!/usr/bin/env python3
"""Read-only audit of current frozen expansion indexes and typed graph edges."""
import collections
import hashlib
import json
import re
import subprocess
from pathlib import Path

COHORT = Path('/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index')
SOURCE = Path('/home/brittonr/sbfp-20261113-berdKH/runner-source-e9a58881-typed-index')
B3 = '/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
SCRATCH = Path('/tmp/luna-e9a58881-expansion-edges-20261115')

FILES = {
    'cold_route': COHORT / 'inputs/mantle-source/bootstrap/source-built-cold-route.json',
    'lineage': COHORT / 'inputs/stagex-lineage/stagex-transition-lineage.json',
    'stagex_recipe': COHORT / 'program-sources/source-built-stagex-programs/stagex-transition.json',
    'stagex_index': COHORT / 'program-sources/source-built-stagex-programs/stagex-transition.json.expansion.json',
    'publication_recipe': COHORT / 'program-sources/source-built-stagex-programs/stagex-provider-publication.json',
    'publication_index': COHORT / 'program-sources/source-built-stagex-programs/stagex-provider-publication.json.expansion.json',
    'native_recipe': COHORT / 'program-sources/fixed-point-native-source-program/source-built/native/program.json',
    'native_index': COHORT / 'program-sources/fixed-point-native-source-program/source-built/native/program.expansion.json',
    'rust_recipe': COHORT / 'inputs/rust-source-archives/source-built/rust/program.json',
    'rust_index': COHORT / 'inputs/rust-source-archives/source-built/rust/program.json.expansion.json',
    'rust_route': COHORT / 'inputs/rust-source-archives/cairn/rust/rust-source-musl-host-plan.ncl',
}

def b3(path):
    return subprocess.check_output([B3, '--no-names', str(path)], text=True).strip()

def ref_key(ref):
    return json.dumps(ref, sort_keys=True, separators=(',', ':'))

def used_refs(step):
    """Port source_built_fixed_point_stagex_actions::used_inputs exactly."""
    op = step['operation']
    kind = op['kind']
    inputs = step['inputs']
    names = set()
    direct_refs = set()
    if kind == 'controller':
        spec = op['specification']
        direct_refs.add(ref_key(spec['controller']))
        direct_refs.update(ref_key(ref) for ref in spec['inputs'].values())
        for slot in spec['slots']:
            direct_refs.update(ref_key(ref) for ref in slot['inputs'].values())
            executable = slot['executable']
            variant = executable['kind']
            payload = executable.get('value')
            if variant == 'fixed':
                direct_refs.add(ref_key(payload))
            elif variant == 'adapted':
                direct_refs.add(ref_key(payload['executable']))
                direct_refs.add(ref_key(payload['original_image']))
            elif variant == 'inventory':
                direct_refs.update(ref_key(pair[1]) for pair in payload)
            elif variant == 'request-output':
                pass
            else:
                raise RuntimeError(f'unknown ControllerExecutable variant {variant}')
        guard = op.get('guard')
        if guard:
            direct_refs.update(ref_key(inputs[cond['status_input']]) for cond in guard['conditions'])
        return direct_refs
    if kind == 'exec':
        names.add(op['executable_input'])
        names.update(name for name in (op.get('cwd_input'), op.get('stdin_input'), op.get('invocation_input')) if name)
        names.update(op.get('runtime_mounts', {}).values())
        names.update(op.get('output_seeds', {}).values())
        state = op.get('command_state')
        if state:
            names.update(state.get('prior_times_input', {}).keys() if isinstance(state.get('prior_times_input'), dict) else [])
            if isinstance(state.get('prior_times_input'), str):
                names.add(state['prior_times_input'])
        guard = op.get('guard')
        if guard:
            names.update(cond['status_input'] for cond in guard['conditions'])
        def response_file(word):
            parts = word.get('parts', [])
            return (len(parts) == 2 and parts[0].get('kind') == 'literal' and parts[0].get('text') == '@'
                    and parts[1].get('kind') == 'input')
        dynamic = bool(op.get('invocation_input')) or any(response_file(word) for word in op.get('arguments', []))
        if dynamic:
            names.update(inputs.keys())
        runtime = op.get('runtime_mounts', {})
        for word in list(op.get('arguments', [])) + list(op.get('environment', {}).values()):
            for part in word.get('parts', []):
                p_kind = part.get('kind')
                if p_kind in ('input', 'text_input', 'text_slice'):
                    names.add(part['name'])
                elif p_kind == 'runtime' and part.get('mount') in runtime:
                    names.add(runtime[part['mount']])
    elif kind in ('copy-text', 'extract-archive'):
        names.add(op['input'])
    elif kind == 'assemble-tree':
        names.update(op['entries'].values())
    elif kind == 'materialize-fetcher':
        role, identity = op['role'], op['record_identity']
        for name, ref in inputs.items():
            if (ref.get('kind') == 'manifest-source' and ref.get('role') == role
                    and ref.get('record_identity') == identity and ref.get('member') == ''):
                names.add(name)
            elif ref.get('kind') == 'source' and ref.get('role') == role and ref.get('path') == '':
                names.add(name)
    elif kind == 'observe-phase':
        state = op.get('state')
        if state:
            names.add(state['skipped_input'])
    elif kind in ('write-text', 'materialize-record'):
        pass
    else:
        raise RuntimeError(f'unknown RecipeOperation variant {kind}')
    return {ref_key(inputs[name]) for name in names if name in inputs}

def local_id(stage, reference):
    if reference.get('kind') != 'produced':
        return None
    prefix = stage + '/'
    action_id = reference['action_id']
    return action_id[len(prefix):] if action_id.startswith(prefix) else None

def depends_on(stage, steps, actual, output, target):
    if output == target:
        return True
    first = local_id(stage, output)
    pending = [first] if first in steps else []
    seen = set()
    while pending:
        step_id = pending.pop()
        if step_id in seen:
            continue
        seen.add(step_id)
        step = steps[step_id]
        for key in actual[step_id]:
            ref = json.loads(key)
            if ref == target:
                return True
            producer = local_id(stage, ref)
            if producer in steps:
                pending.append(producer)
        op = step['operation']
        if op['kind'] == 'observe-phase':
            phase_id = op['phase']
            observed = steps.get(phase_id)
            state = op.get('state')
            if state and target.get('kind') == 'produced':
                if (target.get('action_id') == f'{stage}/{phase_id}' and target.get('output_id') == state['output']
                        and observed and state['output'] in observed['outputs']):
                    return True
            if observed:
                pending.append(phase_id)
    return False

def partition(recipe, nodes, stage):
    steps = {step['id']: step for step in recipe['steps']}
    actual = {step_id: used_refs(step) for step_id, step in steps.items()}
    assigned = []
    missing_steps, missing_outputs, unconsumed = [], [], []
    output_count = 0
    for node_id, node in nodes.items():
        step_ids = node['step_ids']
        if not step_ids or not node['outputs']:
            missing_steps.append(f'{node_id}:empty')
        for step_id in step_ids:
            if step_id not in steps:
                missing_steps.append(f'{node_id}:{step_id}:unknown')
            assigned.append(step_id)
        for ref in node['outputs'].values():
            output_count += 1
            producer = local_id(stage, ref)
            if producer is None or producer not in step_ids or ref['output_id'] not in steps.get(producer, {}).get('outputs', {}):
                missing_outputs.append(f'{node_id}:{ref}')
    duplicates = len(assigned) - len(set(assigned))
    unclassified = sorted(set(steps) - set(assigned))
    for step_id, step in steps.items():
        declared = {ref_key(ref) for ref in step['inputs'].values()}
        unused = sorted(declared - actual[step_id])
        if unused:
            unconsumed.append({'step_id': step_id, 'unused_reference_count': len(unused)})
    return steps, actual, {
        'recipe_step_count': len(steps), 'node_count': len(nodes), 'assigned_step_count': len(assigned),
        'duplicate_assignment_count': duplicates, 'unclassified_step_count': len(unclassified),
        'unclassified_step_examples': unclassified[:6], 'output_reference_count': output_count,
        'invalid_node_output_count': len(missing_outputs), 'invalid_node_output_examples': missing_outputs[:6],
        'steps_with_unused_declared_inputs': len(unconsumed), 'unused_input_examples': unconsumed[:6],
    }

def check_stagex():
    lineage = json.loads(FILES['lineage'].read_bytes())
    recipe = json.loads(FILES['stagex_recipe'].read_bytes())
    index = json.loads(FILES['stagex_index'].read_bytes())
    stage = recipe['stage_id']
    steps, actual, partition_report = partition(recipe, index['tools'], stage)
    lineage_tools = {item['id']: item for item in lineage['transition_tools']}
    lineage_sources = {item['id']: item for item in lineage['source_artifacts']}
    lineage_patches = {item['id']: item for item in lineage['patches']}
    sets = {
        'tool_ids_match': set(index['tools']) == set(lineage_tools),
        'source_ids_match': set(index['sources']) == set(lineage_sources),
        'patch_ids_match': set(index['patches']) == set(lineage_patches),
    }
    artifacts = dict(index['sources'])
    artifacts.update(index['patches'])
    artifacts['seed'] = {'kind': 'source', 'role': 'stagex-seed', 'path': ''}
    output_ids = set()
    output_inventories_match = True
    generated_bindings = 0
    for tool_id, tool in lineage_tools.items():
        node = index['tools'].get(tool_id)
        if node is None:
            continue
        expected_outputs = set(tool['output_artifact_ids'])
        output_inventories_match &= set(node['outputs']) == expected_outputs
        for artifact_id, ref in node['outputs'].items():
            output_ids.add(artifact_id)
            if artifact_id in artifacts:
                sets['artifact_ids_unique'] = False
            artifacts[artifact_id] = ref
    sets.setdefault('artifact_ids_unique', True)
    generated_missing = []
    for generated in lineage['generated_artifacts']:
        node = index['tools'].get(generated['producing_tool_id'])
        if node is None or generated['id'] not in node['outputs']:
            generated_missing.append(generated['id'])
        else:
            generated_bindings += 1
    required_edges = 0
    disposed_edges = 0
    missing_actual_edges = []
    for tool_id, tool in lineage_tools.items():
        node = index['tools'].get(tool_id)
        if not node:
            continue
        for raw_id in list(tool['source_artifact_ids']) + list(tool['input_artifact_ids']):
            canonical = 'tinycc-0.9.27-stagex-patch' if raw_id == 'tinycc-0.9.27-patch-source' else raw_id
            ref = artifacts.get(canonical)
            if ref is None:
                missing_actual_edges.append({'tool': tool_id, 'input': canonical, 'failure': 'unmapped'})
                continue
            required_edges += 1
            reason = node.get('input_dispositions', {}).get(canonical, '')
            if reason.strip():
                disposed_edges += 1
            elif ref_key(ref) not in {key for step_id in node['step_ids'] for key in actual.get(step_id, set())}:
                missing_actual_edges.append({'tool': tool_id, 'input': canonical, 'failure': 'not-used'})
    def node_refs(node_id):
        if node_id in artifacts:
            return [artifacts[node_id]]
        node = index['tools'].get(node_id)
        if node:
            return list(node['outputs'].values())
        return None
    edge_checks = 0
    skipped_source_edges = 0
    lineage_edge_failures = []
    for edge in lineage['stage_graph']:
        if edge['to_node_id'] in index['sources'] or edge['to_node_id'] in index['patches']:
            skipped_source_edges += 1
            continue
        incoming = node_refs(edge['from_node_id'])
        outgoing = node_refs(edge['to_node_id'])
        if incoming is None or outgoing is None:
            lineage_edge_failures.append({'edge': edge, 'failure': 'unmapped'})
            continue
        for ref in incoming:
            edge_checks += 1
            if not any(depends_on(stage, steps, actual, product, ref) for product in outgoing):
                lineage_edge_failures.append({'edge': edge, 'reference': ref})
    transition_ref = recipe['exports'].get('stagex-transition')
    provider_checks = 0
    provider_failures = []
    for output in lineage['provider_outputs']:
        ref = artifacts.get(output['producing_artifact_id'])
        provider_checks += 1
        if not ref or not transition_ref or not depends_on(stage, steps, actual, transition_ref, ref):
            provider_failures.append(output['producing_artifact_id'])
    return {
        'lineage_counts': {key: len(lineage[key]) for key in ('source_artifacts', 'patches', 'transition_tools', 'generated_artifacts', 'stage_graph', 'provider_outputs')},
        'index_counts': {key: len(index[key]) for key in ('sources', 'patches', 'tools')},
        'set_checks': sets, 'output_inventories_match': bool(output_inventories_match),
        'generated_artifact_mapped_count': generated_bindings, 'generated_artifact_missing': generated_missing[:8],
        'required_original_input_edge_count': required_edges, 'authored_disposition_edge_count': disposed_edges,
        'missing_actual_original_input_edge_count': len(missing_actual_edges), 'missing_actual_examples': missing_actual_edges[:8],
        'lineage_stage_graph_edge_count': len(lineage['stage_graph']), 'skipped_edges_to_admitted_source_count': skipped_source_edges,
        'actual_dependency_checks': edge_checks, 'lineage_edge_failure_count': len(lineage_edge_failures), 'lineage_edge_failures': lineage_edge_failures[:8],
        'provider_output_checks': provider_checks, 'provider_output_failure_count': len(provider_failures), 'provider_output_failures': provider_failures[:8],
        'partition': partition_report,
    }

def parse_route(text):
    source_region = text.split('  sources = [', 1)[1].split('  stages = [', 1)[0]
    stage_region = text.split('  stages = [', 1)[1].split('  final_outputs = [', 1)[0]
    final_region = text.split('  final_outputs = [', 1)[1].split('\n  ],', 1)[0]
    source_pairs = re.findall(r'\{\s*id\s*=\s*"([^"]+)"\s*,\s*kind\s*=\s*"[^"]+"\s*,.*?sha256_hex\s*=\s*"([0-9a-f]{64})"', source_region, re.S)
    stage_matches = re.findall(
        r'\{\s*id\s*=\s*"([^"]+)"\s*,\s*kind\s*=\s*"([^"]+)"\s*,\s*source_ids\s*=\s*\[([^\]]*)\]\s*,\s*bootstrap_stage_id\s*=\s*"([^"]*)"\s*,\s*rust_version\s*=\s*"([^"]+)"\s*,\s*outputs\s*=\s*\[([^\]]*)\]',
        stage_region, re.S)
    stages = []
    for stage_id, kind, sources, bootstrap, version, outputs in stage_matches:
        stages.append({'id': stage_id, 'kind': kind, 'source_ids': re.findall(r'"([^"]+)"', sources),
                       'bootstrap_stage_id': bootstrap, 'rust_version': version, 'outputs': re.findall(r'"([^"]+)"', outputs)})
    final_outputs = re.findall(r'role\s*=\s*"([^"]+)"', final_region)
    if not source_pairs or not stages or not final_outputs:
        raise RuntimeError('could not statically parse source, stage, and output lists from the frozen Nickel route')
    policy_region = text.split('  policy = {', 1)[1].split('  },', 1)[0]
    policy = {key: bool(value == 'true') for key, value in re.findall(r'([a-z_]+)\s*=\s*(true|false)', policy_region)}
    return source_pairs, stages, final_outputs, policy

def check_rust():
    recipe = json.loads(FILES['rust_recipe'].read_bytes())
    index = json.loads(FILES['rust_index'].read_bytes())
    text = FILES['rust_route'].read_text()
    sources, stages, final_outputs, policy = parse_route(text)
    stage = recipe['stage_id']
    steps, actual, partition_report = partition(recipe, index['stages'], stage)
    source_ids = {item[0] for item in sources}
    route_stages = {item['id']: item for item in stages}
    archive_keys_match = set(index['archives']) == source_ids
    stage_keys_match = set(index['stages']) == set(route_stages)
    archive_checks = []
    for source_id, expected_sha in sources:
        ref = index['archives'].get(source_id, {})
        source_path = COHORT / 'inputs/rust-source-archives' / ref.get('path', '')
        valid_ref = ref.get('kind') == 'source' and ref.get('role') == 'rust-source-archive-set'
        observed_sha = hashlib.sha256(source_path.read_bytes()).hexdigest() if source_path.is_file() else None
        archive_checks.append({'id': source_id, 'path': ref.get('path'), 'ref_valid': valid_ref,
                               'route_sha256': expected_sha, 'actual_sha256': observed_sha,
                               'sha256_matches': observed_sha == expected_sha})
    source_edge_checks = 0
    missing_source_edges = []
    bootstrap_edge_checks = 0
    missing_bootstrap_edges = []
    bootstrap_compiler_checks = 0
    missing_compiler_execs = []
    node_output_checks = 0
    node_output_failures = []
    for stage_id, route_stage in route_stages.items():
        node = index['stages'].get(stage_id)
        if not node:
            continue
        expected_outputs = set(route_stage['outputs'])
        if set(node['outputs']) != expected_outputs:
            node_output_failures.append({'stage': stage_id, 'expected': sorted(expected_outputs), 'actual': sorted(node['outputs'])})
        for source_id in route_stage['source_ids']:
            source_edge_checks += 1
            ref = index['archives'].get(source_id)
            if not ref or ref_key(ref) not in {key for step_id in node['step_ids'] for key in actual.get(step_id, set())}:
                missing_source_edges.append({'stage': stage_id, 'source': source_id})
        prior_id = route_stage['bootstrap_stage_id']
        if prior_id:
            prior = index['stages'].get(prior_id)
            if prior:
                compiler_ref = prior['outputs'].get('rustc')
                for output_ref in prior['outputs'].values():
                    bootstrap_edge_checks += 1
                    if ref_key(output_ref) not in {key for step_id in node['step_ids'] for key in actual.get(step_id, set())}:
                        missing_bootstrap_edges.append({'stage': stage_id, 'predecessor': prior_id, 'output': output_ref})
                bootstrap_compiler_checks += 1
                found = any(steps[step_id]['operation']['kind'] == 'exec'
                            and steps[step_id]['inputs'].get(steps[step_id]['operation']['executable_input']) == compiler_ref
                            for step_id in node['step_ids'] if step_id in steps)
                if not found:
                    missing_compiler_execs.append({'stage': stage_id, 'predecessor': prior_id})
        for role, ref in node['outputs'].items():
            node_output_checks += 1
            producer = local_id(stage, ref)
            if producer not in node['step_ids'] or ref.get('output_id') not in steps.get(producer, {}).get('outputs', {}):
                node_output_failures.append({'stage': stage_id, 'role': role, 'reference': ref})
    cargo_ref = recipe['exports'].get('cargo')
    cargo_execs = []
    for step_id, step in steps.items():
        op = step['operation']
        if op['kind'] == 'exec' and cargo_ref and step['inputs'].get(op['executable_input']) == cargo_ref:
            cargo_execs.append(step_id)
    native_recipe = json.loads(FILES['native_recipe'].read_bytes())
    native_exports = native_recipe['exports']
    native_roles = [
        'native.cc1','native.cc1plus','native.as','native.ld','native.ar','native.nm','native.ranlib',
        'native.objcopy','native.strip','native.readelf','native.loader','host.make','host.cmake','host.python',
        'host.perl','host.busybox','native.provider','native.sysroot','linux.headers','native.libc','native.libgcc',
        'native.libgcc-eh','native.libstdcxx']
    native_role_checks = {role: any(ref_key(native_exports[role]) in refs for refs in actual.values())
                          for role in native_roles if role in native_exports}
    last_stage = stages[-1]
    final_stage_outputs = index['stages'][last_stage['id']]['outputs']
    final_export_matches = {role: recipe['exports'].get(role) == final_stage_outputs.get(role) for role in final_outputs}
    return {
        'route_source_path': str(FILES['rust_route']), 'route_source_sha256': hashlib.sha256(FILES['rust_route'].read_bytes()).hexdigest(),
        'route_source_blake3': b3(FILES['rust_route']), 'route_was_evaluated': False,
        'route_policy': policy, 'route_sources': [item[0] for item in sources],
        'route_stages': stages, 'route_final_outputs': final_outputs,
        'index_schema': index['schema'], 'index_recipe_path': index['recipe_path'], 'index_route_path': index['route_path'],
        'index_archive_count': len(index['archives']), 'index_stage_count': len(index['stages']),
        'archive_key_set_matches_route': archive_keys_match, 'stage_key_set_matches_route': stage_keys_match,
        'archive_checks': archive_checks, 'archive_identity_failures': sum(not item['sha256_matches'] or not item['ref_valid'] for item in archive_checks),
        'source_edge_checks': source_edge_checks, 'missing_source_edge_count': len(missing_source_edges), 'missing_source_edges': missing_source_edges[:6],
        'bootstrap_output_checks': bootstrap_edge_checks, 'missing_bootstrap_output_count': len(missing_bootstrap_edges), 'missing_bootstrap_outputs': missing_bootstrap_edges[:6],
        'bootstrap_compiler_exec_checks': bootstrap_compiler_checks, 'missing_bootstrap_compiler_execs': missing_compiler_execs,
        'node_output_checks': node_output_checks, 'node_output_failure_count': len(node_output_failures), 'node_output_failures': node_output_failures[:6],
        'cargo_executable_reference': cargo_ref, 'cargo_exec_count': len(cargo_execs), 'cargo_exec_examples': cargo_execs[:6],
        'native_role_usage': native_role_checks, 'native_role_missing': sorted(role for role, used in native_role_checks.items() if not used),
        'final_export_matches': final_export_matches,
        'provider_export': recipe['exports'].get(stage), 'toolchain_closure_export': recipe['exports'].get('toolchain-closure'),
        'partition': partition_report,
    }

def check_native():
    recipe = json.loads(FILES['native_recipe'].read_bytes())
    index = json.loads(FILES['native_index'].read_bytes())
    stage = recipe['stage_id']
    nodes = index['derivations']
    steps, actual, partition_report = partition(recipe, nodes, stage)
    path_to_outputs = {path: set(node['outputs']) for path, node in nodes.items()}
    derivation_edges = 0
    declared_edges = 0
    used_edges = 0
    missing_declared = []
    missing_actual = []
    source_paths_total = 0
    source_nodes = 0
    source_nodes_with_typed_source_use = 0
    source_examples = []
    output_checks = 0
    invalid_output_refs = []
    derivation_source_ref_counts = collections.Counter()
    for drv, node in nodes.items():
        ids = node['step_ids']
        node_actual = {key for step_id in ids for key in actual.get(step_id, set())}
        node_declared = {ref_key(ref) for step_id in ids if step_id in steps for ref in steps[step_id]['inputs'].values()}
        source_paths = node.get('input_sources', [])
        source_paths_total += len(source_paths)
        if source_paths:
            source_nodes += 1
            source_refs = [json.loads(key) for key in node_actual if json.loads(key).get('kind') in ('source', 'manifest-source')]
            if source_refs:
                source_nodes_with_typed_source_use += 1
            if len(source_examples) < 12:
                source_examples.append({'derivation': drv, 'input_source_count': len(source_paths),
                                        'typed_source_ref_count': len(source_refs), 'typed_source_refs': source_refs[:4]})
        for output_id, ref in node['outputs'].items():
            output_checks += 1
            producer = local_id(stage, ref)
            if producer not in ids or ref.get('output_id') not in steps.get(producer, {}).get('outputs', {}):
                invalid_output_refs.append({'derivation': drv, 'output': output_id, 'reference': ref})
        for dependency, outputs in node['input_derivations'].items():
            dependency_node = nodes.get(dependency)
            for output in outputs:
                derivation_edges += 1
                if dependency_node is None or output not in dependency_node['outputs']:
                    missing_declared.append({'derivation': drv, 'dependency': dependency, 'output': output, 'failure': 'unknown-index-output'})
                    continue
                ref = dependency_node['outputs'][output]
                key = ref_key(ref)
                if key in node_declared:
                    declared_edges += 1
                else:
                    missing_declared.append({'derivation': drv, 'dependency': dependency, 'output': output, 'failure': 'not-declared'})
                if key in node_actual:
                    used_edges += 1
                else:
                    missing_actual.append({'derivation': drv, 'dependency': dependency, 'output': output, 'failure': 'not-used'})
    roots = index['roots']
    return {
        'index_schema': index['schema'], 'recipe_member': index['recipe_member'],
        'root_count': len(roots), 'roots': roots, 'derivation_count': len(nodes),
        'partition': partition_report, 'derivation_output_reference_checks': output_checks,
        'invalid_derivation_output_count': len(invalid_output_refs), 'invalid_derivation_output_examples': invalid_output_refs[:6],
        'input_derivation_output_edge_count': derivation_edges,
        'edge_refs_attached_to_some_declared_step_input': declared_edges,
        'edge_refs_used_as_actual_operation_inputs': used_edges,
        'missing_declared_edge_count': len(missing_declared), 'missing_declared_examples': missing_declared[:6],
        'missing_actual_edge_count': len(missing_actual), 'missing_actual_examples': missing_actual[:12],
        'input_source_path_count': source_paths_total, 'derivations_with_input_sources': source_nodes,
        'source_nodes_with_any_typed_Source_or_ManifestSource_actual_input': source_nodes_with_typed_source_use,
        'source_examples': source_examples,
    }

def main():
    hashes = {key: {'path': str(path), 'bytes': path.stat().st_size, 'blake3': b3(path),
                    'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
              for key, path in FILES.items()}
    route = json.loads(FILES['cold_route'].read_bytes())
    publication_index = json.loads(FILES['publication_index'].read_bytes())
    publication_recipe = json.loads(FILES['publication_recipe'].read_bytes())
    result = {
        'scope': 'Read-only current source/index/route/lineage data audit; static NCL text only, no NCL evaluation and no product/helper/recipe execution.',
        'cold_route': route,
        'input_file_identities': hashes,
        'stagex_lineage_and_index': check_stagex(),
        'rust_route_and_expansion': check_rust(),
        'native_expansion': check_native(),
        'publication_index': {
            'schema': publication_index['schema'], 'recipe_member': publication_index['recipe_member'],
            'binutils_admitted': publication_index['binutils_admitted'],
            'validation_command_count': len(publication_index['validation_commands']),
            'validation_commands': publication_index['validation_commands'],
            'recipe_step_count': len(publication_recipe['steps']),
            'distinct_command_ids': len(set(publication_index['validation_commands'].values())),
            'all_command_steps_present': all(any(step['id'] == value for step in publication_recipe['steps'])
                                             for value in publication_index['validation_commands'].values()),
        },
    }
    out = SCRATCH / 'expansion-edge-audit.json'
    out.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps(result, indent=2, sort_keys=True))

if __name__ == '__main__':
    main()
