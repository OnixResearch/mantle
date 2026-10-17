#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Run only fresh T4.3 format, rail, planning and strict Clippy gates."""
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
nightly = Path('/nix/store/33vf8n9jip02hlkx1i96pkkgbg3aqc1z-mantle-unit-plan-nightly')
gcc = Path('/nix/store/hb2bs5fg5wkm04x565737qd5nh2hy5nk-gcc-wrapper-15.2.0')
binutils = Path('/nix/store/x7ikkplbrv5dlihy1bqq32gp6lilkval-binutils-wrapper-2.44')
pkg = Path('/nix/store/0v0raqk1qw5g2a21km4xa1hwhaq4s976-pkg-config-wrapper-0.29.2')
bwrap = '/nix/store/lqndphylsxqwbwm804n473pb4sqb98sh-bubblewrap-0.11.2/bin/bwrap'
env = os.environ.copy()
env.update({
    'PATH': ':'.join([str(nightly / 'bin'), str(gcc / 'bin'), str(binutils / 'bin'), str(pkg / 'bin'), str(Path(bwrap).parent), env.get('PATH', '')]),
    'RUSTC': str(nightly / 'bin/rustc'),
    'RUSTFLAGS': '-C link-arg=-Wl,--allow-multiple-definition',
    'CARGO_HOME': str(root / 'cargo-home'),
    'CARGO_TARGET_DIR': str(root / 'target'),
    'TMPDIR': str(root / 'tmp/t43-root-gates-20261015'),
    'KACHE_CACHE_DIR': str(root / 'kache'),
    'SNIX_BUILD_SANDBOX_SHELL': '/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox',
    'SNIX_BUILD_BWRAP': bwrap,
    'CRUNCH_NO_FUSE': '1',
    'PKG_CONFIG_PATH': '/nix/store/79xszwqidf98ag703b5c82q3pwg9lyzq-openssl-3.6.1-dev/lib/pkgconfig',
    'CARGO_TERM_COLOR': 'never',
})
Path(env['TMPDIR']).mkdir(exist_ok=True)
rust_files = [
    'tests/rust_compatibility_rail.rs',
    'tests/examples_inventory.rs',
    'examples/rust_compatibility_rail.rs',
    'crates/mantle-cargo-unit-plan/tests/helper.rs',
]
inputs = rust_files + [
    'examples/rust_compatibility_surface_matrix.ncl', 'examples/catalog.ncl',
    'examples/README.md', 'README.md', 'docs/operator-workflows.md',
    'docs/release-notes/cargo-build-script-units-2026-10-15.md',
    'rustfmt.toml', '.cargo/vendor-config.toml',
]
def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()
def available(path):
    state = os.statvfs(path)
    return {'available_bytes': state.f_bavail * state.f_frsize, 'available_inodes': state.f_favail}
def hashes():
    return [{'path': name, 'bytes': (source / name).stat().st_size,
             'sha256': hashlib.sha256((source / name).read_bytes()).hexdigest()} for name in inputs]
def preflight():
    facts = {'observed_utc': utc(), 'root': available(root), 'home': available('/home/brittonr')}
    if facts['root']['available_bytes'] < 50 * 1024**3 or facts['home']['available_bytes'] < 2684354560:
        raise RuntimeError('t43-gate-resource-bound ' + json.dumps(facts, sort_keys=True))
    return facts
cargo = [str(nightly / 'bin/cargo'), '--config', '.cargo/vendor-config.toml',
         '--config', 'target.x86_64-unknown-linux-gnu.linker="' + str(gcc / 'bin/gcc') + '"']
steps = [
    ('format', [str(nightly / 'bin/rustfmt'), '--config-path', str(source / 'rustfmt.toml')] + rust_files, 120),
    ('format-check', [str(nightly / 'bin/rustfmt'), '--check', '--config-path', str(source / 'rustfmt.toml')] + rust_files, 120),
    ('rail-inventory', cargo + ['test', '--locked', '--offline', '-p', 'mantle', '--test', 'rust_compatibility_rail', '--test', 'examples_inventory', '--', '--nocapture', '--test-threads=1'], 1200),
    ('no-oracle', cargo + ['test', '--locked', '--offline', '-p', 'mantle', '--test', 'rust_plan_cli', 'no_cargo_oracle', '--', '--nocapture', '--test-threads=1'], 1200),
    ('clippy-unit', cargo + ['clippy', '--locked', '--offline', '-p', 'mantle-cargo-unit-plan', '--lib', '--tests', '--bins', '--no-deps', '--', '-D', 'warnings'], 1500),
    ('clippy-root', cargo + ['clippy', '--locked', '--offline', '-p', 'mantle', '--test', 'rust_compatibility_rail', '--test', 'examples_inventory', '--example', 'rust_compatibility_rail', '--no-deps', '--', '-D', 'warnings'], 1500),
]
summary_path = root / 't43-root-gates-summary-20261015.json'
summary = {'schema': 'mantle-t43-fresh-gates-v1', 'start_utc': utc(), 'cwd': str(source),
           'env_overrides': {key: env[key] for key in ['PATH', 'RUSTC', 'RUSTFLAGS', 'CARGO_HOME', 'CARGO_TARGET_DIR', 'TMPDIR', 'KACHE_CACHE_DIR', 'SNIX_BUILD_SANDBOX_SHELL', 'SNIX_BUILD_BWRAP', 'CRUNCH_NO_FUSE', 'PKG_CONFIG_PATH', 'CARGO_TERM_COLOR']},
           'inputs_before_format': hashes(), 'steps': [], 'proof_replay': False}
for name, command, timeout in steps:
    log_path = root / ('t43-' + name + '-20261015.log')
    record = {'name': name, 'command': command, 'log': str(log_path), 'timeout_seconds': timeout,
              'start_utc': utc(), 'preflight': preflight()}
    started = time.monotonic()
    with log_path.open('xb') as log:
        header = 'start_utc=' + record['start_utc'] + '\ncwd=' + str(source) + '\ncommand=' + shlex.join(command) + '\npreflight=' + json.dumps(record['preflight'], sort_keys=True) + '\n--- stdout+stderr ---\n'
        log.write(header.encode())
        log.flush()
        child = subprocess.Popen(command, cwd=source, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            result = child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGTERM)
            try:
                child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
            result = 124
        record.update({'end_utc': utc(), 'elapsed_seconds': time.monotonic() - started, 'exit': result,
                       'postflight': preflight()})
        log.write(('\n--- end ---\n' + json.dumps(record, sort_keys=True) + '\n').encode())
    raw = log_path.read_bytes()
    record.update({'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
    summary['steps'].append(record)
    summary['inputs_after_format'] = hashes()
    summary_path.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(record, sort_keys=True), flush=True)
    if result:
        sys.exit(result)
summary.update({'end_utc': utc(), 'result': 'Success'})
summary_path.write_text(json.dumps(summary, indent=2) + '\n')
print('t43-fresh-root-gates=success', flush=True)
