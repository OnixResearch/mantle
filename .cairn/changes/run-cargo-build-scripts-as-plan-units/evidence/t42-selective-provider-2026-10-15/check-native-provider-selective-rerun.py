#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Compare admitted cfgfix 0.1 with a provider-only 0.2 rerun."""
import json
import re
import sys
from pathlib import Path


def admitted_logical_output(report, output):
    physical = Path(output['path'])
    prefix = report.get('store_dir')
    if not isinstance(prefix, str) or not prefix.startswith('/') or report.get('output_dir') != str(root / 'client-store') or physical.parent != root / 'client-store':
        raise SystemExit('report logical prefix and physical client output binding mismatch')
    logical = str(Path(prefix) / physical.name)
    attested = output.get('artifact_attestation', {}).get('logical_path')
    if attested != logical:
        raise SystemExit('artifact attestation logical path differs from admitted prefix/output basename')
    return logical

def goal_outcome(report_data, role, goal_name):
    prefix = report_data.get('store_dir')
    if not isinstance(prefix, str) or not prefix.startswith('/'):
        raise SystemExit('canonical goal identity lacks admitted logical store prefix')
    pattern = rf'{re.escape(prefix.rstrip("/"))}/[0-9a-df-np-sv-z]{{32}}-{re.escape(goal_name)}\.drv'
    rows = [row for row in report_data['outcomes'] if row['label'] == role
            and isinstance(row.get('drv_key'), str) and re.fullmatch(pattern, row['drv_key'])]
    if len(rows) != 1:
        raise SystemExit(f'exact role and canonical goal identity required: {role} {goal_name}')
    return rows[0]


root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
store = root / 'client-store'
old_label = 'cargo-build-script-units-native-cfgfix-87lq'
new_label = 'cargo-build-script-units-native-variant-87lq'
if sys.argv[1:] not in ([], ['cfgfix-native-app-report.json']):
    raise SystemExit('baseline must be the accepted cfgfix same-client Rust app report')


def receipt(name, logical):
    text = (root / name).read_text()
    basename = Path(logical).name
    if not re.search(rf'^OK {re.escape(basename)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*$', text, re.M) or '1 checked, 0 mismatches' not in text:
        raise SystemExit(f'trusted physical store verification prerequisite missing: {name} {logical}')


def physical(logical, suffix):
    if not re.fullmatch(r'/nix/store/[0-9a-df-np-sv-z]{32}-[^/]+', logical):
        raise SystemExit(f'malformed signed provider root: {logical}')
    path = store / Path(logical).name
    if path.is_symlink() or not (path / suffix).is_file():
        raise SystemExit(f'native archive missing physically: {path / suffix}')


def report(name, header, completion=None):
    trace = (root / name).read_text()
    first, separator, payload = trace.partition('\n')
    if not separator or not first.startswith(header) or 'no-substitute=true' not in first or 'timeout: sending signal TERM' in trace:
        raise SystemExit(f'missing completed no-substitute sandbox report: {name}')
    if completion and completion not in trace:
        raise SystemExit(f'baseline run not complete: {name}')
    if name == 'cfgfix-native-app-trace.log':
        return json.loads((root / 'cfgfix-native-app-report.json').read_text())
    return json.loads(payload)



def output(row, name, file_output):
    paths = [Path(item['path']) for item in row['outputs'] if item['name'] == name]
    if len(paths) != 1 or paths[0].parent != store or paths[0].is_symlink() or not (paths[0].is_file() if file_output else paths[0].is_dir()):
        raise SystemExit(f'one physical same-client output required for {row["label"]} {name}: {paths}')
    return paths[0]


