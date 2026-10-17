#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Complete only remaining canonical format/package-wide strict quality coverage."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import time

root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
source = root / 'source'
prior = json.loads((root / 't43-root-gates-summary-20261015.json').read_text())
env = os.environ.copy()
env.update(prior['env_overrides'])
env['CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER'] = '/nix/store/hb2bs5fg5wkm04x565737qd5nh2hy5nk-gcc-wrapper-15.2.0/bin/gcc'
env['CARGO_NET_OFFLINE'] = 'true'
cargo = ['/nix/store/33vf8n9jip02hlkx1i96pkkgbg3aqc1z-mantle-unit-plan-nightly/bin/cargo', '--config', '.cargo/vendor-config.toml', '--config', 'target.x86_64-unknown-linux-gnu.linker="' + env['CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER'] + '"']
steps = [
    ('fmt-first-party', ['bash', '-c', 'source scripts/quality-gate-common.sh; cargo_fmt_first_party'], 180),
    ('fmt-unit-package', [cargo[0], 'fmt', '--check', '-p', 'mantle-cargo-unit-plan'], 180),
    ('clippy-unit-all-targets', cargo + ['clippy', '--locked', '--offline', '-p', 'mantle-cargo-unit-plan', '--all-targets', '--no-deps', '--', '-D', 'warnings'], 1800),
    ('clippy-root-all-targets', cargo + ['clippy', '--locked', '--offline', '-p', 'mantle', '--all-targets', '--no-deps', '--', '-D', 'warnings'], 2400),
]
def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()
def floor():
    facts = {}
    for label, path, bound in [('root', root, 50 * 1024**3), ('home', '/home/brittonr', 2684354560)]:
        state = os.statvfs(path)
        facts[label] = {'available_bytes': state.f_bavail * state.f_frsize, 'available_inodes': state.f_favail}
        if facts[label]['available_bytes'] < bound:
            raise RuntimeError('t43-quality-resource-bound ' + json.dumps(facts))
    return facts
summary = {'schema': 'mantle-t43-canonical-quality-v1', 'start_utc': utc(), 'cwd': str(source),
           'source_inputs': prior['inputs_after_format'], 'env_overrides': {**prior['env_overrides'], 'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER': env['CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER'], 'CARGO_NET_OFFLINE': 'true'},
           'prior_failure': '1067 clippy-unit exit101: clang missing; fixed by the existing canonical explicit linker environment, not source suppression',
           'steps': [], 'proof_or_focused_test_replay': False}
summary_path = root / 't43-canonical-quality-summary-20261015.json'
for name, command, timeout in steps:
    path = root / ('t43-' + name + '-20261015.log')
    record = {'name': name, 'command': command, 'timeout_seconds': timeout, 'log': str(path), 'start_utc': utc(), 'preflight': floor()}
    started = time.monotonic()
    with path.open('xb') as log:
        log.write(('start_utc=' + record['start_utc'] + '\ncwd=' + str(source) + '\ncommand=' + shlex.join(command) + '\npreflight=' + json.dumps(record['preflight'], sort_keys=True) + '\n--- stdout+stderr ---\n').encode())
        log.flush()
        child = subprocess.Popen(command, cwd=source, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGTERM)
            try:
                child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
            code = 124
        record.update({'end_utc': utc(), 'elapsed_seconds': time.monotonic() - started, 'exit': code, 'postflight': floor()})
        log.write(('\n--- end ---\n' + json.dumps(record, sort_keys=True) + '\n').encode())
    raw = path.read_bytes()
    record.update({'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
    summary['steps'].append(record)
    summary_path.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(record, sort_keys=True), flush=True)
summary.update({'end_utc': utc(), 'result': 'Success' if all(item['exit'] == 0 for item in summary['steps']) else 'Failed'})
summary_path.write_text(json.dumps(summary, indent=2) + '\n')
print('t43-canonical-quality=' + summary['result'], flush=True)
sys.exit(0 if summary['result'] == 'Success' else 1)
