#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
import datetime
import hashlib
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
nightly = '/nix/store/33vf8n9jip02hlkx1i96pkkgbg3aqc1z-mantle-unit-plan-nightly'
gcc = '/nix/store/hb2bs5fg5wkm04x565737qd5nh2hy5nk-gcc-wrapper-15.2.0'
binutils = '/nix/store/x7ikkplbrv5dlihy1bqq32gp6lilkval-binutils-wrapper-2.44'
pkg = '/nix/store/0v0raqk1qw5g2a21km4xa1hwhaq4s976-pkg-config-wrapper-0.29.2'
source = root / 'source/crates/mantle-cargo-unit-plan/tests/helper.rs'
expected = '5102b381238f4508f823a466844d13fce4838bace0d7db8adb9f366485c9bf7c'
command = [nightly + '/bin/cargo', '--config', '.cargo/vendor-config.toml', '--config', 'target.x86_64-unknown-linux-gnu.linker="' + gcc + '/bin/gcc"', 'test', '--locked', '--offline', '-p', 'mantle-cargo-unit-plan', '--lib', '--tests', '--bins', '--', '--nocapture', '--test-threads=1']
with (root / 't43-helper-package-20261015.log').open('x') as log:
    payload = source.read_bytes()
    identity = hashlib.sha256(payload).hexdigest()
    print('source_bytes=' + str(len(payload)) + ' source_sha256=' + identity, file=log, flush=True)
    if len(payload) != 20761 or identity != expected:
        print('FAIL frozen source mismatch; no launch', file=log, flush=True)
        raise SystemExit(76)
    for path, floor in ((root, 50 * 1024**3), (Path('/home/brittonr'), 5 * 1024**3 // 2)):
        stat = os.statvfs(path)
        free = stat.f_bavail * stat.f_frsize
        print(f'preflight path={path} available_bytes={free} floor={floor}', file=log, flush=True)
        if free < floor:
            print('FAIL resource floor; no launch', file=log, flush=True)
            raise SystemExit(75)
    environment = os.environ.copy()
    environment.update({'PATH': ':'.join([nightly + '/bin', gcc + '/bin', binutils + '/bin', pkg + '/bin', environment.get('PATH', '')]), 'RUSTC': nightly + '/bin/rustc', 'RUSTFLAGS': '-C link-arg=-Wl,--allow-multiple-definition', 'CARGO_HOME': str(root / 'cargo-home'), 'CARGO_TARGET_DIR': str(root / 'target'), 'TMPDIR': str(root / 'tmp'), 'KACHE_CACHE_DIR': str(root / 'kache'), 'SNIX_BUILD_SANDBOX_SHELL': '/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox', 'PKG_CONFIG_PATH': '/nix/store/79xszwqidf98ag703b5c82q3pwg9lyzq-openssl-3.6.1-dev/lib/pkgconfig'})
    start = time.monotonic()
    print('start_utc=' + datetime.datetime.now(datetime.UTC).isoformat(), file=log, flush=True)
    print('command=' + repr(command), file=log, flush=True)
    child = subprocess.Popen(command, cwd=root / 'source', env=environment, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    try:
        result = child.wait(timeout=900)
    except subprocess.TimeoutExpired:
        print('FAIL timeout_seconds=900; terminating owned process group=' + str(child.pid), file=log, flush=True)
        os.killpg(child.pid, signal.SIGTERM)
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait()
        result = 124
    print('end_utc=' + datetime.datetime.now(datetime.UTC).isoformat(), file=log, flush=True)
    print(f'exit={result} elapsed_seconds={time.monotonic() - start:.6f}', file=log, flush=True)
raise SystemExit(result if result >= 0 else 128 - result)
