#!/usr/bin/env python3
"""Retained-resource-campaign inspection; no VM rerun or replay promotion."""
import ast
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path('/home/brittonr/git/OnixResearch/mantle')
RUN = Path('/home/brittonr/.cargo-target/mantle-resource-chaos-20261023/run-boot')
HERE = Path(__file__).resolve().parent
OUT = HERE / 'observed'
OUT.mkdir(exist_ok=False)
commands = []

def save(path, data):
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + '\n')

def command(label, argv, expected=None, env_extra=None):
    environment = dict(os.environ)
    environment.update(env_extra or {})
    result = subprocess.run([str(x) for x in argv], cwd=ROOT, env=environment, capture_output=True)
    stdout, stderr = OUT / (label + '.stdout'), OUT / (label + '.stderr')
    stdout.write_bytes(result.stdout)
    stderr.write_bytes(result.stderr)
    row = {'label': label, 'argv': [str(x) for x in argv], 'cwd': str(ROOT),
           'environment_overrides': env_extra or {}, 'exit_code': result.returncode,
           'stdout': str(stdout.relative_to(ROOT)), 'stderr': str(stderr.relative_to(ROOT))}
    if expected is not None:
        row.update(expected_exit_code=expected, matches_expected=result.returncode == expected)
    commands.append(row)
    return result

def digest(path):
    result = subprocess.run(['b3sum', '--no-names', str(path)], cwd=ROOT, capture_output=True, text=True)
    return 'blake3:' + result.stdout.strip() if result.returncode == 0 else None

manifest = json.loads((RUN / 'commands.json').read_text())
# Execute the repository-owned declarations, not its CLI dispatch or any VM stage.
runner = ROOT / 'workloads/resource-policy/campaign.py'
module = ast.parse(runner.read_text(), filename=str(runner))
declarations = []
for node in module.body:
    if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'parser' for t in node.targets):
        break
    declarations.append(node)
namespace = {'__file__': str(runner), '__name__': 'retained_campaign_inspection'}
exec(compile(ast.Module(body=declarations, type_ignores=[]), str(runner), 'exec'), namespace)
validation = []
for name, changed in [('current', None), ('changed-digest', 'digest'), ('missing-artifact', 'missing')]:
    candidate = json.loads(json.dumps(manifest))
    path = RUN / 'commands.json'
    if changed:
        first = next(iter(candidate['identities']))
        if changed == 'digest':
            candidate['identities'][first] = 'blake3:' + '0' * 64
        else:
            candidate['identities'][str(OUT / 'absent-artifact')] = candidate['identities'].pop(first)
        path = OUT / (name + '-commands.json')
        save(path, candidate)
    try:
        namespace['load_manifest'](path)
        validation.append({'case': name, 'manifest': str(path), 'accepted': True})
    except RuntimeError as error:
        validation.append({'case': name, 'manifest': str(path), 'accepted': False, 'error': str(error)})

bindings = [{'path': path, 'expected': identity, 'actual': digest(path)} for path, identity in manifest['identities'].items()]
for binding in bindings:
    binding['matches'] = binding['actual'] == binding['expected']
receipt = json.loads((RUN / 'local/receipt.json').read_text())
positive = json.loads((RUN / 'local/positive/report.json').read_text())
expected_messages = {row['message'] for row in positive['assertion_coverage']}
receipt_validation = {
    'manifest_binding_matches': receipt['commands_manifest_identity'] == digest(RUN / 'commands.json'),
    'positive_report_binding_matches': receipt['positive_report_identity'] == digest(RUN / 'local/positive/report.json'),
    'evidence_class': receipt['evidence_class'], 'replay_evidence': receipt['replay_evidence']}

# Existing acceptance predicates are inspected without invoking vm(), replacing
# subprocesses, or emitting the runner's success-only execution-classification.
seeds = []
artifact_bindings = []
for seed in manifest['config']['seeds']:
    directory = RUN / f'seed-{seed}'
    summary = json.loads((directory / 'assertions.json').read_text())
    checkpoint = json.loads((directory / 'checkpoint.json').read_text())
    trace = [json.loads(line) for line in (directory / 'frontier-trace.jsonl').read_text().splitlines()]
    observed = {row['message'] for row in summary['assertions'] if row['hit_count'] > 0 and row['verdict'] == 'passed'}
    assertions = [{key: row[key] for key in ['id', 'message', 'kind', 'guest', 'category', 'verdict', 'hit_count', 'true_count', 'false_count']}
                  | {'identity_fingerprint': row['identity']['fingerprint'], 'source_site': row['identity']['descriptor']['logical_key']}
                  for row in summary['assertions']]
    predicates = {'authoritative_v2': summary['schema'] == 'chaoscontrol.assertion-summary.v2' and summary['collision_safe_evidence'],
                  'all_local_messages_observed_passing': expected_messages.issubset(observed),
                  'every_catalog_verdict_passed': all(row['verdict'] == 'passed' for row in summary['assertions']),
                  'no_standalone_bug_files': not list(directory.glob('bug_*.json'))}
    export = command(f'export-no-candidate-{seed}', [manifest['tools'] + '/bin/chaoscontrol-explore', 'export-bugs',
                     '--checkpoint', directory / 'checkpoint.json', '--output', OUT / f'export-{seed}',
                     '--min-replay-parent-depth', '1', '--max-bugs', '1', '--no-overwrite'], expected=0)
    exports = list((OUT / f'export-{seed}').glob('bug_*.json'))
    seeds.append({'seed': seed, 'exact_vm_argv': manifest['vm_commands'][manifest['config']['seeds'].index(seed)],
                  'catalog_status': summary['catalog_status'], 'predicates': predicates,
                  'acceptance_predicates_pass': all(predicates.values()), 'assertions': assertions,
                  'missing_local_messages': sorted(expected_messages - observed),
                  'rounds_completed': checkpoint['rounds_completed'], 'branches_explored': checkpoint['total_branches_run'],
                  'checkpoint_bugs': checkpoint['bugs'], 'checkpoint_journal_binding': checkpoint['campaign_journal'],
                  'frontier_stop': trace[-1], 'exported_bug_files': [str(x) for x in exports]})
    for path in sorted(directory.rglob('*')):
        if path.is_file():
            artifact_bindings.append({'path': str(path), 'identity': digest(path), 'bytes': path.stat().st_size})

