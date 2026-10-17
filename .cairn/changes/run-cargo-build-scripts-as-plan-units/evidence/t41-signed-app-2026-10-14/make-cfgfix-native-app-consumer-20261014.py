#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
import json
import re
import subprocess
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
store = root / 'client-store'
trace = (root / 'cfgfix-native-producer-trace.log').read_text()
if not trace.startswith('phase=cfgfix-producer-start ') or 'no-substitute=true' not in trace.splitlines()[0]:
    raise SystemExit('lean producer lacks signed no-substitute invocation')
if 'phase=cfgfix-producer-done ' not in trace or 'timeout: sending signal TERM' in trace:
    raise SystemExit('lean producer timed out or did not complete')
report = json.loads((root / 'cfgfix-native-producer-report.json').read_text())
counts = report['counts']
if counts['failed_total'] or counts['succeeded_total'] < 1 or counts['built_total'] < 1:
    raise SystemExit(f'lean producer did not build a dynamic plan: {counts}')
label = 'cargo-build-script-units-native-cfgfix-87lq'
rows = [row for row in report['outcomes'] if row['label'] == label]
if len(rows) != 1 or rows[0]['cached']:
    raise SystemExit('lean native producer must have one uncached same-client outcome')
outputs = {item['name']: Path(item['path']) for item in rows[0]['outputs']}
if set(outputs) != {'out', 'plan', 'sources'}:
    raise SystemExit(f'lean producer output set incomplete: {outputs}')
for path in outputs.values():
    if path.parent != store or path.is_symlink() or not path.exists():
        raise SystemExit(f'lean producer output is not physical same-client path: {path}')
trusted = 'mantle-unit-plan-local-1:jlpkELaOmQsXYXwNvRQxPfzO1fqmMfJ4yxpj4hbt2Kw=,mantle-unit-plan-remote-87lq:qXjNX6CsTA+vCkoh+nZDI3psVEfCLnvLhV4ZhfiDnt8='
receipt_path = root / 'cfgfix-native-producer-verify.log'
if receipt_path.exists():
    retained = receipt_path.read_text()
    for path in outputs.values():
        if not re.search(rf'^OK {re.escape(path.name)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*$', retained, re.M):
            raise SystemExit(f'retained trusted producer receipt lacks exact output: {path.name}')
    if len(re.findall(r'^1 checked, 0 mismatches$', retained, re.M)) != len(outputs):
        raise SystemExit('retained producer physical receipt lacks all zero-mismatch checks')
else:
    verified = []
    for path in outputs.values():
        command = [str(root / 'target/debug/mantle'), '--nix-compat', '--store', str(store),
                   '--state-dir', str(root / 'client-state'), 'store', 'verify', path.name, '--trusted-public-keys', trusted]
        result = subprocess.run(command, capture_output=True, text=True, timeout=600)
        text = result.stdout + result.stderr
        verified.append(text)
        if result.returncode or not re.search(rf'OK {re.escape(path.name)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*', text) or '1 checked, 0 mismatches' not in text:
            raise SystemExit(f'lean producer output failed trusted physical verification: {text}')
    (root / 'cfgfix-native-producer-verify.log').write_text(''.join(verified))
plan = json.loads(outputs['plan'].read_text())
old_toolchain = '/nix/store/33vf8n9jip02hlkx1i96pkkgbg3aqc1z-mantle-unit-plan-nightly'
lean = (root / 'lean-nightly-path.txt').read_text().strip()
if plan['schema'] != 'mantle-plan-v2' or len(plan['units']) != 34 or len(plan['sources']) != 14 or len(plan['roots']) != 1:
    raise SystemExit('lean producer did not emit exact 34-unit/14-slice app graph')
if old_toolchain in outputs['plan'].read_text() or lean not in outputs['plan'].read_text():
    raise SystemExit('lean plan retained original toolchain or lost signed lean toolchain')
new_helper = (root / 'cfgfix-helper-path.txt').read_text().strip()
old_helper = '/nix/store/vqssk3gwk0cjkcqyzv40ay96iflxfl9l-mantle-cargo-unit-helper-current-87lq'
if new_helper not in outputs['plan'].read_text() or old_helper in outputs['plan'].read_text():
    raise SystemExit('corrected plan lost signed helper or retained old helper')
