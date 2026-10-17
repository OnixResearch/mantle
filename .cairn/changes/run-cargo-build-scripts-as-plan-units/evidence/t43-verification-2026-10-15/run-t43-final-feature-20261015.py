#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Verify final feature-owned formatted helper source and real all-target inputs."""
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
prior = json.loads((root / 't43-canonical-quality-summary-20261015.json').read_text())
staged = json.loads((root / 't43-staged-compile-inputs-20261015.json').read_text())
env = os.environ.copy()
env.update(prior['env_overrides'])
env['TMPDIR'] = str(root / 'tmp/t43-final-feature-20261015')
Path(env['TMPDIR']).mkdir(exist_ok=True)
nightly = '/nix/store/33vf8n9jip02hlkx1i96pkkgbg3aqc1z-mantle-unit-plan-nightly/bin/'
cargo = [nightly + 'cargo', '--config', '.cargo/vendor-config.toml', '--config',
         'target.x86_64-unknown-linux-gnu.linker="' + env['CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER'] + '"']
feature_files = ['crates/mantle-cargo-unit-plan/src/lib.rs',
                 'crates/mantle-cargo-unit-plan/tests/lowering.rs',
                 'crates/mantle-cargo-unit-plan/tests/producer.rs']
package_files = ['Cargo.toml', 'src/unit_helper.rs', 'src/producer.rs', 'src/directives.rs', 'src/lib.rs',
                 'src/source_nar.rs', 'tests/helper.rs', 'tests/producer.rs', 'tests/directives.rs',
                 'tests/lowering.rs', 'tests/source_nar.rs']
inputs = sorted(set([item['path'] for item in prior['source_inputs']] +
                    ['crates/mantle-cargo-unit-plan/' + name for name in package_files] +
                    [item['path'] for item in staged['files']] +
                    ['Cargo.toml', 'Cargo.lock', 'scripts/quality-gate-common.sh']))
def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()
def hashes():
    return [{'path': name, 'bytes': (source / name).stat().st_size,
             'transport_sha256': hashlib.sha256((source / name).read_bytes()).hexdigest()} for name in inputs]
def floor():
    result = {}
    for label, path, bound in [('root', root, 50 * 1024**3), ('home', '/home/brittonr', 2684354560)]:
        state = os.statvfs(path)
        result[label] = {'available_bytes': state.f_bavail * state.f_frsize, 'available_inodes': state.f_favail}
        if result[label]['available_bytes'] < bound:
            raise RuntimeError('t43-final-resource-bound ' + json.dumps(result))
    return result
for item in staged['files']:
    data = (source / item['path']).read_bytes()
    if len(data) != item['bytes'] or hashlib.sha256(data).hexdigest() != item['transport_sha256']:
        raise RuntimeError('staged-compile-input-mismatch ' + item['path'])
steps = [
    ('feature-format-final', [nightly + 'rustfmt', '--config-path', str(source / 'rustfmt.toml'), '--config', 'skip_children=true'] + feature_files, 120),
    ('helper-fmt-final', [nightly + 'cargo', 'fmt', '--check', '-p', 'mantle-cargo-unit-plan'], 180),
    ('helper-package-final', cargo + ['test', '--locked', '--offline', '-p', 'mantle-cargo-unit-plan', '--lib', '--tests', '--bins', '--', '--nocapture', '--test-threads=1'], 1200),
    ('helper-clippy-final', cargo + ['clippy', '--locked', '--offline', '-p', 'mantle-cargo-unit-plan', '--all-targets', '--no-deps', '--', '-D', 'warnings'], 1800),
    ('root-clippy-final', cargo + ['clippy', '--locked', '--offline', '-p', 'mantle', '--all-targets', '--no-deps', '--', '-D', 'warnings'], 2400),
]
summary = {'schema': 'mantle-t43-final-feature-verification-v1', 'start_utc': utc(), 'cwd': str(source),
           'env_overrides': {**prior['env_overrides'], 'TMPDIR': env['TMPDIR']},
           'inputs_before_final_feature_format': hashes(), 'staged_compile_inputs': staged,
           'scope': 'final helper package and mantle all-targets Clippy; global canonical first-party fmt remains failed and is not replayed or hidden',
           'steps': [], 'signed_proof_replay': False}
summary_path = root / 't43-final-feature-summary-20261015.json'
for name, command, timeout in steps:
    path = root / ('t43-' + name + '-20261015.log')
    record = {'name': name, 'command': command, 'timeout_seconds': timeout, 'log': str(path),
              'start_utc': utc(), 'preflight': floor(), 'actual_inputs_before_command': hashes()}
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
        footer = {key: value for key, value in record.items() if key != 'actual_inputs_before_command'}
        log.write(('\n--- end ---\n' + json.dumps(footer, sort_keys=True) + '\n').encode())
    raw = path.read_bytes()
    record.update({'bytes': len(raw), 'transport_sha256': hashlib.sha256(raw).hexdigest()})
    summary['steps'].append(record)
    summary['actual_final_inputs'] = hashes()
    summary_path.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({key: value for key, value in record.items() if key != 'actual_inputs_before_command'}, sort_keys=True), flush=True)
summary.update({'end_utc': utc(), 'result': 'Success' if all(item['exit'] == 0 for item in summary['steps']) else 'Failed'})
summary_path.write_text(json.dumps(summary, indent=2) + '\n')
print('t43-final-feature=' + summary['result'], flush=True)
sys.exit(0 if summary['result'] == 'Success' else 1)