def plan_for(report_data, role, goal_name, verify_log):
    plan_path = output(goal_outcome(report_data, role, goal_name), 'plan', True)
    receipt(verify_log, plan_path.name)
    producer_row = goal_outcome(report_data, role, goal_name)
    plan_output = next(item for item in producer_row['outputs'] if item['name'] == 'plan')
    logical_plan = admitted_logical_output(report_data, plan_output)
    admitted = [row for row in report_data['native_dynamic_plans'] if row.get('plan_artifact_path') == logical_plan]
    if len(admitted) != 1 or admitted[0]['rejection_reason'] is not None or admitted[0]['scheduler_action'] != 'registered-roots' or len(admitted[0]['accepted_unit_ids']) != 34:
        raise SystemExit(f'producer plan was not admitted: {goal_name}')
    plan = json.loads(plan_path.read_text())
    if plan['schema'] != 'mantle-plan-v2' or len(plan['units']) != 34 or len(plan['sources']) != 14 or len(plan['roots']) != 1:
        raise SystemExit(f'wrong fixture graph for {goal_name}')
    units = {unit['id']: unit for unit in plan['units']}
    if len(units) != 34 or plan['roots'][0] not in units:
        raise SystemExit(f'duplicate or missing plan identities for {goal_name}')
    accepted_ids = admitted[0]['accepted_unit_ids']
    if len(accepted_ids) != len(set(accepted_ids)) or set(accepted_ids) != set(units):
        raise SystemExit(f'accepted unit IDs do not exactly match physical plan IDs: {goal_name}')
    slices = admitted[0].get('source_slices', [])
    if len(slices) != 14 or {row['source_id'] for row in slices} != {source['id'] for source in plan['sources']}:
        raise SystemExit('source admission rows do not exactly match all 14 plan slices')
    for row in slices:
        if row.get('disposition') != 'admitted' or not row.get('admitted_store_path') or not re.fullmatch(r'[0-9a-f]{64}', row.get('declared_nar_blake3', '')) or row.get('observed_nar_blake3') != row['declared_nar_blake3']:
            raise SystemExit('source slice lacks a matching admitted digest')
    return plan_path, units, plan['roots'][0]




def unit_execution_events(units, trace_text, producer_drv_key):
    names = {unit['derivation']['name'] + '.drv': identity for identity, unit in units.items()}
    if len(names) != len(units):
        raise SystemExit('full unit IDs do not map to unique trace derivation names')
    producer_basename = Path(producer_drv_key).name
    if not re.fullmatch(r'[0-9a-df-np-sv-z]{32}-.+\.drv', producer_basename):
        raise SystemExit('producer trace binding lacks canonical logical derivation key')
    producer_name = producer_basename[33:]
    if producer_name in names:
        raise SystemExit('producer trace name collides with an admitted unit')
    events = {}
    for number, raw in enumerate(trace_text.splitlines(), 1):
        line = re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]', '', raw)
        if 'crunch_build::orchestrate:' not in line:
            continue
        built = 'build succeeded' in line
        cached = ('shared action result admitted, skipping executor' in line
                  or 'all outputs cached, skipping build' in line)
        if not built and not cached:
            continue
        if built and cached:
            raise SystemExit('contradictory execution/cache trace event')
        match = re.search(r'\bdrv=(?:"([^"]+)"|([^\s]+))', line)
        if not match:
            raise SystemExit('execution/cache trace event lacks derivation name')
        name = match.group(1) or match.group(2)
        if name not in names and name != producer_name:
            raise SystemExit(f'unexpected execution/cache trace goal: {name}')
        if name in events:
            raise SystemExit(f'duplicate or contradictory unit trace event: {name}')
        events[name] = {'kind': ('cached-admitted' if 'shared action result admitted, skipping executor' in line else 'cached-local') if cached else 'built-success', 'line': number, 'drv_name': name}
    if set(events) != set(names) | {producer_name}:
        missing = sorted((set(names) | {producer_name}) - set(events))
        raise SystemExit(f'execution/cache trace omits admitted goals: {missing}')
    if events[producer_name]['kind'] != 'built-success':
        raise SystemExit('declared uncached producer did not execute successfully in its trace')
    return {identity: events[name] for name, identity in names.items()}

def assert_independent_reuse(old_units, new_units, old_affected, new_affected, observed_events):
    old_independent = {key: unit for key, unit in old_units.items() if key not in old_affected}
    new_independent = {key: unit for key, unit in new_units.items() if key not in new_affected}
    if set(old_independent) != set(new_independent):
        raise SystemExit('unexpected independent unit identity change')
    if old_independent != new_independent:
        raise SystemExit('unexpected independent unit record change')
    for unit in new_independent.values():
        event = observed_events.get(unit['id'])
        if event is None or event['kind'] != 'cached-admitted':
            raise SystemExit(f'independent unit lacks positive admitted-cache event: {unit["id"]}')
    return len(new_independent)

