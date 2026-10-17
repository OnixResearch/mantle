#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
import json
import re
import subprocess
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
trace = (root / 'cfgfix-native-app-trace.log').read_text()
header = trace.splitlines()[0]
unit_id = (root / 'cfgfix-native-app-unit-id.txt').read_text().strip()
if not header.startswith('phase=cfgfix-app-start ') or 'no-substitute=true' not in header or 'verified_producer=true' not in header or f'selected_unit={unit_id}' not in header:
    raise SystemExit('app lacked exact signed no-substitute and selected-root binding')
if 'phase=cfgfix-app-done ' not in trace or 'timeout: sending signal TERM' in trace:
    raise SystemExit('a timed-out CLI cannot prove a Rust app')
report = json.loads((root / 'cfgfix-native-app-report.json').read_text())
counts = report['counts']
if any(counts[key] != value for key, value in {'failed_total': 0, 'succeeded_total': 3, 'built_total': 1, 'cached_total': 2}.items()) or len(report['outcomes']) != 3:
    raise SystemExit(f'producer and actual app did not both succeed: {counts}')
producer = goal_outcome(report, 'producer', 'cargo-build-script-units-native-cfgfix-87lq')
app = goal_outcome(report, 'app', 'cargo-build-script-units-app-check-cfgfix-87lq')
original = json.loads((root / 'cfgfix-native-producer-report.json').read_text())
original_producer = goal_outcome(original, 'cargo-build-script-units-native-cfgfix-87lq', 'cargo-build-script-units-native-cfgfix-87lq')
if producer['drv_key'] != original_producer['drv_key'] or not producer['cached'] or app['cached']:
    raise SystemExit('exact producer and uncached Rust application check required')


def physical_output(row, name, is_file):
    paths = [Path(item['path']) for item in row['outputs'] if item['name'] == name]
    if len(paths) != 1 or paths[0].parent != store or paths[0].is_symlink():
        raise SystemExit(f'exactly one physical same-client output expected for {row["label"]} {name}: {paths}')
    if not (paths[0].is_file() if is_file else paths[0].is_dir()):
        raise SystemExit(f'physical same-client output absent: {paths[0]}')
    return paths[0]


outputs = {name: physical_output(producer, name, name == 'plan') for name in ('plan', 'out', 'sources')}
app_dir = physical_output(app, 'out', False)
result = app_dir / 'result.txt'
if result.is_symlink() or result.read_text() != 'cargo-build-script-fixture: proc-macro2 serde-derive libc native-sys = 42\n':
    raise SystemExit(f'actual Rust application result not exact answer 42: {result}')
plan_text = outputs['plan'].read_text()
plan = json.loads(plan_text)
if plan['schema'] != 'mantle-plan-v2' or len(plan['units']) != 34 or len(plan['sources']) != 14 or plan['roots'] != [unit_id]:
    raise SystemExit('actual producer graph not the signed 34-unit/14-slice selected fixture')
helper = (root / 'cfgfix-helper-path.txt').read_text().strip()
if helper not in plan_text or '/nix/store/vqssk3gwk0cjkcqyzv40ay96iflxfl9l-mantle-cargo-unit-helper-current-87lq' in plan_text:
    raise SystemExit('signed corrected helper not exclusively selected by the dynamic plan')
plan_output = next(item for item in producer['outputs'] if item['name'] == 'plan')
logical_plan = admitted_logical_output(report, plan_output)
plans = [row for row in report['native_dynamic_plans'] if row.get('plan_artifact_path') == logical_plan]
if len(plans) != 1 or plans[0]['rejection_reason'] is not None or plans[0]['scheduler_action'] != 'registered-roots':
    raise SystemExit('actual native graph was not admitted as registered roots')
plan_ids = [unit['id'] for unit in plan['units']]
accepted_ids = plans[0]['accepted_unit_ids']
if len(plan_ids) != len(set(plan_ids)) or len(accepted_ids) != len(set(accepted_ids)) or set(accepted_ids) != set(plan_ids):
    raise SystemExit('accepted app-plan unit IDs do not exactly match unique physical plan IDs')
if len(plans[0]['accepted_unit_ids']) != 34 or len(plans[0]['source_slices']) != 14:
    raise SystemExit('actual accepted graph lacks exact unit or source slice coverage')
for item in plans[0]['source_slices']:
    if item['declared_nar_blake3'] != item['observed_nar_blake3'] or not item['admitted_store_path']:
        raise SystemExit(f'source slice hash/admission mismatch: {item}')
root_unit = next(unit for unit in plan['units'] if unit['id'] == unit_id)
root_name = root_unit['derivation']['name']
selected_root = goal_outcome(report, root_name + '.drv', root_name)
original_root = goal_outcome(original, root_name + '.drv', root_name)
if selected_root['drv_key'] != original_root['drv_key'] or not selected_root['cached']:
    raise SystemExit('selected application root identity/reuse differs from original admitted graph')
trusted = 'mantle-unit-plan-local-1:jlpkELaOmQsXYXwNvRQxPfzO1fqmMfJ4yxpj4hbt2Kw=,mantle-unit-plan-remote-87lq:qXjNX6CsTA+vCkoh+nZDI3psVEfCLnvLhV4ZhfiDnt8='
verified = []
for path in (*outputs.values(), app_dir):
    command = [str(root / 'target/debug/mantle'), '--nix-compat', '--store', str(store), '--state-dir', str(root / 'client-state'), 'store', 'verify', path.name, '--trusted-public-keys', trusted]
    finished = subprocess.run(command, capture_output=True, text=True, timeout=600)
    text = finished.stdout + finished.stderr
    verified.append(text)
    if finished.returncode or not re.search(rf'OK {re.escape(path.name)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*', text) or '1 checked, 0 mismatches' not in text:
        raise SystemExit(f'actual output failed trusted physical verification: {text}')
(root / 'cfgfix-native-app-verify.log').write_text(''.join(verified))
print(f'actual-signed-client-rust-app-42=success result={result} units=34 slices=14 built={counts["built_total"]} cached={counts["cached_total"]} verified_outputs=4', flush=True)