# Re-exercise the actual immutable guest and repository-owned SDK commands.
# These are local instrumentation runs, not product builds or test suites.
local_results = []
for name in ['positive', *namespace['NEGATIVES']]:
    directory = OUT / name
    directory.mkdir()
    argv = [manifest['guest']]
    if name != 'positive':
        argv += ['--negative-control', name]
    run = command(name + '-guest', argv, expected=0 if name == 'positive' else 1,
                  env_extra={'CHAOSCONTROL_SDK_LOCAL_OUTPUT': str(directory / 'sdk.jsonl')})
    summary = command(name + '-summary', [manifest['tools'] + '/bin/summarize-sdk-local-output', '--input', directory / 'sdk.jsonl',
                                         '--output', directory / 'report.json'], expected=0)
    quality = command(name + '-quality', [manifest['tools'] + '/bin/check-sdk-assertion-quality', '--input', directory / 'report.json'],
                      expected=0 if name == 'positive' else 1)
    report = json.loads((directory / 'report.json').read_text())
    failing = [{'message': row['message'], 'failures': row['failure_count']} for row in report['assertion_coverage'] if row['failure_count'] > 0]
    local_results.append({'case': name, 'guest_exit': run.returncode, 'summary_exit': summary.returncode,
                          'quality_exit': quality.returncode, 'failing_assertions': failing,
                          'expected_failure_observed': name == 'positive' or any(row['message'] == namespace['NEGATIVES'][name] for row in failing)})
for index, args in enumerate([['--negative-control'], ['--negative-control', 'unknown'], ['--unknown'], ['--negative-control', 'x' * 129]]):
    command(f'rejected-input-{index}', [manifest['guest'], *args], expected=2,
            env_extra={'CHAOSCONTROL_SDK_LOCAL_OUTPUT': str(OUT / f'rejected-{index}.jsonl')})

result = {'schema': 'mantle-resource-campaign-retained-inspection-v1', 'observed_unix_s': time.time(),
          'invocation': ['python3', str(Path(__file__).resolve())], 'cwd': str(ROOT),
          'manifest': str(RUN / 'commands.json'), 'manifest_identity': digest(RUN / 'commands.json'),
          'repository_owned_validator': {'source': str(runner), 'identity': digest(runner), 'function': 'load_manifest',
                                         'execution': 'AST declarations only, preserving __file__; no CLI/VM dispatch', 'cases': validation},
          'source_and_artifact_bindings': bindings, 'retained_receipt_validation': receipt_validation,
          'retained_boot_result': json.loads((RUN.parent / 'boot-result.json').read_text()),
          'seed_acceptance_predicates_source': 'workloads/resource-policy/campaign.py:147-177',
          'seeds': seeds, 'artifact_bindings': artifact_bindings, 'fresh_local_results': local_results,
          'commands': commands, 'execution_classification_exists': (RUN / 'execution-classification.json').exists(),
          'replay_evidence': False,
          'remaining_blocker': 'Seed 64003 exited 3 at assertion exercise floor; 18 named workload sites unexercised, including all progress and accounting/crash/CAS outcomes. No selected observed bug exists for replay/minimization.',
          'non_claims': manifest['config']['non_claims']}
save(OUT / 'inspection.json', result)
print(json.dumps({'inspection': str(OUT / 'inspection.json'), 'bindings_match': all(x['matches'] for x in bindings),
                  'seed_acceptance': {x['seed']: x['acceptance_predicates_pass'] for x in seeds},
                  'fresh_local_results': local_results, 'unexpected_command_exits': [x for x in commands if x.get('matches_expected') is False]}, indent=2))
if not all(x['matches'] for x in bindings) or any(x.get('matches_expected') is False for x in commands):
    raise SystemExit(1)
