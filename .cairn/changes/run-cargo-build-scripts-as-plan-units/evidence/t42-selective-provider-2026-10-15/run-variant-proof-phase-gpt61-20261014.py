#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Run exactly one guarded provider-only proof phase after genuine cfgfix acceptance."""
import datetime
import json
import os
import re
import signal
import subprocess
import sys
import time
from pathlib import Path

def run_owned_stage(command, deadline, log):
    """Bound one stage and tear down its known sessions before its leader exits."""
    child = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT, cwd=root / 'source', start_new_session=True)
    groups = {child.pid}
    (root / 'owned-proof-cfgfix.pgid').write_text(str(child.pid) + '\n')
    finish_by = time.monotonic() + deadline
    try:
        while True:
            try:
                pgid = int((root / 'owned-proof-cfgfix.pgid').read_text().strip())
                if pgid != os.getpid() and os.getpgid(pgid) == pgid:
                    cmdline = Path(f'/proc/{pgid}/cmdline').read_bytes()
                    cwd = Path(f'/proc/{pgid}/cwd').resolve()
                    if str(root).encode() in cmdline and cwd == root / 'source':
                        groups.add(pgid)
            except (OSError, ValueError):
                pass
            remaining = finish_by - time.monotonic()
            if remaining <= 0:
                raise subprocess.TimeoutExpired(command, deadline)
            try:
                return child.wait(timeout=min(5, remaining))
            except subprocess.TimeoutExpired:
                continue
    finally:
        for pgid in groups:
            try:
                os.killpg(pgid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        try:
            child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            pass
        for pgid in groups:
            try:
                os.killpg(pgid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        child.wait(timeout=3)


def require_running_guard(label, command, guard_label, duration):
    state = json.loads(subprocess.check_output(['pueue', 'status', '--json'], text=True, timeout=10))['tasks']
    owners = [(task_id, task) for task_id, task in state.items()
              if task.get('label') == label and task.get('command') == command
              and isinstance(task.get('status'), dict) and 'Running' in task['status']]
    guards = [(task_id, task) for task_id, task in state.items() if task.get('label') == guard_label]
    if len(owners) != 1 or len(guards) != 1:
        raise SystemExit('one exact running client owner and its declared watchdog required')
    task_id = owners[0][0]
    guard_id, guard = guards[0]
    expected = f'{python} {root}/watch-owned-proof-cfgfix.py {task_id} {label} {duration}'
    if guard.get('command') != expected or not isinstance(guard.get('status'), dict) or 'Running' not in guard['status']:
        raise SystemExit('declared watchdog is not Running; refusing client stage')
    record = json.loads(subprocess.check_output(['pueue', 'log', guard_id, '--json', '--lines', '1'], text=True, timeout=10)).get(guard_id, {})
    observed = record.get('task', {})
    if observed.get('command') != expected or not isinstance(observed.get('status'), dict) or 'Running' not in observed['status'] or f'guard-progress id={task_id} ' not in (record.get('output') or ''):
        raise SystemExit('exact Running watchdog resource tick missing; refusing client stage')
    return task_id, guard_id

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
python = '/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14'
if sys.argv[1:] not in (['provider-config'], ['producer'], ['app']):
    raise SystemExit('one exact variant phase required: provider-config, producer, or app')
mode = sys.argv[1]
guard_labels = {
    'provider-config': 'mantle-unit-plan-87lq-variant-config-guard-gpt61-20261014',
    'producer': 'mantle-unit-plan-87lq-variant-producer-verify-guard-gpt61-20261014',
    'app': 'mantle-unit-plan-87lq-variant-app-verify-guard-gpt61-20261014',
}
state = json.loads(subprocess.check_output(['pueue', 'status', '--json'], text=True, timeout=30))['tasks']
for task in state.values():
    status = task.get('status', {})
    if not isinstance(status, dict) or 'Running' not in status or str(root) not in task.get('command', ''):
        continue
    own = task.get('command') == f'{python} {root}/run-variant-proof-phase-gpt61-20261014.py {mode}'
    guard = 'watch-owned-proof-cfgfix.py' in task.get('command', '') and task.get('label') == guard_labels[mode]
    if not own and not guard:
        raise SystemExit(f'variant phase refuses accidental overlap or unfinished prior guard: {task.get("id")}')
for name, marker in (('cfgfix-native-producer-trace.log', 'phase=cfgfix-producer-done '),
                     ('cfgfix-native-app-trace.log', 'phase=cfgfix-app-done ')):
    trace = (root / name).read_text()
    if marker not in trace or 'timeout: sending signal TERM' in trace:
        raise SystemExit(f'accepted cfgfix baseline required: {name}')
base = json.loads((root / 'cfgfix-native-app-report.json').read_text())
apps = [goal_outcome(base, 'app', 'cargo-build-script-units-app-check-cfgfix-87lq')]
if base['counts']['failed_total'] or len(apps) != 1 or apps[0]['cached']:
    raise SystemExit('genuine executed successful cfgfix application required')
paths = [Path(item['path']) for item in apps[0]['outputs'] if item['name'] == 'out']
if len(paths) != 1 or paths[0].parent != root / 'client-store' or paths[0].is_symlink() or (paths[0] / 'result.txt').is_symlink() or (paths[0] / 'result.txt').read_text() != 'cargo-build-script-fixture: proc-macro2 serde-derive libc native-sys = 42\n':
    raise SystemExit('genuine same-client accepted42 result missing')
receipt = (root / 'cfgfix-native-app-verify.log').read_text()
if not re.search(rf'^OK {re.escape(paths[0].name)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*$', receipt, re.M) or '1 checked, 0 mismatches' not in receipt:
    raise SystemExit('genuine trusted physical baseline app receipt missing')
if mode != 'provider-config':
    old = json.loads((root / 'cfgfix-native-config.json').read_text())
    changed = json.loads((root / 'variant-native-config.json').read_text())
    old_provider = next(iter(old['native_inputs'].values()))['inputs']['answer']
    new_provider = next(iter(changed['native_inputs'].values()))['inputs']['answer']
    next(iter(old['native_inputs'].values()))['inputs']['answer'] = new_provider
    if old == changed and old_provider != new_provider:
        pass
    else:
        raise SystemExit('variant config changed beyond its sole provider root')
    for name in ('variant-native-provider-verify.log', 'variant-native-config-verify.log'):
        text = (root / name).read_text()
        if not re.search(r'^OK [^ ]+  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*$', text, re.M) or '1 checked, 0 mismatches' not in text:
            raise SystemExit(f'named trusted variant input prerequisite missing: {name}')
if any(os.statvfs(path).f_bavail * os.statvfs(path).f_frsize < floor for path, floor in ((root, 50 * 1024**3), (Path('/home/brittonr'), 5 * 1024**3 // 2))):
    raise SystemExit('variant phase resource floor unavailable')
commands = {
    'provider-config': [
        ('actual-native-provider-02', ['bash', str(root / 'run-native-variant.sh')], 7300),
        ('trusted-provider-only-config', [python, str(root / 'prepare-variant-native-config-gpt61-20261014.py')], 3300),
    ],
    'producer': [
        ('actual-provider-only-graph', ['bash', str(root / 'run-variant-native-app.sh')], 64900),
        ('trusted-full-variant-plan', [python, str(root / 'verify-variant-native-outputs-gpt61-20261014.py'), 'producer'], 2400),
        ('select-actual-variant-app', [python, str(root / 'make-variant-native-app-consumer.py')], 120),
    ],
    'app': [
        ('actual-variant-rust-app', ['bash', str(root / 'run-variant-native-consumer.sh')], 64900),
        ('trusted-exact-variant-app', [python, str(root / 'verify-variant-native-outputs-gpt61-20261014.py'), 'app'], 1200),
        ('strict-selective-provider-comparison', [python, str(root / 'check-native-provider-selective-rerun.py')], 120),
    ],
}
os.chdir(root / 'source')
if os.getpgrp() != os.getpid():
    os.setsid()
with (root / f'variant-native-phase-{mode}-gpt61-20261014.log').open('x') as log:
    for name, command, deadline in commands[mode]:
        task_id, guard_id = require_running_guard(f'mantle-unit-plan-87lq-variant-{mode}-gpt61-20261014', f'{python} {root}/run-variant-proof-phase-gpt61-20261014.py {mode}', guard_labels[mode], 12000 if mode == 'provider-config' else 69600)
        log.write(f'guard-ready task={task_id} guard={guard_id}\n')
        log.write(f'phase={name}-start time={datetime.datetime.now(datetime.UTC).isoformat()}\n')
        log.flush()
        returncode = run_owned_stage(command, deadline, log)
        log.write(f'phase={name}-done exit={returncode} time={datetime.datetime.now(datetime.UTC).isoformat()}\n')
        log.flush()
        if returncode:
            raise SystemExit(returncode)
print(f'actual-variant-phase={mode} success; only completed strict comparison is selectivity evidence', flush=True)
