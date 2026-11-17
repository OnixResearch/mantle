#!/usr/bin/env python3
"""Inert, stage-local helper-source authority and producer-linkage inspection."""
import collections
import json
import re
import runpy
import sys
from pathlib import Path

STAGES = ('stagex-transition', 'stagex-provider-publication', 'full-source-native-provider',
          'full-source-rust-provider', 'mantle-stage1', 'mantle-stage2')
RECIPE_KEYS = ('stagex_transition_recipe', 'publication_recipe', 'native_recipe',
               'rust_recipe', 'mantle_stage1', 'mantle_stage2')
AUTHOR_FILES = {
    'stagex-transition': ('src/source_built_stagex_programs.rs', 'PROGRAM_RECORD'),
    'stagex-provider-publication': ('src/source_built_stagex_programs.rs', 'PROGRAM_RECORD'),
    'full-source-native-provider': ('src/source_built_native_programs.rs', 'RECORD_IDENTITY'),
    'full-source-rust-provider': ('src/source_built_rust_programs.rs', 'RECORD_IDENTITY'),
    'mantle-stage1': ('src/source_built_mantle_programs.rs', 'PROGRAM_RECORD'),
    'mantle-stage2': ('src/source_built_mantle_programs.rs', 'PROGRAM_RECORD'),
}
RULE_FILES = ('src/source_built_recipe.rs', 'src/source_built_recipe_exec.rs',
              'src/source_bundle.rs', 'src/source_built_native_programs.rs',
              'src/source_built_native_programs_early_gnu.rs',
              'src/source_built_native_programs_host.rs',
              'src/source_built_native_programs_phases.rs',
              'src/source_built_rust_programs.rs',
              'src/source_built_rust_programs_translation.rs',
              'src/source_built_stagex_programs.rs',
              'src/source_built_stagex_programs_binutils.rs',
              'src/source_built_stagex_programs_publication.rs',
              'src/source_built_mantle_programs.rs',
              'src/source_built_mantle_programs_graph.rs')
MAX_DIAGNOSTICS = 100


def _key(ref):
    return json.dumps(ref, sort_keys=True, separators=(',', ':'))


def _parts(words):
    for word in words:
        if isinstance(word, dict):
            yield from word.get('parts', ())


def _words(step, steps):
    op = step['operation']
    if op['kind'] != 'exec':
        return [], True
    words = list(op.get('arguments', ())) + list(op.get('environment', {}).values())
    request = op.get('invocation_input')
    if request:
        ref = step.get('inputs', {}).get(request, {})
        producer = steps.get(ref.get('action_id')) if ref.get('kind') == 'produced' else None
        if not producer or producer['operation']['kind'] != 'write-text':
            return words, False
        try:
            invocation = json.loads(producer['operation']['text'])
            words.extend(invocation['arguments'])
            words.extend(invocation['environment'].values())
            words.extend(invocation[name] for name in ('cwd', 'argv0', 'executable_path')
                         if invocation.get(name) is not None)
        except (KeyError, TypeError, ValueError):
            return words, False
    return words, True


def _typed_names(step, steps):
    words, request_static = _words(step, steps)
    names = {p.get('name') for p in _parts(words)
             if isinstance(p, dict) and p.get('kind') in ('input', 'text_input', 'text_slice')}
    return names - {None}, request_static


def _source_origins(ref, steps, seen=None):
    """Trace a typed source operand back through inert assembled/copied bytes."""
    if not isinstance(ref, dict):
        return [], False
    if ref.get('kind') in ('manifest-source', 'source'):
        return [ref], True
    if ref.get('kind') != 'produced':
        return [], False
    seen = seen or set()
    token = _key(ref)
    if token in seen:
        return [], False
    seen = seen | {token}
    step = steps.get(ref.get('action_id'))
    if step is None:
        return [], False
    op = step['operation']
    if op['kind'] not in ('assemble-tree', 'copy-text'):
        return [], False
    names = op.get('entries', {}).values() if op['kind'] == 'assemble-tree' else (op.get('input'),)
    origins = []
    certain = True
    for name in names:
        sub, ok = _source_origins(step.get('inputs', {}).get(name), steps, seen)
        origins.extend(sub)
        certain &= ok
    return origins, certain and bool(origins)