selected = [unit for unit in plan['units'] if unit['id'] == plan['roots'][0]]
if len(selected) != 1 or '--crate-name' not in selected[0]['derivation']['args']:
    raise SystemExit('selected lean app unit is missing or malformed')
args = selected[0]['derivation']['args']
if args[args.index('--crate-name') + 1] != 'mantle_cargo_build_script_fixture':
    raise SystemExit('selected lean root is not the Rust fixture application')
plan_output = next(item for item in rows[0]['outputs'] if item['name'] == 'plan')
logical_plan = admitted_logical_output(report, plan_output)
plans = [row for row in report['native_dynamic_plans'] if row.get('plan_artifact_path') == logical_plan]
if len(plans) != 1 or plans[0]['rejection_reason'] is not None or plans[0]['scheduler_action'] != 'registered-roots':
    raise SystemExit('lean graph was not admitted as registered roots')
plan_ids = [unit['id'] for unit in plan['units']]
accepted_ids = plans[0]['accepted_unit_ids']
if len(plan_ids) != len(set(plan_ids)) or len(accepted_ids) != len(set(accepted_ids)) or set(accepted_ids) != set(plan_ids):
    raise SystemExit('accepted unit IDs do not exactly match the unique physical plan IDs')
if len(plans[0]['accepted_unit_ids']) != 34 or len(plans[0]['source_slices']) != 14:
    raise SystemExit('lean graph lacks 34 accepted units or 14 source slices')
for slice_record in plans[0]['source_slices']:
    if slice_record['declared_nar_blake3'] != slice_record['observed_nar_blake3'] or not slice_record['admitted_store_path']:
        raise SystemExit(f'lean source slice did not match declared digest: {slice_record}')
recipe = (root / 'cfgfix-native-plan.ncl').read_text()
head = 'let cargo_unit_plan = import "lib/cargo_unit_plan.ncl" in\ncargo_unit_plan {'
if not recipe.startswith(head) or not recipe.rstrip().endswith('}') or recipe.count('cargo_unit_plan {') != 1:
    raise SystemExit('signed lean producer recipe differs from reviewed shape')
verifier = '/nix/store/pv1yr8fbzs88rpyc61xycwjlx9dakmp2-mantle-unit-plan-app-check'
bash = '/nix/store/sfvyavxai6qvzmv9p9x6mp4wwdz4v41m-bash-interactive-5.3p9'
static = '/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0'
if not (store / verifier.removeprefix('/nix/store/')).is_file():
    raise SystemExit('signed app checker not physically present')
consumer = recipe.replace(head, 'let cargo_unit_plan = import "lib/cargo_unit_plan.ncl" in\nlet mantle = import "lib/lib.ncl" in\nlet plan_producer = cargo_unit_plan {', 1).rstrip()
consumer += f''' in
{{
  producer = plan_producer,
  app = {{
    name = "cargo-build-script-units-app-check-cfgfix-87lq",
    builder = "{verifier}",
    addressing_mode = 'input-addressed,
    sandbox = 'native,
    inputs = [
      "{verifier}",
      "{bash}",
      "{static}",
      {{
        name = "app",
        producer = plan_producer,
        plan_output = "plan",
        root = "{plan['roots'][0]}",
        unit_output = "out",
      }},
    ],
    env = {{
      MANTLE_UNIT_APP = "{{{{mantle-plan-output:app}}}}",
      PATH = "{static}/bin",
    }},
  }} | mantle.Derivation,
}}
'''
path = root / 'cfgfix-native-app-consumer.ncl'
if path.exists():
    raise SystemExit('lean app consumer already exists; do not overwrite')
path.write_text(consumer)
(root / 'cfgfix-native-app-unit-id.txt').write_text(plan['roots'][0] + '\n')
print(f'lean-producer-admitted=success app_unit={plan["roots"][0]} signed_roots=3 accepted_units=34 source_slices=14 app_consumer={path}', flush=True)
