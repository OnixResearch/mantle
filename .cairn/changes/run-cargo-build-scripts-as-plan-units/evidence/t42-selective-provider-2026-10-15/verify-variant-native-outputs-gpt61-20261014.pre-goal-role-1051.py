#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Retain named actual trusted variant producer/app receipts, never a synthetic claim."""
import json
import os
import re
import subprocess
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

root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
if sys.argv[1:] not in (['producer'], ['app']):
    raise SystemExit('one exact verification mode required: producer or app')
mode = sys.argv[1]
state = json.loads(subprocess.check_output(['pueue', 'status', '--json'], text=True, timeout=30))['tasks']
for task in state.values():
    status = task.get('status', {})
    if not isinstance(status, dict) or 'Running' not in status or str(root) not in task.get('command', ''):
        continue
    own = ('verify-variant-native-outputs-gpt61-20261014.py ' + mode in task.get('command', '')
           or task.get('command') == f'/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14 {root}/run-variant-proof-phase-gpt61-20261014.py ' + ('producer' if mode == 'producer' else 'app'))
    guard = 'watch-owned-proof-cfgfix.py' in task.get('command', '') and task.get('label') == f'mantle-unit-plan-87lq-variant-{mode}-verify-guard-gpt61-20261014'
    if not own and not guard:
        raise SystemExit(f'variant verification refuses active same-client task or prior guard: {task.get("id")}')
filename = 'variant-native-app.log' if mode == 'producer' else 'variant-native-app-consumer.log'
header_prefix = f'variant-app-native-root-source={root}/variant-native-plan.ncl ' if mode == 'producer' else 'sandbox-shell='
text = (root / filename).read_text()
header, separator, payload = text.partition('\n')
if not separator or not header.startswith(header_prefix) or 'no-substitute=true' not in header or 'timeout: sending signal TERM' in text:
    raise SystemExit(f'actual completed no-substitute variant report required: {filename}')
report = json.loads(payload)
label = 'cargo-build-script-units-native-variant-87lq' if mode == 'producer' else 'cargo-build-script-units-app-check-variant-87lq'
rows = [row for row in report['outcomes'] if row['label'] == label]
if report['counts']['failed_total'] or len(rows) != 1 or rows[0]['cached']:
    raise SystemExit(f'one actually built successful variant {mode} required')
outputs = {item['name']: Path(item['path']) for item in rows[0]['outputs']}
if set(outputs) != ({'out', 'plan', 'sources'} if mode == 'producer' else {'out'}):
    raise SystemExit('variant physical output set incomplete')
for name, path in outputs.items():
    if path.parent != root / 'client-store' or path.is_symlink() or not (path.is_file() if name == 'plan' else path.is_dir()):
        raise SystemExit(f'variant output missing from physical same client: {path}')
if mode == 'producer':
    plan = json.loads(outputs['plan'].read_text())
    plan_output = next(item for item in rows[0]['outputs'] if item['name'] == 'plan')
    logical_plan = admitted_logical_output(report, plan_output)
    admissions = [row for row in report['native_dynamic_plans'] if row.get('plan_artifact_path') == logical_plan]
    if plan['schema'] != 'mantle-plan-v2' or len(plan['units']) != 34 or len(plan['sources']) != 14 or len(plan['roots']) != 1 or len(admissions) != 1:
        raise SystemExit('variant plan is not the full fixture graph')
    admitted = admissions[0]
    if admitted['rejection_reason'] is not None or admitted['scheduler_action'] != 'registered-roots' or len(admitted['accepted_unit_ids']) != 34 or len(admitted['source_slices']) != 14:
        raise SystemExit('variant plan roots/slices were not fully admitted')
    plan_ids = [unit['id'] for unit in plan['units']]
    accepted_ids = admitted['accepted_unit_ids']
    if len(plan_ids) != len(set(plan_ids)) or len(accepted_ids) != len(set(accepted_ids)) or set(accepted_ids) != set(plan_ids):
        raise SystemExit('accepted variant unit IDs do not exactly match unique physical plan IDs')
    if any(row['declared_nar_blake3'] != row['observed_nar_blake3'] or not row['admitted_store_path'] for row in admitted['source_slices']):
        raise SystemExit('variant source admission hash mismatch')
else:
    result = outputs['out'] / 'result.txt'
    if result.is_symlink() or result.read_text() != 'cargo-build-script-fixture: proc-macro2 serde-derive libc native-sys = 42\n':
        raise SystemExit('actual variant Rust app did not produce the exact result')
receipt = root / f'variant-native-{mode}-verify.log'
if receipt.exists():
    raise SystemExit(f'preserving existing named receipt; refusing replay: {receipt}')
if any(os.statvfs(path).f_bavail * os.statvfs(path).f_frsize < floor for path, floor in ((root, 50 * 1024**3), (Path('/home/brittonr'), 5 * 1024**3 // 2))):
    raise SystemExit('variant verification resource floor unavailable')
os.chdir(root / 'source')
if os.getpgrp() != os.getpid():
    os.setsid()
(root / 'owned-proof-cfgfix.pgid').write_text(str(os.getpid()) + '\n')
trusted = 'mantle-unit-plan-local-1:jlpkELaOmQsXYXwNvRQxPfzO1fqmMfJ4yxpj4hbt2Kw=,mantle-unit-plan-remote-87lq:qXjNX6CsTA+vCkoh+nZDI3psVEfCLnvLhV4ZhfiDnt8='
with receipt.open('x') as log:
    for path in outputs.values():
        result = subprocess.run([str(root / 'target/debug/mantle'), '--nix-compat', '--store', str(root / 'client-store'), '--state-dir', str(root / 'client-state'), 'store', 'verify', path.name, '--trusted-public-keys', trusted], capture_output=True, text=True, timeout=600)
        text = result.stdout + result.stderr
        log.write(text)
        log.flush()
        if result.returncode or not re.search(rf'^OK {re.escape(path.name)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*$', text, re.M) or '1 checked, 0 mismatches' not in text:
            raise SystemExit(f'variant physical output trust failed: {path.name}')
print(f'actual-variant-{mode}-trusted-physical-outputs=success count={len(outputs)} receipt={receipt}; selectivity still requires comparison', flush=True)