def _dynamic_source_bridge(step, name, steps):
    """Prove a GCC phase's symbolic source from its typed specs request/map.

    The runtime GCC response remains unobserved; this establishes only the
    static original query -> splitter response -> phase operand contract.
    """
    match = re.fullmatch(r'(.+)\.phase-([0-9]+)', step['id'])
    if not match or step['operation'].get('invocation_input') != 'gcc-phase-response':
        return False
    prefix, ordinal = match.groups()
    stage = next(iter(steps)).split('/', 1)[0]
    query = steps.get(stage + '/' + prefix + '.specs')
    splitter = steps.get(stage + '/' + prefix + '.split')
    mapping = steps.get(stage + '/' + prefix + '.paths-' + ordinal)
    if not query or not splitter or not mapping or mapping['operation']['kind'] != 'write-text':
        return False
    typed, request_static = _typed_names(query, steps)
    if name not in typed or not request_static or query.get('inputs', {}).get(name) != step.get('inputs', {}).get(name):
        return False
    response = step.get('inputs', {}).get('gcc-phase-response')
    log = splitter.get('inputs', {}).get('gcc-phase-log')
    if response != {'kind': 'produced', 'action_id': stage + '/' + prefix + '.split',
                    'output_id': 'response-' + ordinal}:
        return False
    if log != {'kind': 'produced', 'action_id': stage + '/' + prefix + '.specs', 'output_id': 'stderr'}:
        return False
    return any(line.split('\t', 1)[0] == '/inputs/' + name
               for line in mapping['operation']['text'].splitlines())


def _compile_shape(step, typed):
    op = step['operation']
    if op['kind'] != 'exec':
        return False
    outputs = step.get('outputs', {})
    kinds = {x.get('kind') for x in outputs.values()}
    literals = [p.get('text') for p in _parts(op.get('arguments', ()))
                if isinstance(p, dict) and p.get('kind') == 'literal']
    if 'helper-source' in step.get('inputs', {}) or 'helper-source' in typed:
        return True
    if not kinds.intersection({'object', 'executable', 'tree'}):
        return False
    return (any(x in literals for x in ('-c', '-S', '--crate-type=bin', '-x'))
            or ('-o' in literals and bool(kinds.intersection({'executable', 'object'}))))


def _author_record(source, stage, subject):
    filename, constant = AUTHOR_FILES[stage]
    body = (source / filename).read_text()
    matches = re.findall(r'\bconst\s+' + constant + r'\s*:\s*&str\s*=\s*"([^"]+)"\s*;', body)
    if len(matches) != 1 or matches[0] not in subject['program_records']:
        raise ValueError(f'{stage}: frozen author record constant not uniquely admitted')
    return matches[0]


def _bound_member(api, cohort, ref, record, expected_digest):
    """Check actual manifest record, leaf and hydrated authored bytes, not a path suffix."""
    if ref.get('kind') != 'manifest-source':
        return {'error': 'source operand is not authenticated ManifestSource'}
    if ref.get('role') != 'native-source-bundle' or ref.get('record_identity') != record:
        return {'error': 'foreign source role/author record'}
    resolution = api['source_kind_resolution'](ref)
    if resolution.get('resolved_artifact_kind') != 'text':
        return {'error': resolution.get('unresolved_reason', 'helper source is not an exact regular member')}
    inventory = api['source_record_inventories'](ref['role']).get(record)
    if not inventory or inventory['content_blake3'] != expected_digest:
        return {'error': 'admitted author record content identity differs'}
    member = ref.get('member')
    entries = inventory['file_entries'].get(member, ())
    if len(entries) != 1 or entries[0].get('file_type') != 'regular' or entries[0].get('chunk_index') is not None:
        return {'error': 'authored helper is not a single exact regular manifest member'}
    try:
        path = api['checked_source_path'](cohort / 'program-sources' / record, member)
        if not path.is_file():
            raise ValueError('authored member is not a regular hydrated file')
        digest = api['b3'](path)
    except (OSError, ValueError) as exc:
        return {'error': f'authored member bytes unavailable: {exc}'}
    if digest != entries[0]['blake3']:
        return {'error': 'hydrated member bytes differ from authenticated manifest leaf',
                'manifest_leaf_blake3': entries[0]['blake3'], 'observed_blake3': digest}
    return {'record_content_blake3': inventory['content_blake3'],
            'manifest_leaf_blake3': entries[0]['blake3'], 'hydrated_member_blake3': digest,
            'member_path': str(path), 'member': member,
            'manifest_raw_blake3': api['SOURCE_MANIFEST_OBSERVATIONS'][ref['role']]['blake3']}