def crate(units, name):
    matches = []
    for unit in units.values():
        args = unit['derivation']['args']
        if '--crate-name' in args and args[args.index('--crate-name') + 1] == name:
            matches.append(unit)
    return matches


def native_chain(units, app_id, provider):
    scripts = [unit for unit in units.values() if unit['derivation']['args'] == ['run-build-script'] and unit['derivation']['env'].get('MANTLE_UNIT_LINKS_KEY') == 'native_answer']
    sys_crates = crate(units, 'native_answer_sys')
    if len(scripts) != 1 or len(sys_crates) != 1 or len(crate(units, 'mantle_cargo_build_script_fixture')) != 1:
        raise SystemExit('native build script/sys/app must each be unambiguous')
    script, native_sys = scripts[0], sys_crates[0]
    if crate(units, 'mantle_cargo_build_script_fixture')[0]['id'] != app_id:
        raise SystemExit('selected root is not the fixture application')
    native = json.loads(script['derivation']['env']['MANTLE_UNIT_NATIVE'])
    if native.get('inputs') != {'answer': provider} or native.get('env') != {'NATIVE_ANSWER_ROOT': provider}:
        raise SystemExit('native script role does not bind the provider root')
    if {'kind': 'store_path', 'path': provider} not in script['derivation']['inputs']:
        raise SystemExit('provider is not a declared build-script input')
    flag_ref = f'{{{{mantle-unit-output:{script["id"]}:flags}}}}'
    sys_env = native_sys['derivation']['env']
    if sys_env.get('MANTLE_UNIT_FLAGS') != flag_ref:
        raise SystemExit('native-sys compilation is not bound to its script flags')
    extern = json.loads(units[app_id]['derivation']['env']['MANTLE_UNIT_EXTERNS'])
    if extern.get('native_answer_sys') != f'{{{{mantle-unit-output:{native_sys["id"]}:out}}}}':
        raise SystemExit('application is not linked to native-sys compilation')
    return script, native_sys


def successful_unit_outputs(unit, event, trace_text):
    """Bind a nominal plan label to actual successful effective output paths."""
    if unit['derivation'].get('addressing_mode') != 'input-addressed' or unit['derivation'].get('fixed_output') is not None:
        raise SystemExit('effective output inequality requires these input-addressed fixture units')
    if event['kind'] != 'built-success' or event['drv_name'] != unit['derivation']['name'] + '.drv':
        raise SystemExit('affected unit lacks a bound successful execution event')
    line = re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]', '', trace_text.splitlines()[event['line'] - 1])
    match = re.search(r'\boutputs=(\[.*\])\s*$', line)
    if not match:
        raise SystemExit('successful execution event lacks complete effective output paths')
    paths = json.loads(match.group(1))
    expected = unit['derivation']['outputs']
    if len(paths) != len(expected) or len(set(paths)) != len(paths):
        raise SystemExit('successful effective output paths are missing or duplicated')
    result = {}
    for name in expected:
        suffix = unit['derivation']['name'] + ('' if name == 'out' else '-' + name)
        matches = [Path(path) for path in paths if isinstance(path, str) and Path(path).parent == store and re.fullmatch(r'[0-9a-df-np-sv-z]{32}-' + re.escape(suffix), Path(path).name)]
        if len(matches) != 1:
            raise SystemExit(f'effective output path is not bound to unit/output: {unit["id"]}:{name}')
        result[name] = '/nix/store/' + matches[0].name
    return result


def affected_effective_changes(old_chain, new_chain, old_events, new_events, old_trace, new_trace):
    changes = []
    for role, before, after in zip(('native-script', 'native-sys', 'app'), old_chain, new_chain):
        old_outputs = successful_unit_outputs(before, old_events[before['id']], old_trace)
        new_outputs = successful_unit_outputs(after, new_events[after['id']], new_trace)
        if set(old_outputs) != set(new_outputs) or any(old_outputs[name] == new_outputs[name] for name in old_outputs):
            raise SystemExit(f'provider-dependent effective output identity did not change: {role}')
        changes.append({'role': role, 'old_nominal_id': before['id'], 'new_nominal_id': after['id'], 'old_outputs': old_outputs, 'new_outputs': new_outputs})
    return changes


