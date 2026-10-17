#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Queue one guarded variant phase and its next success-only callback, then exit."""
import json
import subprocess
import sys
import time
from pathlib import Path



def wait_guard_ready(task_id, guard_id, guard_label, command, timeout_seconds=30):
    """Observe exact Running watchdog and its resource tick; never force-start."""
    until = time.monotonic() + timeout_seconds
    while time.monotonic() < until:
        tasks = json.loads(subprocess.check_output(['pueue', 'status', '--json'], text=True, timeout=5))['tasks']
        guard = tasks.get(str(guard_id), {})
        status = guard.get('status', {})
        if guard.get('label') != guard_label or guard.get('command') != command:
            raise SystemExit('watchdog identity mismatch; client task remains stashed')
        if isinstance(status, dict) and 'Done' in status:
            raise SystemExit('watchdog ended before guarded launch; client task remains stashed')
        if isinstance(status, dict) and 'Running' in status:
            record = json.loads(subprocess.check_output(
                ['pueue', 'log', str(guard_id), '--json', '--lines', '1'],
                text=True, timeout=5)).get(str(guard_id), {})
            task = record.get('task', {})
            observed = task.get('status', {})
            if task.get('label') == guard_label and task.get('command') == command and isinstance(observed, dict) and 'Running' in observed and f'guard-progress id={task_id} ' in (record.get('output') or ''):
                return
        time.sleep(0.25)
    raise SystemExit('watchdog not observably Running with exact resource tick; client task remains stashed')



root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
python = '/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14'
if len(sys.argv) != 4 or sys.argv[1] not in ('provider-config', 'producer', 'app') or not all(value.isdecimal() for value in sys.argv[2:]):
    raise SystemExit('one phase and exact previous task+guard IDs required')
mode, previous, previous_guard = sys.argv[1:]
guard_labels = {
    'provider-config': 'mantle-unit-plan-87lq-variant-config-guard-gpt61-20261014',
    'producer': 'mantle-unit-plan-87lq-variant-producer-verify-guard-gpt61-20261014',
    'app': 'mantle-unit-plan-87lq-variant-app-verify-guard-gpt61-20261014',
}
expected_labels = {
    'provider-config': ('mantle-unit-plan-87lq-cfgfix-app-goal-acceptance-20261014', 'mantle-unit-plan-87lq-cfgfix-app-goal-acceptance-guard-20261014'),
    'producer': ('mantle-unit-plan-87lq-variant-provider-config-gpt61-20261014', guard_labels['provider-config']),
    'app': ('mantle-unit-plan-87lq-variant-producer-gpt61-20261014', guard_labels['producer']),
}
state = json.loads(subprocess.check_output(['pueue', 'status', '--json'], text=True, timeout=30))['tasks']
for task_id, label in zip((previous, previous_guard), expected_labels[mode]):
    task = state.get(task_id, {})
    if task.get('label') != label or task.get('status', {}).get('Done', {}).get('result') != 'Success':
        raise SystemExit(f'variant callback refuses unfinished or failed previous task/guard: {task_id}')
label = f'mantle-unit-plan-87lq-variant-{mode}-gpt61-20261014'
if any(task.get('label') in (label, guard_labels[mode]) for task in state.values()):
    raise SystemExit('variant phase already scheduled; refusing duplicate')


def add(arguments):
    task_id = subprocess.check_output(['pueue', 'add', '--print-task-id', '--after', previous, previous_guard, *arguments], text=True, timeout=30).strip()
    if not task_id.isdecimal():
        raise SystemExit('one exact variant task ID required')
    return task_id


phase = add(['--stashed', '--group', 'mantle-unit-plan-87lq-build', '--working-directory', str(root / 'source'), '--label', label,
             '--', f'{python} {root}/run-variant-proof-phase-gpt61-20261014.py {mode}'])
deadline = 12000 if mode == 'provider-config' else 69600
command = f'{python} {root}/watch-owned-proof-cfgfix.py {phase} {label} {deadline}'
guard = add(['--group', 'mantle-unit-plan-87lq-guard', '--working-directory', str(root / 'source'), '--label', guard_labels[mode], '--', command])
wait_guard_ready(phase, guard, guard_labels[mode], command)
subprocess.run(['pueue', 'enqueue', phase], check=True, timeout=30)
print(f'variant-phase-scheduled mode={mode} task={phase} guard={guard} success_dependencies={previous},{previous_guard} guard_seconds={deadline}; not execution success', flush=True)
next_mode = {'provider-config': 'producer', 'producer': 'app'}.get(mode)
if next_mode:
    callback = subprocess.check_output([
        'pueue', 'add', '--print-task-id', '--after', phase, guard,
        '--group', 'mantle-unit-plan-87lq-build', '--working-directory', str(root / 'source'),
        '--label', f'mantle-unit-plan-87lq-variant-supervisor-{next_mode}-gpt61-20261014', '--',
        f'{python} {root}/run-variant-proof-supervisor-gpt61-20261014.py {next_mode} {phase} {guard}',
    ], text=True, timeout=30).strip()
    if not callback.isdecimal():
        raise SystemExit('next variant callback task ID missing; preserve submitted phase history')
    print(f'next-variant-callback-scheduled mode={next_mode} task={callback} success_dependencies={phase},{guard}', flush=True)
# Never wait for the child phase: releasing this one-parallel build slot is mandatory.
