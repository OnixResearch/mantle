import ast
import copy
from pathlib import Path

root = Path('/tmp/mantle-unit-plan-20261013-87lqAtNv')
store = root / 'client-store'
source = (root / 'check-native-provider-selective-rerun.py').read_text()
module = ast.parse(source)
# Exercise only production helper definitions, not the incomplete final campaign.
selected = [node for node in module.body if isinstance(node, (ast.Import, ast.ImportFrom, ast.FunctionDef))]
ns = {'root': root, 'store': store}
exec(compile(ast.Module(body=selected, type_ignores=[]), '<actual-checker-functions>', 'exec'), ns)
json = ns['json']
original = json.loads((root / 'cfgfix-native-producer-report.json').read_text())
old_plan = json.loads((store / 'w42pw6bqpdv08k3nxkvsbmrq3r0nxmql-cargo-build-script-units-native-cfgfix-87lq-plan').read_text())
new_plan = json.loads((store / '0x1rw7kpynpz10vmf8qr1f76bgf2akq5-cargo-build-script-units-native-variant-87lq-plan').read_text())
old_units = {unit['id']: unit for unit in old_plan['units']}
new_units = {unit['id']: unit for unit in new_plan['units']}
old_root = '/nix/store/77giqf2dbsvv1sjcmydx9x366s5614fc-native-answer-0.1'
new_root = '/nix/store/jiwk6cxf281jlsxmlx80d02cmrw7cc1p-native-answer-0.2'
old_pair = ns['native_chain'](old_units, old_plan['roots'][0], old_root)
new_pair = ns['native_chain'](new_units, new_plan['roots'][0], new_root)
old_chain = (*old_pair, old_units[old_plan['roots'][0]])
new_chain = (*new_pair, new_units[new_plan['roots'][0]])
for version, chain in (('old', old_chain), ('new', new_chain)):
    for role, unit in zip(('native-script', 'native-sys', 'app'), chain):
        assert unit['derivation']['addressing_mode'] == 'input-addressed'
        assert unit['derivation']['fixed_output'] is None
        print('actual-fixture-addressing', version, role, unit['id'], 'input-addressed')
trace = (root / 'cfgfix-native-producer-trace.log').read_text()
producer = ns['goal_outcome'](original, ns.get('old_label', 'cargo-build-script-units-native-cfgfix-87lq'), 'cargo-build-script-units-native-cfgfix-87lq')
events = ns['unit_execution_events'](old_units, trace, producer['drv_key'])
assert len(events) == 34 and all(event['kind'] == 'built-success' for event in events.values())
for role, unit in zip(('native-script', 'native-sys', 'app'), old_chain):
    paths = ns['successful_unit_outputs'](unit, events[unit['id']], trace)
    print('actual-positive-output-binding', role, json.dumps(paths, sort_keys=True))
app_paths = ns['successful_unit_outputs'](old_chain[2], events[old_chain[2]['id']], trace)
print('actual-positive-app-report-binding', ns['bind_effective_app_report'](original, old_chain[2], app_paths))

negative_count = 0
def refused(label, function):
    global negative_count
    try:
        function()
    except SystemExit as error:
        negative_count += 1
        print('negative-refused', label, str(error))
    else:
        raise AssertionError('negative accepted: ' + label)

refused('unchanged-effective-paths', lambda: ns['affected_effective_changes'](old_chain, old_chain, events, events, trace, trace))
ca = copy.deepcopy(old_chain[0])
ca['derivation']['addressing_mode'] = 'content-addressed'
refused('content-addressed-generalization', lambda: ns['successful_unit_outputs'](ca, events[ca['id']], trace))
lines = trace.splitlines()
position = events[old_chain[0]['id']]['line'] - 1
missing_outputs = lines.copy()
missing_outputs[position] = ns['re'].sub(r'\boutputs=.*$', '', ns['re'].sub(r'\x1b\[[0-?]*[ -/]*[@-~]', '', missing_outputs[position]))
assert 'outputs=' not in missing_outputs[position]
refused('missing-effective-output-evidence', lambda: ns['successful_unit_outputs'](old_chain[0], events[old_chain[0]['id']], '\n'.join(missing_outputs)))
wrong = copy.deepcopy(original)
app_row = ns['goal_outcome'](wrong, old_chain[2]['derivation']['name'] + '.drv', old_chain[2]['derivation']['name'])
app_row['outputs'][0]['artifact_attestation']['logical_path'] = '/nix/store/wrong'
refused('wrong-report-logical-binding', lambda: ns['bind_effective_app_report'](wrong, old_chain[2], app_paths))
refused('duplicate-execution-event', lambda: ns['unit_execution_events'](old_units, trace + '\n' + lines[position], producer['drv_key']))
refused('missing-execution-event', lambda: ns['unit_execution_events'](old_units, '\n'.join(lines[:position] + lines[position + 1:]), producer['drv_key']))
refused('build-is-not-positive-cache-evidence', lambda: ns['assert_independent_reuse'](old_units, old_units, [u['id'] for u in old_chain], [u['id'] for u in old_chain], events))
assert negative_count == 7
print('checker-focused-smoke=PASS actual-original-builds=34 actual-positive-output-bindings=3 actual-app-report-binding=1 negative-refusals=7; no synthetic execution proof; final whole campaign comparison not invoked')