def bind_effective_app_report(data, unit, trace_outputs):
    name = unit['derivation']['name']
    row = goal_outcome(data, name + '.drv', name)
    bound = {item['name']: admitted_logical_output(data, item) for item in row['outputs']}
    if row['cached'] or len(bound) != len(row['outputs']) or bound != trace_outputs:
        raise SystemExit('actual app execution outputs are not bound to its canonical report goal')
    return row['drv_key']


# Signed roots are explicit prerequisites; these log names must contain actual same-client
# trusted physical store-verify receipts before any success can be reported.
base_config = json.loads((root / 'cfgfix-native-config.json').read_text())
changed_config = json.loads((root / 'variant-native-config.json').read_text())
if len(base_config['native_inputs']) != 1 or set(base_config['native_inputs']) != set(changed_config['native_inputs']):
    raise SystemExit('cfgfix and variant native roles are not identical')
old_entry = next(iter(base_config['native_inputs'].values()))
new_entry = next(iter(changed_config['native_inputs'].values()))
old_root, new_root = old_entry['inputs']['answer'], new_entry['inputs']['answer']
if old_entry['env'] != {'NATIVE_ANSWER_ROOT': '{{role:answer}}'} or new_entry['env'] != old_entry['env']:
    raise SystemExit('provider role marker changed')
old_entry['inputs']['answer'] = new_root
if base_config != changed_config or old_root == new_root:
    raise SystemExit('cfgfix config changed beyond the sole native provider root')
for logical, version, log in ((old_root, '0.1', 'current-native-c-provider-verify.log'), (new_root, '0.2', 'variant-native-provider-verify.log')):
    if not re.fullmatch(rf'/nix/store/[0-9a-df-np-sv-z]{{32}}-native-answer-{re.escape(version)}', logical):
        raise SystemExit(f'wrong Mantle-built provider root: {logical}')
    physical(logical, 'lib/libnative_answer.a')
    receipt(log, logical)
old_recipe = (root / 'cfgfix-native-plan.ncl').read_text()
new_recipe = (root / 'variant-native-plan.ncl').read_text()
old_configs = re.findall(r'(?m)^  config = "([^"]+)",$', old_recipe)
new_configs = re.findall(r'(?m)^  config = "([^"]+)",$', new_recipe)
if len(old_configs) != 1 or len(new_configs) != 1 or old_configs[0] == new_configs[0] or not new_configs[0].endswith('-mantle-unit-plan-config-variant-87lq'):
    raise SystemExit('distinct signed variant producer config reference required')
receipt('cfgfix-native-config-verify.log', old_configs[0])
receipt('variant-native-config-verify.log', new_configs[0])
if (old_recipe.count(old_root), old_recipe.count(old_label), old_recipe.count(old_configs[0])) != (1, 1, 1) or new_recipe != old_recipe.replace(old_root, new_root).replace(old_label, new_label).replace(old_configs[0], new_configs[0]):
    raise SystemExit('variant producer recipe changed beyond provider/config root and goal label')
base = report('cfgfix-native-app-trace.log', 'phase=cfgfix-app-start ', 'phase=cfgfix-app-done ')
changed = report('variant-native-app.log', f'variant-app-native-root-source={root}/variant-native-plan.ncl ')
consumer = report('variant-native-app-consumer.log', 'sandbox-shell=')
for label, data in (('cfgfix', base), ('variant', changed), ('consumer', consumer)):
    if data['counts']['failed_total'] or data['counts']['succeeded_total'] < (2 if label == 'cfgfix' else 1):
        raise SystemExit(f'unsuccessful {label} client report')
old_plan, old_units, old_app = plan_for(base, 'producer', old_label, 'cfgfix-native-producer-verify.log')
new_plan, new_units, new_app = plan_for(changed, new_label, new_label, 'variant-native-producer-verify.log')
if (root / 'cfgfix-native-app-unit-id.txt').read_text().strip() != old_app or (root / 'variant-native-app-unit-id.txt').read_text().strip() != new_app:
    raise SystemExit('selected cfgfix/variant application identities not bound to producer plans')
old_script, old_sys = native_chain(old_units, old_app, old_root)
new_script, new_sys = native_chain(new_units, new_app, new_root)
original = json.loads((root / 'cfgfix-native-producer-report.json').read_text())
original_plan, original_units, original_app = plan_for(original, old_label, old_label, 'cfgfix-native-producer-verify.log')
if original['counts']['failed_total'] or original_plan != old_plan or original_units != old_units or original_app != old_app:
    raise SystemExit('original full-graph trace is not bound to the accepted cfgfix app plan')
