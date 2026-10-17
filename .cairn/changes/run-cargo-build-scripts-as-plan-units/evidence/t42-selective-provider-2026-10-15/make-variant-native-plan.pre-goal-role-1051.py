#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
"""Prepare a provider-only variant of the accepted cfgfix producer recipe."""
import json
import re
import sys
from pathlib import Path

root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
store = root / 'client-store'
old_label = 'cargo-build-script-units-native-cfgfix-87lq'
new_label = 'cargo-build-script-units-native-variant-87lq'
variant_root = sys.argv[1] if len(sys.argv) == 2 else ''
if not re.fullmatch(r'/nix/store/[0-9a-df-np-sv-z]{32}-native-answer-0\.2', variant_root):
    raise SystemExit(f'expected one exact 0.2 native provider root: {variant_root}')


def receipt(name, logical):
    text = (root / name).read_text()
    basename = Path(logical).name
    if not re.search(rf'^OK {re.escape(basename)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*$', text, re.M) or '1 checked, 0 mismatches' not in text:
        raise SystemExit(f'missing trusted physical verification for {logical}: {name}')


def physical(logical, suffix):
    if not re.fullmatch(r'/nix/store/[0-9a-df-np-sv-z]{32}-[^/]+', logical):
        raise SystemExit(f'not a store root: {logical}')
    path = store / Path(logical).name
    if path.is_symlink() or not (path / suffix).is_file():
        raise SystemExit(f'provider absent from same client: {path / suffix}')


# Nothing is generated until the producer and the actual 0.1 application have been accepted.
for name in ('cfgfix-native-config.json', 'cfgfix-native-plan.ncl',
             'cfgfix-native-producer-report.json', 'cfgfix-native-producer-trace.log',
             'cfgfix-native-app-report.json', 'cfgfix-native-app-trace.log',
             'cfgfix-native-producer-verify.log', 'cfgfix-native-app-verify.log'):
    if not (root / name).is_file() or not (root / name).stat().st_size:
        raise SystemExit(f'accepted cfgfix baseline prerequisite missing: {name}')
if 'phase=cfgfix-producer-done ' not in (root / 'cfgfix-native-producer-trace.log').read_text() or 'phase=cfgfix-app-done ' not in (root / 'cfgfix-native-app-trace.log').read_text():
    raise SystemExit('cfgfix producer and app must finish before preparing variant')
config = json.loads((root / 'cfgfix-native-config.json').read_text())
if len(config['native_inputs']) != 1:
    raise SystemExit('cfgfix baseline has not exactly one native binding')
entry = next(iter(config['native_inputs'].values()))
if entry['env'] != {'NATIVE_ANSWER_ROOT': '{{role:answer}}'} or set(entry['inputs']) != {'answer'}:
    raise SystemExit('cfgfix native role differs from accepted fixture')
old_root = entry['inputs']['answer']
if not re.fullmatch(r'/nix/store/[0-9a-df-np-sv-z]{32}-native-answer-0\.1', old_root):
    raise SystemExit(f'cfgfix provider root malformed: {old_root}')
for logical, name in ((old_root, 'current-native-c-provider-verify.log'), (variant_root, 'variant-native-provider-verify.log')):
    physical(logical, 'lib/libnative_answer.a')
    receipt(name, logical)
producer = json.loads((root / 'cfgfix-native-producer-report.json').read_text())
app = json.loads((root / 'cfgfix-native-app-report.json').read_text())
if producer['counts']['failed_total'] or app['counts']['failed_total'] or app['counts']['succeeded_total'] < 2:
    raise SystemExit('cfgfix baseline producer or application failed')
producers = [row for row in producer['outcomes'] if row['label'] == old_label]
apps = [row for row in app['outcomes'] if row['label'] == 'cargo-build-script-units-app-check-cfgfix-87lq']
if len(producers) != 1 or len(apps) != 1 or apps[0]['cached']:
    raise SystemExit('cfgfix producer or actual uncached 42 application outcome missing')
plans = [Path(out['path']) for out in producers[0]['outputs'] if out['name'] == 'plan']
if len(plans) != 1 or plans[0].parent != store or plans[0].is_symlink() or not plans[0].is_file():
    raise SystemExit('accepted cfgfix plan file missing from same client')
receipt('cfgfix-native-producer-verify.log', plans[0].name)
plan = json.loads(plans[0].read_text())
if plan['schema'] != 'mantle-plan-v2' or len(plan['roots']) != 1 or len(plan['units']) != 34 or len(plan['sources']) != 14:
    raise SystemExit('cfgfix producer plan not the accepted fixture graph')
selected = [unit for unit in plan['units'] if unit['id'] == plan['roots'][0]]
if len(selected) != 1 or selected[0]['derivation']['args'][:2] != ['--crate-name', 'mantle_cargo_build_script_fixture']:
    raise SystemExit('cfgfix plan root is not the Rust fixture application')
if (root / 'cfgfix-native-app-unit-id.txt').read_text().strip() != plan['roots'][0]:
    raise SystemExit('cfgfix consumer selected a different root')
app_outputs = [Path(out['path']) for out in apps[0]['outputs'] if out['name'] == 'out']
if len(app_outputs) != 1 or app_outputs[0].parent != store or app_outputs[0].is_symlink() or (app_outputs[0] / 'result.txt').read_text() != 'cargo-build-script-fixture: proc-macro2 serde-derive libc native-sys = 42\n':
    raise SystemExit('cfgfix 42 application output missing')
receipt('cfgfix-native-app-verify.log', app_outputs[0].name)
recipe = (root / 'cfgfix-native-plan.ncl').read_text()
config_refs = re.findall(r'(?m)^  config = "([^"]+)",$', recipe)
if len(config_refs) != 1 or not config_refs[0].endswith('-mantle-unit-plan-config-cfgfix-87lq'):
    raise SystemExit('cfgfix recipe config reference absent or ambiguous')
receipt('cfgfix-native-config-verify.log', config_refs[0])
if recipe.count(old_root) != 1 or recipe.count(old_label) != 1 or recipe.count(config_refs[0]) != 1:
    raise SystemExit('accepted cfgfix recipe references not unique')
if any((root / name).exists() for name in ('variant-native-config.json', 'variant-native-plan.template.ncl')):
    raise SystemExit('variant config/template already exists; do not overwrite')
variant = json.loads(json.dumps(config))
next(iter(variant['native_inputs'].values()))['inputs']['answer'] = variant_root
variant_recipe = recipe.replace(old_root, variant_root).replace(old_label, new_label).replace(config_refs[0], '@VARIANT_CONFIG_STORE_PATH@')
if recipe == variant_recipe or variant_recipe.count('@VARIANT_CONFIG_STORE_PATH@') != 1:
    raise SystemExit('variant recipe did not change only the intended references')
(root / 'variant-native-config.json').write_text(json.dumps(variant, sort_keys=True, separators=(',', ':')) + '\n')
(root / 'variant-native-plan.template.ncl').write_text(variant_recipe)
print(f'variant-plan-prepared cfgfix_app={plan["roots"][0]} old={old_root} new={variant_root} changed_only=provider_root config_store_root producer_label; no build or selective proof', flush=True)