def _linkage(recipe, step, steps, references):
    stage = recipe['stage_id']
    query = step['id'].endswith('.specs')
    initial = [{'kind': 'produced', 'action_id': stage + '/' + step['id'], 'output_id': name}
               for name, declaration in step.get('outputs', {}).items()
               if declaration.get('kind') in ('object', 'executable', 'tree', 'text')
               and (name not in ('stdout', 'stderr', 'status') or query and name == 'stderr')]
    roots = {_key(ref) for ref in recipe.get('roots', [])}
    roots.update(_key(ref) for ref in recipe.get('exports', {}).values())
    roots.update(_key(ref) for values in recipe.get('semantic_roots', {}).values() for ref in values)
    frontier = [(_key(ref), [step['id']]) for ref in initial]
    seen = set()
    direct = []
    terminal = []
    while frontier:
        token, path = frontier.pop()
        if token in seen:
            continue
        seen.add(token)
        ref = json.loads(token)
        producer = steps.get(ref.get('action_id'))
        decl = producer.get('outputs', {}).get(ref.get('output_id')) if producer else None
        kind = decl.get('kind') if decl else None
        consumers = references.get(token, ())
        if len(path) == 1:
            direct.extend(x['id'] for x in consumers)
        if kind == 'executable' and (consumers or token in roots):
            terminal.append({'producer': ref, 'path': path,
                             'actual_consumers': [x['id'] for x in consumers[:12]],
                             'root_or_export': token in roots})
        for consumer in consumers:
            if consumer['id'] in path:
                continue
            next_path = path + [consumer['id']]
            for name, output in consumer.get('outputs', {}).items():
                if output.get('kind') not in ('object', 'executable', 'tree', 'text') or name in ('stdout', 'stderr', 'status'):
                    continue
                frontier.append((_key({'kind': 'produced', 'action_id': stage + '/' + consumer['id'],
                                       'output_id': name}), next_path))
    return {'compiled_outputs': initial, 'direct_consumers': sorted(set(direct)),
            'executable_producer_refs': sorted({_key(x['producer']) for x in terminal}),
            'executable_descendants': terminal[:12], 'executable_descendant_count': len(terminal)}


def _baseline_helpers(cohort, graph_api, records):
    """Expected source operands from the separately hash-bound selected recipes."""
    graph_path = graph_api['SCRATCH'] / 'graph-audit.json'
    graph = json.loads(graph_path.read_bytes())
    selected = graph['selected_input_blake3']
    expected = {}
    expected_links = {}
    bindings = {}
    for stage, key in zip(STAGES, RECIPE_KEYS):
        item = selected[key]
        path = Path(item['path'])
        if (not path.resolve().is_relative_to(cohort.resolve()) or
                path.stat().st_size != item['size_bytes'] or graph_api['b3'](path) != item['blake3']):
            raise ValueError(f'{stage}: baseline recipe is not the graph-bound selected bytes')
        recipe = json.loads(path.read_bytes())
        steps = {stage + '/' + step['id']: step for step in recipe['steps']}
        references = collections.defaultdict(list)
        for consumer in recipe['steps']:
            try:
                used = graph_api['used_refs'](consumer)
            except (KeyError, ValueError, TypeError):
                used = set()
            for ref in used:
                references[ref].append(consumer)
        for step in recipe['steps']:
            if step['operation']['kind'] != 'exec':
                continue
            typed, _ = _typed_names(step, steps)
            if not _compile_shape(step, typed):
                continue
            for name, ref in step.get('inputs', {}).items():
                origins, certain = _source_origins(ref, steps)
                if certain and len(origins) == 1 and origins[0].get('kind') == 'manifest-source' and origins[0].get('record_identity') == records[stage]:
                    expected[(stage, step['id'], name)] = origins[0]
                    expected_links[(stage, step['id'], name)] = _linkage(recipe, step, steps, references)
        bindings[stage] = item
        del recipe, steps
    return expected, expected_links, bindings


