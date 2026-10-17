#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Verify only the completed first app execution; never replay a build."""
import datetime
import json
import os
import re
import signal
import subprocess
import time
from pathlib import Path

def run_owned_stage(command, deadline, log):
    """Bound one stage and tear down its known sessions before its leader exits."""
    child = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT,
                             cwd=root / 'source', start_new_session=True)
    groups = {child.pid}
    (root / 'owned-proof-cfgfix.pgid').write_text(str(child.pid) + '\n')
    finish_by = time.monotonic() + deadline
    try:
        while True:
            # The existing app runner creates an inner owned session. Capture its
            # live leader now so cleanup does not depend on that leader surviving.
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
        # Known stage group IDs remain cleanup authority even if their leaders
        # exited; never require a dead leader's /proc entry to kill descendants.
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

def require_prior_settlement(state):
    expected = {
        '1044': ('mantle-unit-plan-87lq-cfgfix-20261014', 'Success'),
        '1045': ('mantle-unit-plan-87lq-cfgfix-guard-20261014', 'Success'),
        '1047': ('mantle-unit-plan-87lq-cfgfix-app-gpt61-20261014', {'Failed': 1}),
        '1048': ('mantle-unit-plan-87lq-cfgfix-app-guard-gpt61-20261014', 'Success'),
        '1049': ('mantle-unit-plan-87lq-variant-supervisor-provider-config-gpt61-20261014', 'DependencyFailed'),
        '1051': ('mantle-unit-plan-87lq-cfgfix-app-logical-repair-20261014', {'Failed': 1}),
        '1052': ('mantle-unit-plan-87lq-cfgfix-app-logical-repair-guard-20261014', 'Success'),
        '1053': ('mantle-unit-plan-87lq-variant-supervisor-provider-config-logical-repair-20261014', 'DependencyFailed'),
    }
    for task_id, (label, result) in expected.items():
        task = state.get(task_id, {})
        if task.get('label') != label or task.get('status', {}).get('Done', {}).get('result') != result:
            raise SystemExit(f'exact terminal prior proof/guard history required: {task_id}')


root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
python = '/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14'
state = json.loads(subprocess.check_output(['pueue', 'status', '--json'], text=True, timeout=30))['tasks']
require_prior_settlement(state)
allowed = {
    'mantle-unit-plan-87lq-cfgfix-app-goal-acceptance-20261014': f'{python} {root}/run-cfgfix-app-goal-acceptance-20261014.py',
    'mantle-unit-plan-87lq-cfgfix-goal-acceptance-supervisor-20261014': f'{python} {root}/queue-cfgfix-app-goal-acceptance-20261014.py',
}
for task in state.values():
    status = task.get('status', {})
    command = task.get('command', '')
    if not isinstance(status, dict) or 'Running' not in status or str(root) not in command:
        continue
    if task.get('label') == 'mantle-unit-plan-87lq-cfgfix-app-goal-acceptance-guard-20261014' and 'watch-owned-proof-cfgfix.py' in command:
        continue
    if allowed.get(task.get('label')) != command:
        raise SystemExit('acceptance-only owner refuses same-client overlap')
prior = (root / 'cfgfix-native-app-repair-logical-binding-20261014.log').read_text()
if 'phase=actual-no-substitute-app-done exit=0 ' not in prior or 'exact producer and uncached Rust application check required' not in prior:
    raise SystemExit('preserved successful first execution and exact1051 acceptance failure required')
trace = (root / 'cfgfix-native-app-trace.log').read_text()
if 'phase=cfgfix-app-done ' not in trace or 'timeout: sending signal TERM' in trace:
    raise SystemExit('completed first actual app trace required; never rerun it here')
report = json.loads((root / 'cfgfix-native-app-report.json').read_text())
if report['counts'] != {'succeeded_total': 3, 'built_total': 1, 'cached_total': 2, 'failed_total': 0}:
    raise SystemExit('successful three-root first app report required')
if (root / 'cfgfix-native-app-verify.log').exists():
    raise SystemExit('preserving existing app verification receipt; refusing replay')
for path, floor in ((root, 50 * 1024**3), (Path('/home/brittonr'), 5 * 1024**3 // 2)):
    stat = os.statvfs(path)
    if stat.f_bavail * stat.f_frsize < floor:
        raise SystemExit('acceptance-only resource floor unavailable')
os.chdir(root / 'source')
if os.getpgrp() != os.getpid():
    os.setsid()
with (root / 'cfgfix-native-app-goal-acceptance-20261014.log').open('x') as log:
    task_id, guard_id = require_running_guard('mantle-unit-plan-87lq-cfgfix-app-goal-acceptance-20261014', f'{python} {root}/run-cfgfix-app-goal-acceptance-20261014.py', 'mantle-unit-plan-87lq-cfgfix-app-goal-acceptance-guard-20261014', 3600)
    log.write(f'guard-ready task={task_id} guard={guard_id}\n')
    log.write(f'phase=trusted-first-app-acceptance-start time={datetime.datetime.now(datetime.UTC).isoformat()}\n')
    log.flush()
    code = run_owned_stage([python, str(root / 'check-cfgfix-native-app-20261014.py')], 3000, log)
    log.write(f'phase=trusted-first-app-acceptance-done exit={code} time={datetime.datetime.now(datetime.UTC).isoformat()}\n')
    log.flush()
    if code:
        raise SystemExit(code)
print('accepted-first-cfgfix-app=success; original actual app/report untouched; no build replay', flush=True)
