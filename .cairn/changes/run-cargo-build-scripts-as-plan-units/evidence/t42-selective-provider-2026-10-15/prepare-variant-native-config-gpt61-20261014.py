#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Admit one provider-only variant config after the genuine cfgfix app succeeds."""
import json
import os
import re
import subprocess
from pathlib import Path

root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
python = '/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14'
cli = str(root / 'target/debug/mantle')
public = 'mantle-unit-plan-remote-87lq:qXjNX6CsTA+vCkoh+nZDI3psVEfCLnvLhV4ZhfiDnt8='
trusted = 'mantle-unit-plan-local-1:jlpkELaOmQsXYXwNvRQxPfzO1fqmMfJ4yxpj4hbt2Kw=,' + public
state = json.loads(subprocess.check_output(['pueue', 'status', '--json'], text=True, timeout=30))['tasks']
for task in state.values():
    status = task.get('status', {})
    if not isinstance(status, dict) or 'Running' not in status or str(root) not in task.get('command', ''):
        continue
    own = task.get('command') in (f'{python} {root}/prepare-variant-native-config-gpt61-20261014.py', f'{python} {root}/run-variant-proof-phase-gpt61-20261014.py provider-config')
    guard = 'watch-owned-proof-cfgfix.py' in task.get('command', '') and task.get('label') == 'mantle-unit-plan-87lq-variant-config-guard-gpt61-20261014'
    if not own and not guard:
        raise SystemExit(f'variant admission refuses active same-client task or prior guard: {task.get("id")}')
for name, completion in (('cfgfix-native-producer-trace.log', 'phase=cfgfix-producer-done '),
                         ('cfgfix-native-app-trace.log', 'phase=cfgfix-app-done ')):
    text = (root / name).read_text()
    if completion not in text or 'timeout: sending signal TERM' in text:
        raise SystemExit(f'completed cfgfix baseline required: {name}')
for name in ('cfgfix-native-producer-report.json', 'cfgfix-native-app-report.json'):
    report = json.loads((root / name).read_text())
    if report['counts']['failed_total']:
        raise SystemExit(f'failed cfgfix baseline: {name}')
for name in ('variant-native-provider-verify.log', 'variant-native-config.json',
             'variant-native-plan.template.ncl', 'variant-native-plan.ncl',
             'variant-native-plan-eval.json', 'variant-native-config-pull.log',
             'variant-native-config-verify.log', 'variant-native-config-admit-gpt61-20261014.log'):
    if (root / name).exists():
        raise SystemExit(f'preserving existing variant artifact; refusing replay: {name}')
if (root / 'gcroots/variant-config').exists() or (root / 'gcroots/variant-config').is_symlink():
    raise SystemExit('preserving existing variant config GC root; refusing replacement')
provider_text = (root / 'native-variant.log').read_text()
header, separator, payload = provider_text.partition('\n')
if not separator or not header.startswith('variant-native-provider-static-shell=') or 'no-substitute=true' not in header or 'timeout: sending signal TERM' in provider_text:
    raise SystemExit('actual no-substitute Mantle native 0.2 provider report required')
provider_report = json.loads(payload)
providers = [row for row in provider_report['outcomes'] if row['label'] == 'native-answer-0.2']
if provider_report['counts']['failed_total'] or len(providers) != 1 or providers[0]['cached']:
    raise SystemExit('native 0.2 provider must actually build successfully in this client')
outputs = [Path(item['path']) for item in providers[0]['outputs'] if item['name'] == 'out']
if len(outputs) != 1 or outputs[0].parent != root / 'client-store' or outputs[0].is_symlink() or not (outputs[0] / 'lib/libnative_answer.a').is_file() or not re.fullmatch(r'[0-9a-df-np-sv-z]{32}-native-answer-0\.2', outputs[0].name):
    raise SystemExit('actual Mantle 0.2 archive missing from the same physical client')
