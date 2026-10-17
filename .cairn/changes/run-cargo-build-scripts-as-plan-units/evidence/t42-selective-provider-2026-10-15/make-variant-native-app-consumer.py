#!/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3.14
import json
from pathlib import Path

root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
report_text = (root / 'variant-native-app.log').read_text()
if not report_text.startswith('variant-app-native-root-source=') or 'no-substitute=true' not in report_text.split('\n', 1)[0]:
    raise SystemExit('variant producer report lacks signed sandbox invocation')
report = json.loads(report_text.split('\n', 1)[1])
counts = report['counts']
if counts['failed_total'] or counts['succeeded_total'] < 1:
    raise SystemExit(f'variant producer-only graph failed before app root can be selected: {counts}')
source = (root / 'variant-native-plan.ncl').read_text()
old = (root / 'cfgfix-native-plan.ncl').read_text()
if old == source or '/nix/store/' not in source:
    raise SystemExit('variant native Nickel plan did not change the admitted provider root')
producer_goal = source.split('goal_name = "', 1)[1].split('"', 1)[0]
producers = [row for row in report['outcomes'] if row['label'] == producer_goal]
if len(producers) != 1:
    raise SystemExit(f'expected one variant signed RustPlan producer outcome, found {len(producers)}')
plans = [row['path'] for row in producers[0]['outputs'] if row['name'] == 'plan']
if len(plans) != 1:
    raise SystemExit('variant producer did not publish one accepted dynamic plan')
plan_path = Path(plans[0])
if plan_path.parent != root / 'client-store' or plan_path.is_symlink() or not plan_path.is_file():
    raise SystemExit(f'variant plan is not an admitted physical client output: {plan_path}')
plan = json.loads(plan_path.read_text())
if plan['schema'] != 'mantle-plan-v2':
    raise SystemExit(f'unexpected variant signed-client plan schema: {plan["schema"]}')

def is_fixture_binary(unit):
    args = unit['derivation']['args']
    return ('--crate-name' in args and args[args.index('--crate-name') + 1] == 'mantle_cargo_build_script_fixture'
            and unit['derivation']['name'].startswith('rust-unit-')
            and unit['id'] in plan['roots'])

apps = [unit for unit in plan['units'] if is_fixture_binary(unit)]
if len(apps) != 1:
    raise SystemExit(f'expected exactly one selected variant fixture app binary root, found {len(apps)}')
app_id = apps[0]['id']
head = 'let cargo_unit_plan = import "lib/cargo_unit_plan.ncl" in\ncargo_unit_plan {'
if not source.startswith(head) or not source.rstrip().endswith('}') or source.count('cargo_unit_plan {') != 1:
    raise SystemExit('variant signed config plan differs from producer-only Nickel template')
consumer_root = '/nix/store/pv1yr8fbzs88rpyc61xycwjlx9dakmp2-mantle-unit-plan-app-check'
bash_root = '/nix/store/sfvyavxai6qvzmv9p9x6mp4wwdz4v41m-bash-interactive-5.3p9'
static_root = '/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0'
consumer = root / 'client-store' / consumer_root.removeprefix('/nix/store/')
if consumer.is_symlink() or not consumer.is_file() or 'native-sys = 42' not in consumer.read_text():
    raise SystemExit('signed physical variant app-check script missing exact answer-42 assertion')
if not (root / 'client-store' / bash_root.removeprefix('/nix/store/') / 'bin/bash').is_file():
    raise SystemExit('signed Bash interpreter required by variant app-check script is not admitted physically')
if not (root / 'client-store' / static_root.removeprefix('/nix/store/') / 'bin/mkdir').exists():
    raise SystemExit('signed static BusyBox mkdir required by variant app-check script is not admitted physically')
consumer_plan = source.replace(head, 'let cargo_unit_plan = import "lib/cargo_unit_plan.ncl" in\nlet mantle = import "lib/lib.ncl" in\nlet plan_producer = cargo_unit_plan {', 1).rstrip()
consumer_plan += f''' in
{{
  producer = plan_producer,
  app = {{
    name = "cargo-build-script-units-app-check-variant-87lq",
    builder = "{consumer_root}",
    addressing_mode = 'input-addressed,
    sandbox = 'native,
    inputs = [
      "{consumer_root}",
      "{bash_root}",
      "{static_root}",
      {{
        name = "app",
        producer = plan_producer,
        plan_output = "plan",
        root = "{app_id}",
        unit_output = "out",
      }},
    ],
    env = {{
      MANTLE_UNIT_APP = "{{{{mantle-plan-output:app}}}}",
      PATH = "{static_root}/bin",
    }},
  }} | mantle.Derivation,
}}
'''
if any((root / name).exists() for name in ('variant-native-app-consumer.ncl', 'variant-native-app-unit-id.txt')):
    raise SystemExit('variant consumer already exists; do not overwrite')
(root / 'variant-native-app-consumer.ncl').write_text(consumer_plan)
(root / 'variant-native-app-unit-id.txt').write_text(app_id + '\n')
print(f'accepted-variant-app-unit={app_id}; signed-consumer={consumer_root}; signed-interpreter={bash_root}; signed-command-PATH={static_root}/bin; derived-after-producer={plan_path}', flush=True)