original_trace = (root / 'cfgfix-native-producer-trace.log').read_text()
if 'phase=cfgfix-producer-done ' not in original_trace or 'timeout: sending signal TERM' in original_trace:
    raise SystemExit('completed original producer execution trace required')
original_events = unit_execution_events(old_units, original_trace, goal_outcome(original, old_label, old_label)['drv_key'])
if any(event['kind'] != 'built-success' for event in original_events.values()):
    raise SystemExit('original full graph did not execute every admitted unit')
variant_trace = (root / 'variant-native-producer-trace.log').read_text()
if not variant_trace.startswith('phase=variant-producer-start ') or 'phase=variant-producer-done ' not in variant_trace or 'no-substitute=true' not in variant_trace.splitlines()[0] or 'timeout: sending signal TERM' in variant_trace:
    raise SystemExit('completed variant no-substitute execution/cache trace required')
variant_events = unit_execution_events(new_units, variant_trace, goal_outcome(changed, new_label, new_label)['drv_key'])
for unit in (new_script, new_sys, new_units[new_app]):
    if variant_events[unit['id']]['kind'] != 'built-success':
        raise SystemExit(f'provider-dependent unit lacks actual successful build event: {unit["id"]}')
independent_count = assert_independent_reuse(old_units, new_units,
    (old_script['id'], old_sys['id'], old_app),
    (new_script['id'], new_sys['id'], new_app), variant_events)
if independent_count != 31 or sum(event['kind'] == 'built-success' for event in variant_events.values()) != 3 or sum(event['kind'] == 'cached-admitted' for event in variant_events.values()) != 31:
    raise SystemExit('strict selective outcome requires exactly 3 affected builds and 31 admitted-cache events')
effective_changes = affected_effective_changes(
    (old_script, old_sys, old_units[old_app]), (new_script, new_sys, new_units[new_app]),
    original_events, variant_events, original_trace, variant_trace)
old_app_drv = bind_effective_app_report(original, old_units[old_app], effective_changes[2]['old_outputs'])
new_app_drv = bind_effective_app_report(changed, new_units[new_app], effective_changes[2]['new_outputs'])
if old_app_drv == new_app_drv:
    raise SystemExit('provider-dependent canonical application derivation identity did not change')
print('effective-dependent-identities=' + json.dumps({'changes': effective_changes, 'old_app_drv': old_app_drv, 'new_app_drv': new_app_drv}, sort_keys=True), flush=True)
for name in ('proc_macro2', 'serde_derive', 'libc', 'serde'):
    before, after = crate(old_units, name), crate(new_units, name)
    if not before or {unit['id'] for unit in before} != {unit['id'] for unit in after}:
        raise SystemExit(f'independent {name} identities changed or disappeared')
base_result = output(goal_outcome(base, 'app', 'cargo-build-script-units-app-check-cfgfix-87lq'), 'out', False)
variant_result_row = goal_outcome(consumer, 'app', 'cargo-build-script-units-app-check-variant-87lq')
variant_result = output(variant_result_row, 'out', False)
if goal_outcome(base, 'app', 'cargo-build-script-units-app-check-cfgfix-87lq')['cached'] or variant_result_row['cached']:
    raise SystemExit('each actual app checker must execute')
expected = 'cargo-build-script-fixture: proc-macro2 serde-derive libc native-sys = 42\n'
for path, log in ((base_result, 'cfgfix-native-app-verify.log'), (variant_result, 'variant-native-app-verify.log')):
    receipt(log, path.name)
    result = path / 'result.txt'
    if result.is_symlink() or result.read_text() != expected:
        raise SystemExit(f'actual Rust app result differs from 42: {result}')
print(f'selective-native-provider-rerun=success baseline=cfgfix old={old_root} new={new_root} native_script={old_script["id"]}->{new_script["id"]} native_sys={old_sys["id"]}->{new_sys["id"]} app={old_app}->{new_app} independent_unit_count={independent_count} all_independent_records_unchanged=true all_independent_positive_cache_events=true affected_successful_build_events=3 named_presence=proc_macro2,serde_derive,libc,serde verified_plan_roots={old_plan.name},{new_plan.name} result={variant_result / "result.txt"}', flush=True)