if any(os.statvfs(path).f_bavail * os.statvfs(path).f_frsize < floor for path, floor in ((root, 50 * 1024**3), (Path('/home/brittonr'), 5 * 1024**3 // 2))):
    raise SystemExit('variant admission resource floor unavailable')
os.chdir(root / 'source')
if os.getpgrp() != os.getpid():
    os.setsid()
(root / 'owned-proof-cfgfix.pgid').write_text(str(os.getpid()) + '\n')
with (root / 'variant-native-config-admit-gpt61-20261014.log').open('x') as log:
    def run(command, deadline):
        completed = subprocess.run(command, capture_output=True, text=True, timeout=deadline, cwd=root / 'source')
        text = completed.stdout + completed.stderr
        log.write(text)
        log.flush()
        if completed.returncode:
            raise SystemExit(f'variant admission command failed exit={completed.returncode}; see {log.name}')
        return text

    def verify(basename, filename):
        text = run([cli, '--nix-compat', '--store', str(root / 'client-store'), '--state-dir', str(root / 'client-state'), 'store', 'verify', basename, '--trusted-public-keys', trusted], 600)
        (root / filename).write_text(text)
        if not re.search(rf'^OK {re.escape(basename)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*$', text, re.M) or '1 checked, 0 mismatches' not in text:
            raise SystemExit(f'variant physical trust rejected: {filename}')

    provider = '/nix/store/' + outputs[0].name
    verify(outputs[0].name, 'variant-native-provider-verify.log')
    run([python, str(root / 'make-variant-native-plan.py'), provider], 60)
    added = run(['nix', 'store', 'add-path', '--offline', '--name', 'mantle-unit-plan-config-variant-87lq', str(root / 'variant-native-config.json')], 180)
    config_paths = re.findall(r'/nix/store/[0-9a-df-np-sv-z]{32}-mantle-unit-plan-config-variant-87lq', added)
    if len(config_paths) != 1:
        raise SystemExit('one exact signed variant config root required')
    config = config_paths[0]
    run(['nix-store', '--realise', '--indirect', '--add-root', str(root / 'gcroots/variant-config'), config], 180)
    run(['nix', 'store', 'sign', '--offline', '--key-file', str(root / 'remote-sign-key'), config], 180)
    run(['nix', 'store', 'verify', '--offline', '--sigs-needed', '1', '--option', 'trusted-public-keys', public, config], 180)
    run(['nix', 'copy', '--offline', '--to', f'file://{root}/current-signed-cache?compression=zstd&secret-key={root}/remote-sign-key', config], 300)
    pull = run([cli, '--nix-compat', '--store', str(root / 'client-store'), '--state-dir', str(root / 'client-state'), 'store', 'pull', '--from', str(root / 'current-signed-cache'), '--trusted-public-keys', trusted, Path(config).name], 600)
    (root / 'variant-native-config-pull.log').write_text(pull)
    if f'PULL {Path(config).name}' not in pull or not re.search(r'imported=1 skipped_present=0 skipped_untrusted=0 skipped_hash_mismatch=0 skipped_missing_nar=0 skipped_parse_error=0 nar_bytes=[1-9][0-9]*', pull):
        raise SystemExit('variant config admission requires nonzero selection and zero rejection counters')
    verify(Path(config).name, 'variant-native-config-verify.log')
    template = (root / 'variant-native-plan.template.ncl').read_text()
    if template.count('@VARIANT_CONFIG_STORE_PATH@') != 1:
        raise SystemExit('one variant config placeholder required')
    recipe = template.replace('@VARIANT_CONFIG_STORE_PATH@', config)
    (root / 'variant-native-plan.ncl').write_text(recipe)
    evaluated = run([cli, '--json', 'eval', '-I', '.', '-I', 'lib', str(root / 'variant-native-plan.ncl')], 180)
    (root / 'variant-native-plan-eval.json').write_text(evaluated)
    if config not in evaluated or provider not in evaluated:
        raise SystemExit('variant eval lost signed native provider/config binding')
    log.write(f'provider-only-variant-config-admitted=success provider={provider} config={config}; no graph execution or selectivity claim\n')
print(f'provider-only-variant-config-admitted=success provider={provider} config={config}; not selective rerun proof', flush=True)