def audit_recipes(recipes, plan, subject, source, cohort, graph_api):
    """Callable inert data-level audit; recipes is a stage-id -> parsed actual recipe map.

    The caller may deepcopy/mutate recipe/plan data before invoking this function.
    graph_api is the runpy-loaded frozen graph inspector namespace with main uncalled.
    """
    source, cohort = Path(source), Path(cohort)
    if set(recipes) != set(STAGES) or [s.get('stage_id') for s in plan.get('stages', ())] != list(STAGES):
        raise ValueError('six selected recipes and exact plan stage order required')
    records = {stage: _author_record(source, stage, subject) for stage in STAGES}
    inventory = graph_api['source_record_inventories']('native-source-bundle')
    baseline, baseline_links, baseline_bindings = _baseline_helpers(cohort, graph_api, records)
    baseline_by_step = collections.defaultdict(dict)
    for (baseline_stage, step_id, name), ref in baseline.items():
        baseline_by_step[baseline_stage, step_id][name] = ref
    stages = {}
    for index, stage in enumerate(STAGES):
        recipe = recipes[stage]
        if recipe.get('stage_id') != stage or not isinstance(recipe.get('steps'), list):
            raise ValueError(f'{stage}: selected recipe stage or steps absent')
        declared = [entry.get('role') for entry in plan['stages'][index].get('inputs', ())
                    if entry.get('authority') == 'source']
        expected = records[stage]
        allowed = subject['program_records'][expected]
        steps = {stage + '/' + s['id']: s for s in recipe['steps']}
        if len(steps) != len(recipe['steps']):
            raise ValueError(f'{stage}: duplicate producer')
        references = collections.defaultdict(list)
        for consumer in recipe['steps']:
            try:
                used = graph_api['used_refs'](consumer)
            except (KeyError, ValueError, TypeError):
                used = set()
            for ref in used:
                references[ref].append(consumer)
        rows = []
        for step in recipe['steps']:
            op = step.get('operation', {})
            if op.get('kind') != 'exec':
                continue
            inputs = step.get('inputs', {})
            typed, static_request = _typed_names(step, steps)
            shaped = _compile_shape(step, typed)
            source_names = set()
            origins = {}
            for name, ref in inputs.items():
                refs, certain = _source_origins(ref, steps)
                origins[name] = (refs, certain)
                if any(r.get('kind') == 'manifest-source' and
                       (r.get('record_identity') in subject['program_records'] or
                        r.get('role') == 'native-source-bundle') for r in refs):
                    source_names.add(name)
            # The authored record, not a helper-looking filename, identifies
            # source candidates. Missing declarations at a typed compiler source
            # operand are separately uncovered. Original foreign source compiles
            # do not become this owning-stage helper authority.
            output_kinds = {declaration.get('kind') for declaration in step.get('outputs', {}).values()}
            candidate_names = (set(source_names) if shaped or output_kinds & {'object', 'executable'}
                               else source_names & {'helper-source'})
            candidate_names.update(baseline_by_step[stage, step['id']])
            if shaped:
                candidate_names |= ({'helper-source', 'source.c', 'unit.c'} & typed)
                if 'source' in typed and 'source' not in inputs:
                    candidate_names.add('source')
            for name in sorted(candidate_names):
                refs, certain = origins.get(name, ([], False))
                if not shaped and name != 'helper-source' and not output_kinds & {'object', 'executable'} and (stage, step['id'], name) not in baseline:
                    continue
                failures = []
                if not shaped:
                    failures.append('authored source passed to an unclassified Exec; compilation not established')
                bridged = _dynamic_source_bridge(step, name, steps) if name not in typed else False
                if name not in typed and not bridged:
                    failures.append('source input not actually consumed by a typed argument/environment operand')
                if not static_request and name not in typed and not bridged and op.get('invocation_input'):
                    failures.append('runtime-symbolic invocation cannot prove typed source use')
                if name not in inputs:
                    failures.append('typed source operand has no declared input')
                frozen_ref = baseline.get((stage, step['id'], name))
                if frozen_ref is None:
                    failures.append('unknown helper compilation candidate absent from frozen author-selected recipe')
                elif not certain or refs != [frozen_ref]:
                    failures.append('wrong source record/member relative to frozen author-selected operand')
                if not certain or len(refs) != 1:
                    failures.append('source origin is absent, ambiguous, dynamic, or not a pure authenticated member')
                if declared.count('native-source-bundle') != 1:
                    failures.append('exact owning-stage native-source-bundle declaration absent or duplicated')
                details = [_bound_member(graph_api, cohort, ref, expected, allowed) for ref in refs]
                if any('error' in detail for detail in details):
                    failures.append('manifest record/member or hydrated source bytes invalid')
                linkage = _linkage(recipe, step, steps, references)
                if not linkage['compiled_outputs'] or not linkage['direct_consumers'] and not linkage['executable_descendant_count']:
                    failures.append('helper compile output has no actual producer/consumer linkage')
                if not linkage['executable_descendant_count']:
                    failures.append('no consumed executable descended from this helper compilation')
                expected_linkage = baseline_links.get((stage, step['id'], name))
                if expected_linkage is not None and (
                        linkage['compiled_outputs'] != expected_linkage['compiled_outputs'] or
                        linkage['direct_consumers'] != expected_linkage['direct_consumers'] or
                        linkage['executable_producer_refs'] != expected_linkage['executable_producer_refs']):
                    failures.append('wrong helper output producer or actual consumer chain relative to frozen author')
                unknown = any('unclassified' in message or 'absent, ambiguous' in message or
                              'runtime-symbolic' in message or 'unknown helper compilation' in message
                              for message in failures)
                classification = 'uncovered' if unknown else 'invalid' if failures else 'covered'
                rows.append({'step_id': step['id'], 'source_operand': name,
                             'source_references': refs, 'origin_certain': certain,
                             'stage_author_record': expected, 'author_record_content_blake3': allowed,
                             'declared_source_roles': declared,
                             'frozen_expected_source_reference': frozen_ref,
                             'frozen_expected_producer_linkage': expected_linkage,
                             'typed_source_operand_consumed': name in typed or bridged,
                             'typed_source_proof': 'static GCC specs/split/map/phase bridge' if bridged
                             else 'direct typed word' if name in typed else None,
                             'manifest_member_bindings': details, 'linkage': linkage,
                             'failures': failures, 'classification': classification})
        seen = {(row['step_id'], row['source_operand']) for row in rows}
        for (candidate_stage, candidate_step), obligations in baseline_by_step.items():
            if candidate_stage != stage:
                continue
            for name, ref in obligations.items():
                if (candidate_step, name) not in seen:
                    rows.append({'step_id': candidate_step, 'source_operand': name,
                                 'frozen_expected_source_reference': ref,
                                 'source_references': [], 'manifest_member_bindings': [],
                                 'failures': ['frozen author-selected helper compilation obligation absent'],
                                 'classification': 'uncovered'})
        counts = collections.Counter(row['classification'] for row in rows)
        global_failures = []
        if not rows:
            global_failures.append('no owning-stage helper compilation candidates identified')
        if expected not in inventory or inventory.get(expected, {}).get('content_blake3') != allowed:
            global_failures.append('author record identity/content absent from actual admitted manifest')
        stages[stage] = {'author_record': expected, 'record_content_blake3': allowed,
                         'manifest_record_present': expected in inventory,
                         'total': len(rows), 'covered': counts['covered'],
                         'uncovered': counts['uncovered'], 'invalid': counts['invalid'],
                         'global_failures': global_failures,
                         'details': rows,
                         'failure_examples': [row for row in rows if row['failures']][:MAX_DIAGNOSTICS]}
    totals = {key: sum(row[key] for row in stages.values()) for key in ('total','covered','uncovered','invalid')}
    totals['global_failures'] = sum(len(row['global_failures']) for row in stages.values())
    totals['failure_count'] = totals['invalid'] + totals['uncovered'] + totals['global_failures']
    return {'stages': stages, 'totals': totals,
            'source_manifest_binding': graph_api['SOURCE_MANIFEST_OBSERVATIONS']['native-source-bundle'],
            'frozen_recipe_bindings': baseline_bindings,
            'baseline_expected_helper_compilations': len(baseline),
            'baseline_expected_helper_compilations_by_stage':
                {stage: sum(candidate_stage == stage for candidate_stage, _, _ in baseline) for stage in STAGES},
            'failure_examples': ([{'stage': stage, 'error': error}
                                  for stage, data in stages.items() for error in data['global_failures']]
                                 + [dict(stage=stage, **finding)
                                    for stage, data in stages.items()
                                    for finding in data['failure_examples']])[:MAX_DIAGNOSTICS],
            'bounded_verdict': 'CLEAR' if totals['invalid'] == 0 and totals['uncovered'] == 0
            and totals['global_failures'] == 0 and totals['covered'] == totals['total'] else 'HOLD',
            'scope': 'Static helper-authority and producer/consumer inspection only; no candidate bytes executed.'}


