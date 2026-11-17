#!/usr/bin/env python3
"""Exercise the retained cold1714 mismatch without executing its recipe or changing pins."""
import copy
import json
import runpy
import subprocess
import sys
from pathlib import Path

if len(sys.argv) != 6:
    raise SystemExit('usage: check-primary-source-admission.py COHORT FROZEN_SOURCE SCRATCH SUBJECT_JSON INSPECTOR')
cohort, source, scratch, subject_path, inspector = map(Path, sys.argv[1:])
subject = json.loads(subject_path.read_bytes())

def digest(path):
    return subprocess.check_output([subject['inert_tools']['b3sum'], '--no-names', str(path)], text=True).strip()

graph_script = inspector.with_name('graph-audit.py')
sys.argv = [str(graph_script), str(cohort), str(source), str(scratch), str(subject_path)]
g = runpy.run_path(str(graph_script), run_name='retained_primary_source_graph_library')
a = runpy.run_path(str(inspector), run_name='retained_primary_source_inspector_library')
_, route = g['selected_route']()
recipes = {stage: json.loads(g['program_member_path'](member).read_bytes())
           for stage, member in zip(a['STAGES'], g['MANTLE_MEMBERS'], strict=True)}
plan_paths = {stage: g['program_member_path'](f'bootstrap/source-built/{stage}.unit-plan.json') for stage in a['STAGES']}
documents = {stage: json.loads(path.read_bytes()) for stage, path in plan_paths.items()}
facts, projections = a['unit_plan_facts'](documents, recipes)
assert not any(row.get('uncovered_count', len(row.get('uncovered', []))) for row in projections.values()), projections

def source_member(role, member):
    try:
        return g['checked_source_path'](g['source_role_root'](role), member).is_file()
    except (KeyError, OSError, ValueError):
        return False

result = a['audit_data'](recipes, facts, source_member)
bucket = result['coverage']['primary_sources']
assert result['bounded_verdict'] is False and bucket['invalid'] > 0 and bucket['uncovered'] == 0, bucket
stage = a['STAGES'][0]
steps = {step['id']: step for step in recipes[stage]['steps']}
index = next(index for index, fact in facts[stage].items()
             if fact['package_id'] == 'path+native#snix-build@0.1.0' and fact['target_kind'] == 'custom-build')
fact = facts[stage][index]
step = steps[f'unit-{index}-compile']
arguments, _, projection_error = a['compiler_static_arguments'](stage, step, steps)
assert projection_error is None
assert '..' in Path(fact['primary_path']).parts
assert not a['exact_primary_source_consumed'](arguments, step['inputs'], fact)
word = next(word for word in arguments if a['part'](word, 'input') and word['parts'][0]['path'].endswith('.rs'))
operand = word['parts'][0]
reference = step['inputs'][operand['name']]
actual_member = (Path(reference['path']) / operand['path']).as_posix()
actual_source = g['checked_source_path'](g['source_role_root'](reference['role']), actual_member)
assert (g['source_role_root'](fact['primary_role']) / fact['primary_path']).resolve(strict=True) == actual_source.resolve(strict=True)
failed_unit = {'ordinal': index, 'unit_id': fact['unit_id'], 'planned_primary_role': fact['primary_role'],
               'planned_primary_member_unmodified': fact['primary_path'], 'compiler_word': word,
               'compiler_input': reference, 'compiler_source_member': actual_member,
               'compiler_source_file_blake3': digest(actual_source), 'same_physical_file': True,
               'exact_admission_rejected': True}
positive_index = next(index for index, fact in facts[stage].items()
                      if fact['package_id'] == 'path+native#mantle@0.1.0' and fact['target_kind'] == 'bin')
positive_fact = facts[stage][positive_index]
positive_step = steps[f'unit-{positive_index}-compile']
positive_args, _, error = a['compiler_static_arguments'](stage, positive_step, steps)
assert error is None
check = a['exact_primary_source_consumed']
assert check(positive_args, positive_step['inputs'], positive_fact)
primary_index = next(i for i, word in enumerate(positive_args) if check([word], positive_step['inputs'], positive_fact))
primary_word = positive_args[primary_index]
cases = [{'case': 'actual_canonical_primary', 'accepted': True}]

def reject(label, replacement=None, inputs=None):
    arguments = copy.deepcopy(positive_args)
    if replacement is None:
        del arguments[primary_index]
    else:
        arguments[primary_index] = replacement
    assert not check(arguments, positive_step['inputs'] if inputs is None else inputs, positive_fact), label
    cases.append({'case': label, 'rejected': True})

reject('missing_primary_word')
reject('literal_instead_of_typed_input', {'parts': [{'kind': 'literal', 'text': positive_fact['primary_path']}]})
reject('concatenated_instead_of_single_input', {'parts': [*primary_word['parts'], {'kind': 'literal', 'text': ''}]})
foreign = copy.deepcopy(positive_step['inputs'])
foreign[primary_word['parts'][0]['name']]['role'] = 'vendor-inputs'
reject('foreign_source_role', primary_word, foreign)
wrong_member = copy.deepcopy(primary_word)
wrong_member['parts'][0]['path'] = 'src/lib.rs'
assert source_member('mantle-source', 'src/lib.rs') and positive_fact['primary_path'] != 'src/lib.rs'
reject('different_existing_source_member', wrong_member)
alias = 'primary-source-scoped-alias'
alias_inputs = dict(positive_step['inputs'])
expected = Path(positive_fact['primary_path'])
alias_inputs[alias] = {'kind': 'source', 'role': positive_fact['primary_role'], 'path': expected.parent.as_posix()}
alias_word = {'parts': [{'kind': 'input', 'name': alias, 'path': expected.name}]}
assert check([alias_word], alias_inputs, positive_fact)
cases.append({'case': 'declared_source_prefix_and_member_join', 'accepted': True})
record = {'scope': 'Inert retained cold1714 data and in-memory operand mutations only. No source/cohort/pin changes, recipe/product/helper/compiler/provider/controller execution, qualification or proof acceptance.',
          'source_revision': subject['frozen_product_revision'], 'subject_blake3': digest(subject_path),
          'inspector_blake3': digest(inspector),
          'source_plans': {stage: {'path': str(path), 'blake3': digest(path)} for stage, path in plan_paths.items()},
          'source_plan_projections': projections, 'retained_data_primary_source_coverage': bucket,
          'retained_data_per_stage_primary_sources': {stage: data['coverage']['primary_sources'] for stage, data in result['per_stage'].items()},
          'actual_failed_unit': failed_unit, 'predicate_cases': cases, 'all_assertions_passed': True}
output = scratch / 'primary-source-admission-regression.json'
with output.open('x') as stream:
    json.dump(record, stream, indent=2, sort_keys=True)
    stream.write('\n')
print(json.dumps({'all_assertions_passed': True, 'retained_data_primary_source_coverage': bucket,
                  'failed_unit': failed_unit['unit_id'], 'predicate_cases': len(cases), 'output': str(output)}, sort_keys=True))