def main():
    if len(sys.argv) != 5:
        raise SystemExit('usage: helper-authority-audit.py COHORT FROZEN_SOURCE REVIEW_SCRATCH SUBJECT_JSON')
    cohort, source, scratch, subject_path = map(Path, sys.argv[1:])
    subject = json.loads(subject_path.read_bytes())
    graph_path = scratch / 'graph-audit.json'
    graph = json.loads(graph_path.read_bytes())
    # runpy imports the graph's inert reductions only. Its main is never called.
    api = runpy.run_path(str(Path(__file__).with_name('graph-audit.py')), run_name='helper_authority_library')
    b3 = api['b3']
    if subject != api['SUBJECT'] or cohort.resolve() != Path(subject['cohort']).resolve() or source.resolve() != Path(subject['frozen_source']).resolve():
        raise ValueError('four-argument frozen subject contract differs')
    if b3(subject_path) != graph['subject_binding']['subject_json_observed']['blake3']:
        raise ValueError('graph audit refers to another subject raw byte identity')
    if not all(graph['subject_binding']['observed_claim_checks'].values()):
        raise ValueError('graph audit has failed subject bindings')
    selected = graph['selected_input_blake3']
    required = dict(zip(STAGES, RECIPE_KEYS))
    paths = {}
    for stage, key in required.items():
        item = selected[key]
        path = Path(item['path'])
        if not path.is_file() or b3(path) != item['blake3'] or path.stat().st_size != item['size_bytes']:
            raise ValueError(f'{stage}: graph-selected recipe bytes differ')
        if not path.resolve().is_relative_to(cohort.resolve()):
            raise ValueError(f'{stage}: graph-selected recipe escapes prepared cohort')
        paths[stage] = path
    plan_item = selected['plan']
    plan_path = Path(plan_item['path'])
    if (plan_path.resolve() != Path(subject['input_plan']).resolve() or b3(plan_path) != plan_item['blake3']
            or plan_path.stat().st_size != plan_item['size_bytes']):
        raise ValueError('selected input plan differs from subject/graph')
    recipes = {stage: json.loads(path.read_bytes()) for stage, path in paths.items()}
    report = audit_recipes(recipes, json.loads(plan_path.read_bytes()), subject, source, cohort, api)
    report['input_bindings'] = {'subject_json': {'path': str(subject_path), 'blake3': b3(subject_path)},
                                'graph_audit': {'path': str(graph_path), 'blake3': b3(graph_path)},
                                'plan': plan_item, **{stage: selected[key] for stage, key in required.items()}}
    rule_paths = set(RULE_FILES)
    for family in ('source_built_native_programs', 'source_built_rust_programs',
                   'source_built_stagex_programs', 'source_built_mantle_programs'):
        rule_paths.update(path.relative_to(source).as_posix() for path in (source/'src').glob(family + '*.rs'))
    report['rule_bindings'] = {name: {'path': str(source/name), 'blake3': b3(source/name),
                                     'size_bytes': (source/name).stat().st_size}
                               for name in sorted(rule_paths)}
    report['inspector_binding'] = {'path': str(Path(__file__).resolve()), 'blake3': b3(Path(__file__)),
                                   'graph_inspector_path': str(Path(__file__).with_name('graph-audit.py')),
                                   'graph_inspector_blake3': b3(Path(__file__).with_name('graph-audit.py')),
                                   'hash_tool_path': api['B3'], 'hash_tool_blake3': b3(Path(api['B3'])),
                                   'interpreter_path': sys.executable, 'interpreter_blake3': b3(Path(sys.executable)),
                                   'argv': sys.argv[:]}
    output = scratch / 'helper-authority-audit.json'
    with output.open('x') as stream:
        json.dump(report, stream, indent=2, sort_keys=True)
        stream.write('\n')
    print(json.dumps({'bounded_verdict': report['bounded_verdict'], 'totals': report['totals'],
                      'output': str(output)}, sort_keys=True))


if __name__ == '__main__':
    main()
