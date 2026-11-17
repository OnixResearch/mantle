#!/usr/bin/env python3
"""Inert original-package and Cargo feature selector audit of the selected Mantle recipes.

The data-level function accepts separately source-derived unit facts. It does not infer
selected features from the very recipe selectors whose correctness is under review.
"""
import collections
import json
import posixpath
import re
import runpy
import subprocess
import sys
import tomllib
from pathlib import Path

STAGES = ('mantle-stage1', 'mantle-stage2')
UNIT = re.compile(r'^unit-(\d{5})-(compile|script|script-original-package)$')
SCRIPT_SCOPE = re.compile(r'^unit-(\d{5})-script(?:$|[-.])')
FEATURE = re.compile(r'^CARGO_FEATURE_[A-Z0-9_]+$')
FEATURE_NAME = re.compile(r'^[A-Za-z0-9_+.-]+$')
LIMIT = 80


def literal(word):
    parts = word.get('parts') if isinstance(word, dict) else None
    return parts[0].get('text') if isinstance(parts, list) and len(parts) == 1 and isinstance(parts[0], dict) and parts[0].get('kind') in ('literal', 'literal-data') and isinstance(parts[0].get('text'), str) else None


def feature_environment_context(step_id, environment):
    if not isinstance(environment, dict):
        return False
    if any(isinstance(key, str) and key.startswith('CARGO_FEATURE_') for key in environment):
        return True
    return (isinstance(step_id, str) and SCRIPT_SCOPE.match(step_id) is not None and
            'CARGO_MANIFEST_DIR' in environment and 'CARGO_PKG_NAME' in environment)


def part(word, kind, name=None, path=None):
    parts = word.get('parts') if isinstance(word, dict) else None
    return isinstance(parts, list) and len(parts) == 1 and isinstance(parts[0], dict) and parts[0].get('kind') == kind and (name is None or parts[0].get('name') == name) and (path is None or parts[0].get('path') == path)


def produced(stage, step, output):
    return {'kind': 'produced', 'action_id': stage + '/' + step, 'output_id': output}


def cfg_features(arguments):
    """Decode only exact original --cfg feature operands, not dependency activations."""
    features, malformed = [], []
    if not isinstance(arguments, list):
        return features, ['non-list arguments']
    words = [literal(w) for w in arguments]
    for i, word in enumerate(words):
        if word == '--cfg':
            operand = words[i + 1] if i + 1 < len(words) else None
        elif isinstance(word, str) and word.startswith('--cfg='):
            operand = word[6:]
        else:
            continue
        if operand is None:
            malformed.append({'position': i, 'reason': 'unresolved cfg operand'})
        elif operand.startswith('feature='):
            try:
                name = json.loads(operand[8:])
                if not isinstance(name, str) or not name:
                    raise ValueError('feature name is empty or non-string')
                features.append(name)
            except (ValueError, TypeError):
                malformed.append({'position': i, 'reason': 'invalid quoted feature cfg'})
    return features, malformed


def env_name(feature):
    # Rust's uppercasing is Unicode-aware; Python upper() implements the same
    # mapping for the source-owned ASCII Cargo feature names in this cohort.
    return 'CARGO_FEATURE_' + feature.upper().replace('-', '_')


def nested_operations(value, trail=()):
    """Walk decoded static/generated/nested JSON; runtime-produced bytes stay unknown."""
    if isinstance(value, dict):
        if 'environment' in value or any(k == 'arguments' or k.endswith('_arguments') for k in value):
            yield trail, value
        for k, v in value.items():
            if isinstance(v, (dict, list)):
                yield from nested_operations(v, trail + (str(k),))
    elif isinstance(value, list):
        for i, v in enumerate(value):
            if isinstance(v, (dict, list)):
                yield from nested_operations(v, trail + (str(i),))


def all_argument_words(operation):
    for key, value in operation.items():
        if (key == 'arguments' or key.endswith('_arguments')) and isinstance(value, list):
            yield from value




def counters():
    return {'total': 0, 'covered': 0, 'uncovered': 0, 'invalid': 0}

def blake3_bytes(data, binary):
    return subprocess.check_output([str(binary), '--no-names'], input=data, text=False).decode().strip()


def profile_mantle_record(profile):
    """Boundedly select the authenticated pretty-JSON Mantle role record.

    A full profile raw digest is checked by the caller first. Skip parsing other
    large records; the selected record is parsed with duplicate-key rejection.
    """
    found = None
    inside = False
    current = None
    with profile.open('rb', buffering=1024 * 1024) as stream:
        for line in stream:
            if not inside:
                if line.strip() == b'"records": [':
                    inside = True
                continue
            if current is None:
                if line in (b'    {\n', b'    {\r\n'):
                    current = bytearray(line)
                elif line.lstrip().startswith(b']'):
                    break
                continue
            current.extend(line)
            if len(current) > 20 * 1024**3:
                raise ValueError('profile source record exceeds declared parser bound')
            if line not in (b'    },\n', b'    },\r\n', b'    }\n', b'    }\r\n'):
                continue
            if b'"bootstrap_profile_class": "mantle-source"' in current:
                def no_dupes(pairs):
                    result = {}
                    for key, value in pairs:
                        if key in result:
                            raise ValueError('duplicate output-profile record key')
                        result[key] = value
                    return result
                raw = current.rstrip().removesuffix(b',')
                obj = json.loads(raw, object_pairs_hook=no_dupes)
                if obj.get('metadata', {}).get('bootstrap_profile_class') != 'mantle-source' or found is not None:
                    raise ValueError('ambiguous authenticated Mantle role record')
                found = obj
            current = None
    if not inside or found is None or current is not None:
        raise ValueError('output profile has no complete unique Mantle role record')
    return found


def member_record_bindings(cohort, subject, members, b3_file, b3_binary):
    """Bind physical authored members to the complete output profile record."""
    profile = Path(subject['profile_file'])
    if profile != cohort / 'source-profile.json' or profile.stat().st_size != subject['profile_file_bytes'] or b3_file(profile) != subject['profile_file_raw_blake3']:
        raise ValueError('raw output-profile subject identity differs')
    if subject['full_profile_ssh_stream_blake3'] != subject['profile_file_raw_blake3']:
        raise ValueError('output-profile transport identity differs')
    with profile.open('rb') as stream:
        stream.seek(-min(profile.stat().st_size, 65_536), 2)
        footer = stream.read()
    manifest = re.search(rb'^  "manifest_blake3"\s*:\s*"([0-9a-f]{64})"\s*,?\s*$', footer, re.M)
    if manifest is None or manifest.group(1).decode() != subject['prepared_manifest_blake3']:
        raise ValueError('output-profile manifest footer identity differs from subject')
    record = profile_mantle_record(profile)
    if record.get('kind') != 'local-path' or not isinstance(record.get('files'), list):
        raise ValueError('Mantle source role record schema differs')
    if sum(entry.get('size', 0) for entry in record['files']) != record.get('payload_bytes'):
        raise ValueError('Mantle role record payload count differs from complete file inventory')
    metadata = record.get('metadata', {})
    process = subprocess.Popen([str(b3_binary), '--no-names'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    encoder = json.JSONEncoder(ensure_ascii=False, sort_keys=True, separators=(',', ':'))
    try:
        process.stdin.write(b'kind\0')
        process.stdin.write(json.dumps(record['kind'], ensure_ascii=False, separators=(',', ':')).encode())
        process.stdin.write(b'\nmetadata\0')
        for fragment in encoder.iterencode(metadata):
            process.stdin.write(fragment.encode())
        process.stdin.write(b'\nfiles\0')
        for entry in record['files']:
            process.stdin.write(json.dumps(entry, ensure_ascii=False, separators=(',', ':')).encode() + b'\n')
        digest, stderr = process.communicate(timeout=3600)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
    if process.returncode or digest.decode().strip() != record.get('content_blake3'):
        raise ValueError('Mantle role record canonical content identity differs: ' + stderr.decode(errors='replace')[-300:])
    selected = {}
    for member in members:
        rows = [x for x in record['files'] if x.get('path') == member]
        if not rows:
            raise ValueError('authored unit plan absent from authenticated Mantle role record: ' + member)
        rows.sort(key=lambda x: -1 if x.get('chunk_index') is None else x['chunk_index'])
        if len(rows) == 1:
            if rows[0].get('chunk_index') is not None or rows[0].get('chunk_count') is not None:
                raise ValueError('single member has noncanonical chunk markers')
        elif ([x.get('chunk_index') for x in rows] != list(range(len(rows))) or
              any(x.get('chunk_count') != len(rows) for x in rows) or
              any(x.get('size') != 67_108_864 for x in rows[:-1]) or
              not (0 < rows[-1].get('size', 0) <= 67_108_864)):
            raise ValueError('unit-plan record has noncanonical chunk geometry')
        path = cohort / 'inputs/mantle-source' / member
        if not path.is_file() or path.is_symlink() or path.stat().st_size != sum(x['size'] for x in rows):
            raise ValueError('unit-plan physical member differs in kind or size')
        with path.open('rb') as physical:
            for entry in rows:
                encoded = entry.get('content_hex')
                if (entry.get('file_type') != 'regular' or entry.get('executable') is not False or
                    not isinstance(encoded, str) or len(encoded) != 2 * entry['size'] or
                    not re.fullmatch('[0-9a-f]*', encoded)):
                    raise ValueError('unit-plan record member is not canonical regular embedded bytes')
                data = bytes.fromhex(encoded)
                if blake3_bytes(data, b3_binary) != entry.get('blake3') or physical.read(len(data)) != data:
                    raise ValueError('embedded unit-plan chunk differs from record or physical source')
            if physical.read(1):
                raise ValueError('unit-plan physical member has trailing bytes')
        selected[member] = {'path': str(path), 'size_bytes': path.stat().st_size,
                            'blake3': b3_file(path), 'record_content_blake3': record['content_blake3'],
                            'record_identity': record['identity'], 'chunks': len(rows),
                            'chunk_blake3': [row['blake3'] for row in rows],
                            'embedded_bytes_compared': path.stat().st_size}
    return selected


def plain_cfg_features(arguments):
    """Frozen rust_plan::build_script_feature_environment selector on source plan args."""
    if not isinstance(arguments, list) or any(not isinstance(arg, str) for arg in arguments):
        raise ValueError('source planner derivation args are not strings')
    selected = []
    for i, argument in enumerate(arguments):
        cfg = arguments[i + 1] if argument == '--cfg' and i + 1 < len(arguments) else argument.removeprefix('--cfg=') if argument.startswith('--cfg=') else None
        if cfg is None or not cfg.startswith('feature='):
            continue
        try:
            feature = json.loads(cfg[len('feature='):])
        except (ValueError, TypeError):
            continue  # serde_json::from_str::<String>(...).ok() skips malformed cfg
        if isinstance(feature, str):
            selected.append(feature)
    return selected


def source_plan_operand(value, roots):
    if not isinstance(value, str) or not value or '\x00' in value:
        raise ValueError('missing original plan source operand')
    mantle = roots['mantle-source']
    vendor = roots['vendor-inputs']
    path = Path(value)
    if path.is_relative_to('/mantle/release/source'):
        path = mantle / path.relative_to('/mantle/release/source')
    elif not path.is_absolute():
        path = mantle / path
    path = Path(posixpath.normpath(str(path)))
    for role, root in (('vendor-inputs', vendor), ('mantle-source', mantle)):
        root = Path(posixpath.normpath(str(root)))
        if path.is_relative_to(root):
            member = path.relative_to(root).as_posix()
            return role, '' if member == '.' else member
    raise ValueError('planner source operand is outside admitted physical source roles')


def unit_plan_facts(plan_documents, recipes):
    """Map immutable source-plan unit derivations to complete semantic recipe owners."""
    facts = {}
    evidence = {}
    for stage in STAGES:
        document = plan_documents.get(stage)
        if not isinstance(document, dict) or document.get('schema') != 'mantle-source-unit-derivation-plan-v1' or document.get('stage_id') != stage:
            evidence[stage] = {'uncovered': ['missing/invalid source-only unit-plan member']}
            facts[stage] = {}
            continue
        roots = document.get('source_roots')
        if (not isinstance(roots, dict) or any(
                not isinstance(roots.get(role), str) or not Path(roots[role]).is_absolute()
                or '\x00' in roots[role] for role in ('mantle-source', 'vendor-inputs'))):
            evidence[stage] = {'uncovered': ['missing/invalid original planner source-root context']}
            facts[stage] = {}
            continue
        roots = {role: Path(roots[role]) for role in ('mantle-source', 'vendor-inputs')}
        graph = document.get('unit_graph')
        derivations = graph.get('derivations') if isinstance(graph, dict) else None
        if not isinstance(derivations, list) or graph.get('ready') is not True or graph.get('blockers') or graph.get('derivation_count') != len(derivations):
            evidence[stage] = {'uncovered': ['source-only unit graph is incomplete or blocked']}
            facts[stage] = {}
            continue
        by_id = {unit.get('unit_id'): unit for unit in derivations if isinstance(unit, dict) and isinstance(unit.get('unit_id'), str)}
        if len(by_id) != len(derivations):
            evidence[stage] = {'uncovered': ['source-only graph has duplicate/invalid unit identities']}
            facts[stage] = {}
            continue
        by_metadata = collections.defaultdict(list)
        for unit in derivations:
            metadata = unit.get('rustc_metadata_hash')
            if isinstance(metadata, str) and re.fullmatch('[0-9a-f]{16}', metadata):
                by_metadata[metadata].append(unit['unit_id'])
        owner = collections.defaultdict(set)
        for name, references in recipes[stage].get('semantic_roots', {}).items():
            if not isinstance(name, str) or not name.startswith('rust-unit:') or not isinstance(references, list):
                continue
            for ref in references:
                action = ref.get('action_id') if isinstance(ref, dict) else None
                prefix = stage + '/unit-'
                if isinstance(action, str) and action.startswith(prefix) and action[len(prefix):len(prefix) + 5].isdigit():
                    owner[action[len(prefix):len(prefix) + 5]].add(name.removeprefix('rust-unit:'))
        facts[stage] = {}
        uncovered = []
        mapped_ids = set()
        metadata_fallbacks = 0
        recipe_steps = {row.get('id'): row for row in recipes[stage]['steps'] if isinstance(row, dict) and isinstance(row.get('id'), str)}
        for step in recipes[stage]['steps']:
            match = UNIT.fullmatch(step.get('id', '')) if isinstance(step, dict) else None
            if not match or match[2] != 'compile':
                continue
            index = match[1]
            ids = owner.get(index, set()) & by_id.keys()
            binding = 'semantic-root'
            if len(ids) != 1:
                try:
                    source_args, _, _ = compiler_static_arguments(stage, step, recipe_steps)
                    words = [literal(word) for word in source_args]
                    metadata = [arg[len('metadata='):] for i, arg in enumerate(words) if i and words[i - 1] == '-C'
                                and isinstance(arg, str) and arg.startswith('metadata=')]
                    ids = set(by_metadata.get(metadata[0], [])) if len(metadata) == 1 else set()
                except (KeyError, TypeError, ValueError, AttributeError):
                    ids = set()
                binding = 'source-plan-metadata-identity'
            if len(ids) != 1:
                uncovered.append({'ordinal': index, 'reason': 'source plan derivation has no unique semantic/metadata identity mapping'})
                continue
            unit_id = next(iter(ids))
            if binding != 'semantic-root':
                metadata_fallbacks += 1
            mapped_ids.add(unit_id)
            unit = by_id[unit_id]
            derivation = unit.get('derivation', {})
            try:
                features = plain_cfg_features(derivation['args'])
                if len(features) != len(set(features)) or any(not FEATURE_NAME.fullmatch(feature) for feature in features):
                    raise ValueError('planner feature cfg is duplicate/invalid')
                manifest_dir = derivation['env']['CARGO_MANIFEST_DIR']
                role, package_path = source_plan_operand(manifest_dir, roots)
                primary_args = [arg for arg in derivation['args'] if isinstance(arg, str) and
                                not arg.startswith('-') and arg.endswith('.rs')]
                if len(primary_args) != 1:
                    raise ValueError('planner has no unique primary source')
                primary_role, primary_path = source_plan_operand(primary_args[0], roots)
                package_prefix = package_path.rstrip('/') + '/' if package_path else ''
                if primary_role != role or not primary_path.startswith(package_prefix):
                    raise ValueError('planner primary source differs from original package')
                facts[stage][index] = {'unit_id': unit_id, 'package_id': unit.get('package_id'),
                                       'target_kind': unit.get('target_kind'),
                                       'selected_features': features,
                                       'package_role': role, 'package_path': package_path,
                                       'primary_role': primary_role, 'primary_path': primary_path,
                                       'source_digest': unit.get('source_digest'),
                                       'mapping_basis': binding}
            except (KeyError, TypeError, ValueError, AttributeError) as error:
                uncovered.append({'ordinal': index, 'reason': str(error)[:200]})
        orphan_ids = set(by_id) - mapped_ids
        if orphan_ids:
            uncovered.append({'reason': 'source plan derivations lack owning recipe compiler', 'count': len(orphan_ids),
                              'examples': sorted(orphan_ids)[:8]})
        evidence[stage] = {'plan_derivations_total': len(derivations), 'mapped_compile_units': len(facts[stage]),
                           'orphan_derivations': len(orphan_ids), 'metadata_identity_fallbacks': metadata_fallbacks,
                           'uncovered': uncovered[:LIMIT], 'uncovered_count': len(uncovered),
                           'source_plan_digest_blake3': graph.get('digest_blake3')}
    return facts, evidence



def compiler_static_arguments(stage, compile_step, steps):
    """Trace authored compiler args through a static template, not runtime bytes."""
    operation = compile_step.get('operation', {})
    direct = operation.get('arguments')
    if not isinstance(direct, list):
        raise ValueError('compiler arguments absent')
    input_name = operation.get('invocation_input')
    if not input_name:
        return direct, {'projection': 'direct', 'consumer_step': compile_step['id']}, None
    if direct:
        raise ValueError('compiler mixes direct and generated invocation arguments')
    ref = compile_step.get('inputs', {}).get(input_name)
    if (not isinstance(ref, dict) or ref.get('kind') != 'produced' or ref.get('output_id') != 'invocation' or
        not isinstance(ref.get('action_id'), str) or not ref['action_id'].startswith(stage + '/')):
        raise ValueError('generated compiler invocation has no exact local producer reference')
    invocation_id = ref.get('action_id', '').removeprefix(stage + '/')
    producer = steps.get(invocation_id)
    if not producer or producer.get('operation', {}).get('kind') != 'exec' or producer.get('outputs', {}).get('invocation', {}).get('kind') != 'text':
        raise ValueError('generated compiler invocation producer is not a bounded text-producing exec')
    template_ref = producer.get('inputs', {}).get('template')
    if (not isinstance(template_ref, dict) or template_ref.get('kind') != 'produced' or template_ref.get('output_id') != 'template' or
        not isinstance(template_ref.get('action_id'), str) or not template_ref['action_id'].startswith(stage + '/')):
        raise ValueError('generated compiler invocation lacks exact local authored template')
    template_id = template_ref.get('action_id', '').removeprefix(stage + '/')
    template = steps.get(template_id)
    if not template or template.get('operation', {}).get('kind') != 'write-text' or template.get('outputs', {}).get('template', {}).get('kind') != 'text':
        raise ValueError('generated compiler source template is not an exact write-text producer')
    document = json.loads(template['operation']['text'])
    invocation = document.get('invocation') if isinstance(document, dict) else None
    if not isinstance(invocation, dict) or not isinstance(invocation.get('arguments'), list) or not isinstance(invocation.get('environment'), dict):
        raise ValueError('generated compiler template lacks typed invocation arguments/environment')
    return invocation['arguments'], {'projection': 'static-template', 'consumer_step': compile_step['id'],
                                     'producer_step': invocation_id, 'template_step': template_id}, document


def audit_data(recipes, unit_facts=None, source_member=None):
    """Audit actual decoded recipes; facts must be independently source-derived.

    unit_facts shape: {stage: {five-digit ordinal: {'unit_id': str,
    'package_role': str, 'package_path': str, 'selected_features': [str]}}}.
    A caller must bind such facts to its original plan independently; recipe cfg,
    environment or peer-stage copies are not acceptable fact sources.
    source_member(role,path) -> bool checks admitted original source membership.
    """
    result = {'per_stage': {}, 'coverage': {name: counters() for name in ('package_exclusions', 'feature_environment', 'feature_cfg')},
              'diagnostics': [], 'uncovered_obligations': [], 'symbolic_contracts': [],
              'fact_provenance': 'independent source-derived unit facts required; recipe selectors never stand in for facts'}

    def observe(stage, category, identity, reason=None, missing=False):
        count = result['coverage'][category]
        local = result['per_stage'][stage]['coverage'][category]
        for item in (count, local):
            item['total'] += 1
            item['uncovered' if missing else 'invalid' if reason else 'covered'] += 1
        if reason or missing:
            row = {'stage': stage, 'category': category, 'identity': identity, 'reason': reason or 'required independent unit provenance absent'}
            if len(result['diagnostics']) < LIMIT:
                result['diagnostics'].append(row)
            if missing and len(result['uncovered_obligations']) < LIMIT:
                result['uncovered_obligations'].append(row)

    for stage in STAGES:
        recipe = recipes.get(stage)
        if not isinstance(recipe, dict) or recipe.get('stage_id') != stage or not isinstance(recipe.get('steps'), list):
            result['per_stage'][stage] = {'coverage': {name: counters() for name in result['coverage']}, 'unit_count': 0, 'source_binding': None}
            for name in result['coverage']:
                observe(stage, name, stage, 'missing or invalid selected Mantle recipe', missing=True)
            continue
        rows = recipe['steps']
        steps = {s.get('id'): s for s in rows if isinstance(s, dict) and isinstance(s.get('id'), str)}
        result['per_stage'][stage] = {'coverage': {name: counters() for name in result['coverage']}, 'unit_count': 0,
                                      'script_count': 0, 'original_package_count': 0, 'source_binding': None,
                                      'unit_evidence': []}
        if len(steps) != len(rows):
            for name in result['coverage']:
                observe(stage, name, stage, 'duplicate or invalid step identifier')
        compiles = {}
        scripts = {}
        originals = {}
        observed_source_packages = []
        nested_env = []
        nested_cfg = []
        for s in rows:
            if not isinstance(s, dict):
                continue
            sid = s.get('id')
            m = UNIT.fullmatch(sid) if isinstance(sid, str) else None
            if m:
                index, kind = m.groups()
                {'compile': compiles, 'script': scripts, 'script-original-package': originals}[kind][index] = s
            for trail, op in nested_operations(s.get('operation', {})):
                argv = list(all_argument_words(op))
                if any(literal(w) == '--source-package' for w in argv):
                    observed_source_packages.append((sid, trail, s, op))
                env = op.get('environment')
                if feature_environment_context(sid, env):
                    nested_env.append((sid, trail, env))
                if any(isinstance(literal(w), str) and (literal(w).startswith('feature=') or literal(w).startswith('--cfg=feature=')) for w in argv):
                    nested_cfg.append((sid, trail))
        static_invocations = 0
        static_invocation_evidence = []
        symbolic_invocations = 0
        for s in rows:
            if not isinstance(s, dict):
                continue
            op = s.get('operation', {})
            if not isinstance(op, dict) or not op.get('invocation_input'):
                continue
            ref = s.get('inputs', {}).get(op['invocation_input'])
            producer_id = ref.get('action_id', '').removeprefix(stage + '/') if isinstance(ref, dict) and ref.get('kind') == 'produced' else None
            producer = steps.get(producer_id)
            if not producer or producer.get('operation', {}).get('kind') != 'write-text':
                symbolic_invocations += 1
                continue
            try:
                request = json.loads(producer['operation']['text'])
                if not isinstance(request, dict) or not isinstance(request.get('arguments'), list) or not isinstance(request.get('environment'), dict):
                    raise ValueError('invalid typed invocation fields')
            except (ValueError, TypeError, KeyError):
                for category in result['coverage']:
                    observe(stage, category, str(s.get('id')), 'static invocation cannot be decoded', missing=True)
                continue
            static_invocations += 1
            static_invocation_evidence.append({'consumer_step': s.get('id'), 'producer_step': producer_id})
            for trail, request_op in nested_operations(request, ('static-invocation',)):
                argv = list(all_argument_words(request_op))
                if any(literal(w) == '--source-package' for w in argv):
                    observed_source_packages.append((s.get('id'), trail, s, request_op))
                env = request_op.get('environment')
                if feature_environment_context(s.get('id'), env):
                    nested_env.append((s.get('id'), trail, env))
                if any(isinstance(literal(w), str) and (literal(w).startswith('feature=') or literal(w).startswith('--cfg=feature=')) for w in argv):
                    nested_cfg.append((s.get('id'), trail))
        compiler_projections = {}
        template_owners = {}
        for index, compile_step in compiles.items():
            try:
                args, binding, document = compiler_static_arguments(stage, compile_step, steps)
                compiler_projections[index] = (args, binding, None)
                if document is not None:
                    template_id = binding['template_step']
                    if template_id in template_owners:
                        raise ValueError('authored compiler template has multiple owning consumers')
                    template_owners[template_id] = index
            except (ValueError, TypeError, KeyError, AttributeError) as error:
                compiler_projections[index] = ([], None, str(error)[:200])
        for row in rows:
            if not isinstance(row, dict) or row.get('operation', {}).get('kind') != 'write-text':
                continue
            text = row['operation'].get('text')
            if not isinstance(text, str) or not any(token in text for token in ('feature=', 'CARGO_FEATURE_', 'CARGO_MANIFEST_DIR', '--source-package')):
                continue
            try:
                document = json.loads(text)
                for trail, operation in nested_operations(document):
                    words = list(all_argument_words(operation))
                    if any(literal(word) == '--source-package' for word in words):
                        observed_source_packages.append((row.get('id'), trail, row, operation))
                    env = operation.get('environment')
                    if feature_environment_context(row.get('id'), env):
                        nested_env.append((row.get('id'), trail, env))
                    if any(isinstance(literal(word), str) and
                           (literal(word).startswith('feature=') or literal(word).startswith('--cfg=feature='))
                           for word in words):
                        nested_cfg.append((row.get('id'), trail))
            except (ValueError, TypeError, KeyError):
                observe(stage, 'feature_cfg', str(row.get('id')), 'static feature-bearing text cannot be decoded', missing=True)
                observe(stage, 'feature_environment', str(row.get('id')), 'static feature-bearing text cannot be decoded', missing=True)
        local = result['per_stage'][stage]
        local['unit_count'] = len(compiles)
        local['script_count'] = len(scripts)
        local['original_package_count'] = len(originals)
        local['static_source_package_invocations'] = len(observed_source_packages)
        local['static_feature_environment_instances'] = len(nested_env)
        local['static_feature_cfg_instances'] = len(nested_cfg)
        local['static_decoded_invocations'] = static_invocations
        local['runtime_symbolic_invocations'] = symbolic_invocations
        local['static_invocation_evidence'] = static_invocation_evidence
        result['symbolic_contracts'].append({'stage': stage, 'runtime_invocation_bytes_absent': symbolic_invocations,
                                             'static_decoded_invocations': static_invocations})
        semantic_unit_ids = collections.defaultdict(set)
        semantic_scripts = set()
        semantic_script_units = collections.defaultdict(set)
        for root_name, references in recipe.get('semantic_roots', {}).items():
            if not isinstance(root_name, str) or not isinstance(references, list):
                continue
            for reference in references:
                if not isinstance(reference, dict) or reference.get('kind') != 'produced':
                    continue
                action = reference.get('action_id', '')
                prefix = stage + '/unit-'
                if not isinstance(action, str) or not action.startswith(prefix):
                    continue
                unit_index = action[len(prefix):len(prefix) + 5]
                if not unit_index.isdigit():
                    continue
                if root_name.startswith('rust-unit:'):
                    semantic_unit_ids[unit_index].add(root_name.removeprefix('rust-unit:'))
                elif root_name.startswith('build-script:'):
                    semantic_scripts.add(unit_index)
                    semantic_script_units[unit_index].add(root_name.removeprefix('build-script:'))
        local['semantic_unit_root_count'] = sum(len(v) for v in semantic_unit_ids.values())
        local['semantic_script_owner_count'] = len(semantic_scripts)
        facts = unit_facts.get(stage, {}) if isinstance(unit_facts, dict) else {}
        if not compiles or not scripts or not isinstance(facts, dict):
            for name in result['coverage']:
                observe(stage, name, stage, 'unknown or empty unit/script provenance', missing=True)
        source_script_ids = {fact['unit_id'] for fact in facts.values() if isinstance(fact, dict) and fact.get('target_kind') == 'custom-build'}
        source_script_owners = {index for index, fact in facts.items() if isinstance(fact, dict) and
                                fact.get('target_kind') == 'custom-build' and semantic_unit_ids[index] == {fact.get('unit_id')}}
        source_script_owner_counts = collections.Counter(facts[index]['unit_id'] for index in source_script_owners)
        for unit_id in source_script_ids:
            if source_script_owner_counts[unit_id] != 1:
                observe(stage, 'feature_environment', unit_id, 'source-planned build script has no unique semantic compiler owner', missing=True)
        for index in source_script_owners:
            if semantic_script_units[index] != {facts[index]['unit_id']}:
                observe(stage, 'feature_environment', index, 'source-planned build script has no matching semantic execution root', missing=True)
        local['direct_script_step_count'] = len(scripts)
        local['script_count'] = len(source_script_owners)
        expected_originals = set()
        validated_facts = {}
        for index, compile_step in compiles.items():
            inputs = compile_step.get('inputs', {})
            original = inputs.get('source-package') if isinstance(inputs, dict) else None
            source_role = original.get('role') if isinstance(original, dict) else None
            source_path = original.get('path') if isinstance(original, dict) else None
            args, projection, projection_error = compiler_projections[index]
            primary = [p for w in args if isinstance(w, dict) for p in w.get('parts', []) if isinstance(p, dict) and p.get('kind') == 'input' and p.get('path', '').endswith('.rs')]
            original_owned = (isinstance(source_role, str) and isinstance(source_path, str) and original.get('kind') == 'source' and
                              any(p.get('name') == source_role and p.get('path', '').startswith(source_path.rstrip('/') + '/' if source_path else '') for p in primary))
            selected, malformed = cfg_features(args)
            fact = facts.get(index)
            fact_valid = (isinstance(fact, dict) and isinstance(fact.get('selected_features'), list) and
                          all(isinstance(x, str) and FEATURE_NAME.fullmatch(x) for x in fact['selected_features']) and
                          len(set(fact['selected_features'])) == len(fact['selected_features']) and
                          isinstance(fact.get('unit_id'), str) and bool(fact['unit_id']) and
                          isinstance(fact.get('package_id'), str) and bool(fact['package_id']) and
                          isinstance(fact.get('source_digest'), dict) and
                          (not semantic_unit_ids[index] or semantic_unit_ids[index] == {fact['unit_id']}) and
                          (index not in source_script_owners or (fact.get('target_kind') == 'custom-build' and
                           fact.get('package_role') == source_role and fact.get('package_path') == source_path and
                           len(primary) == 1 and primary[0].get('name') == fact.get('primary_role') and
                           primary[0].get('path') == fact.get('primary_path'))))
            validated_facts[index] = fact if fact_valid else None
            if fact_valid and projection_error is None:
                cfg_reason = None if not malformed and collections.Counter(selected) == collections.Counter(fact['selected_features']) else 'compile cfg differs from independently bound selected unit features or is malformed'
                observe(stage, 'feature_cfg', index, cfg_reason)
            else:
                observe(stage, 'feature_cfg', index, projection_error or 'no independently source-derived selected-feature unit fact', missing=True)
            if index in source_script_owners:
                script = scripts.get(index, {})
                environment = script.get('operation', {}).get('environment', {})
                actual_env = {k: v for k, v in environment.items() if isinstance(k, str) and k.startswith('CARGO_FEATURE_')} if isinstance(environment, dict) else None
                package_ref = script.get('inputs', {}).get('package')
                if (isinstance(fact, dict) and fact.get('package_path') == 'nickel-lang-parser' and
                    fact.get('package_role') == 'vendor-inputs' and fact.get('target_kind') == 'custom-build' and
                    isinstance(fact.get('package_id'), str) and '#nickel-lang-parser@' in fact['package_id']):
                    expected_originals.add(index)
                    prep = originals.get(index)
                    expected_ref = produced(stage, f'unit-{index}-script-original-package', 'package')
                    prep_inputs = prep.get('inputs', {}) if isinstance(prep, dict) else {}
                    prep_op = prep.get('operation', {}) if isinstance(prep, dict) else {}
                    prep_args = prep_op.get('arguments') if isinstance(prep_op, dict) else None
                    helper = prep_inputs.get('metadata-helper') if isinstance(prep_inputs, dict) else None
                    helper_step = steps.get('helper-metadata-link.native-products') if helper == produced(stage, 'helper-metadata-link.native-products', 'helper') else None
                    valid = (fact_valid and original_owned and prep_inputs.get('original-package') == original and helper_step is not None and
                             helper_step.get('outputs', {}).get('helper', {}).get('kind') == 'executable' and
                             prep_op.get('kind') == 'exec' and prep_op.get('executable_input') == 'metadata-helper' and
                             isinstance(prep_args, list) and len(prep_args) == 4 and literal(prep_args[0]) == '--source-package' and
                             part(prep_args[1], 'input', 'original-package', '') and part(prep_args[2], 'output', 'package', '') and
                             part(prep_args[3], 'input', 'original-package', 'src/grammar.rs') and
                             isinstance(prep.get('outputs'), dict) and prep['outputs'].get('package', {}).get('kind') == 'tree' and
                             script.get('operation', {}).get('executable_input') == 'script' and
                             package_ref == expected_ref and source_member is not None and
                             source_member('vendor-inputs', 'nickel-lang-parser/src/grammar.rs')) if prep else False
                    observe(stage, 'package_exclusions', index, None if valid else 'missing/wrong exclusion operand, owning helper, source member, output or script consumer')
                elif source_path == 'nickel-lang-parser' or (isinstance(source_path, str) and source_path.endswith('/nickel-lang-parser')):
                    observe(stage, 'package_exclusions', index, 'unknown or foreign nickel package provenance', missing=True)
            elif source_path == 'nickel-lang-parser':
                observe(stage, 'feature_environment', index, 'nickel source compile has no original script', missing=True)
            local['unit_evidence'].append({'ordinal': index,
                                           'source_package': original, 'source_owned_primary': original_owned,
                                           'fact_available': fact_valid, 'fact_unit_id': fact.get('unit_id') if fact_valid else None,
                                           'compiler_selector_projection': projection,
                                           'projection_error': projection_error,
                                           'observed_cfg_count': len(selected),
                                           'observed_feature_environment_count': len(actual_env) if index in scripts and actual_env is not None else None,
                                           'script_package': package_ref if index in scripts else None})
        if isinstance(facts, dict):
            for index in facts.keys() - compiles.keys():
                observe(stage, 'feature_cfg', str(index), 'source-planned unit has no owning compiler', missing=True)
                observe(stage, 'feature_environment', str(index), 'source-planned unit has no owning compiler', missing=True)
        for index in semantic_scripts - source_script_owners:
            observe(stage, 'feature_environment', index, 'semantic build-script root has no source-planned compiler owner', missing=True)
        for index in scripts.keys() - semantic_scripts:
            observe(stage, 'feature_environment', index, 'script has no owning semantic build-script root')
        for index in scripts.keys() - compiles.keys():
            observe(stage, 'feature_environment', index, 'original script has no owning compile unit', missing=True)
            observe(stage, 'feature_cfg', index, 'original script has no owning compile unit', missing=True)
        for index in originals.keys() - expected_originals:
            observe(stage, 'package_exclusions', index, 'extraneous original-package producer or foreign source')
        expected_step_ids = {f'unit-{x}-script-original-package' for x in expected_originals}
        for sid, trail, _, _ in observed_source_packages:
            if sid not in expected_step_ids or trail != ():
                observe(stage, 'package_exclusions', str(sid) + '/' + '/'.join(trail), 'extraneous or nested unowned source-package invocation')
        environment_context_counts = collections.Counter()
        for sid, trail, environment in nested_env:
            identity = str(sid) + '/' + '/'.join(trail)
            owner = SCRIPT_SCOPE.match(sid) if isinstance(sid, str) else None
            index = owner[1] if owner else None
            if index not in source_script_owners:
                observe(stage, 'feature_environment', identity, 'unowned static/generated/nested feature environment')
                continue
            environment_context_counts[index] += 1
            fact = validated_facts.get(index)
            if fact is None:
                observe(stage, 'feature_environment', identity, 'no independently source-derived selected-feature unit fact', missing=True)
                continue
            actual_env = {key: value for key, value in environment.items() if isinstance(key, str) and key.startswith('CARGO_FEATURE_')}
            expected_env = {env_name(name): {'parts': [{'kind': 'literal', 'text': '1'}]} for name in fact['selected_features']}
            reason = None if actual_env == expected_env else 'script feature environment differs from selected unit features'
            observe(stage, 'feature_environment', identity, reason)
        for index in source_script_owners - environment_context_counts.keys():
            observe(stage, 'feature_environment', index, 'source-selected build script has no Cargo package environment context', missing=True)
        local['script_environment_context_counts'] = dict(sorted(environment_context_counts.items()))
        for sid, trail in nested_cfg:
            direct = not trail and isinstance(sid, str) and re.fullmatch(r'unit-\d{5}-compile', sid) and sid[5:10] in compiles
            templated = trail == ('invocation',) and sid in template_owners and compiler_projections[template_owners[sid]][2] is None
            if not direct and not templated:
                observe(stage, 'feature_cfg', str(sid) + '/' + '/'.join(trail), 'unowned static/generated/nested feature cfg')
        # Explicitly reject missing selection roots; zero instances cannot pass.
        if not expected_originals:
            observe(stage, 'package_exclusions', stage, 'no independently identified nickel build-script owner', missing=True)
        if not local['static_source_package_invocations']:
            observe(stage, 'package_exclusions', stage, 'no actual source-package invocation', missing=True)
        if not local['unit_count'] or not local['script_count']:
            observe(stage, 'feature_cfg', stage, 'no original compiler/script pair', missing=True)
    result['bounded_verdict'] = all(x['total'] and x['uncovered'] == x['invalid'] == 0 for x in result['coverage'].values())
    result['diagnostics_suppressed'] = sum(x['invalid'] + x['uncovered'] for x in result['coverage'].values()) - len(result['diagnostics'])
    return result


def main():
    if len(sys.argv) != 5:
        raise SystemExit('usage: mantle-selector-audit.py COHORT FROZEN_SOURCE REVIEW_SCRATCH SUBJECT_JSON')
    cohort, source, scratch, subject_path = map(Path, sys.argv[1:])
    helper_path = Path(__file__).with_name('graph-audit.py')
    graph_path = scratch / 'graph-audit.json'
    # graph-audit imports validate the same four-argument subject contract; main
    # remains guarded, and no source/product/compiler/provider code is executed.
    g = runpy.run_path(str(helper_path), run_name='mantle_selector_data_only')
    subject = g['SUBJECT']
    raw_graph = graph_path.read_bytes()
    report = json.loads(raw_graph)
    observed_subject = g['b3'](subject_path)
    if (cohort.resolve() != Path(subject['cohort']).resolve() or source.resolve() != Path(subject['frozen_source']).resolve() or
            report.get('subject_binding', {}).get('subject_json_observed', {}).get('blake3') != observed_subject or
            not all(report.get('subject_binding', {}).get('observed_claim_checks', {}).values()) or
            not report.get('subject_binding', {}).get('observed_claim_checks')):
        raise ValueError('graph report or subject does not bind exact current cohort/source')
    selected = report.get('selected_input_blake3', {})
    route_path = cohort / 'inputs/mantle-source/bootstrap/source-built-cold-route.json'
    route = json.loads(route_path.read_bytes())
    if route != report.get('selectors') or len(route.get('mantle_recipes', [])) != 2:
        raise ValueError('current route differs from selected graph report')
    recipes, inputs = {}, {}
    for stage, member in zip(STAGES, route['mantle_recipes'], strict=True):
        if not g['normal_rel'](member, False):
            raise ValueError('noncanonical selected Mantle recipe')
        path = cohort / 'inputs/mantle-source' / member
        claim = selected.get(stage.replace('-', '_'), {})
        actual = {'path': str(path), 'size_bytes': path.stat().st_size, 'blake3': g['b3'](path)}
        if claim != actual:
            raise ValueError(f'graph selected recipe binding mismatch: {stage}')
        recipes[stage] = json.loads(path.read_bytes())
        inputs[stage] = actual
    if selected.get('cold_route') != {'path': str(route_path), 'size_bytes': route_path.stat().st_size, 'blake3': g['b3'](route_path)}:
        raise ValueError('graph cold-route binding mismatch')
    plan_path = cohort / 'source-program-preparation-input-plan.json'
    if selected.get('plan') != {'path': str(plan_path), 'size_bytes': plan_path.stat().st_size, 'blake3': g['b3'](plan_path)}:
        raise ValueError('graph plan binding mismatch')
    for name in ('src/source_built_mantle_programs_graph.rs', 'src/source_built_mantle_programs.rs',
                 'src/source_built_mantle_programs_metadata_helper.rs', 'src/source_built_recipe.rs'):
        path = source / name
        claim = report.get('consumer_blake3', {}).get(name)
        if claim != {'path': name, 'size_bytes': path.stat().st_size, 'blake3': g['b3'](path)}:
            raise ValueError(f'graph frozen consumer binding mismatch: {name}')
    admitted = {x['role']: x for x in subject['source_inputs']}
    vendor_root = g['source_role_root']('vendor-inputs')
    manifest_path = g['checked_source_path'](vendor_root, 'nickel-lang-parser/Cargo.toml')
    grammar_path = g['checked_source_path'](vendor_root, 'nickel-lang-parser/src/grammar.rs')
    manifest = tomllib.loads(manifest_path.read_text())
    package = manifest.get('package', {})
    package_owned = package.get('name') == 'nickel-lang-parser' and package.get('build') == 'build.rs'
    def source_member(role, member):
        if role not in admitted or admitted[role]['kind'] != 'directory' or not g['normal_rel'](member, False):
            return False
        try:
            path = g['checked_source_path'](g['source_role_root'](role), member)
            return path.is_file() and ((role, member) != ('vendor-inputs', 'nickel-lang-parser/src/grammar.rs') or package_owned)
        except (OSError, ValueError):
            return False
    plan_members = {stage: f'bootstrap/source-built/{stage}.unit-plan.json' for stage in STAGES}
    source_root = cohort / 'inputs/mantle-source'
    present = {stage: (source_root / member).is_file() and not (source_root / member).is_symlink()
               for stage, member in plan_members.items()}
    if all(present.values()):
        bindings = member_record_bindings(cohort, subject, list(plan_members.values()), g['b3'], subject['inert_tools']['b3sum'])
        plan_documents = {stage: json.loads((source_root / member).read_bytes()) for stage, member in plan_members.items()}
        facts, derivation_evidence = unit_plan_facts(plan_documents, recipes)
    else:
        bindings = {}
        facts, derivation_evidence = unit_plan_facts({}, recipes)
        for stage, available in present.items():
            if not available:
                derivation_evidence[stage]['uncovered'] = ['authenticated source-only unit plan member absent']
    consumer_names = ('src/source_built_mantle_programs_graph.rs', 'src/source_built_mantle_programs.rs',
                      'src/source_built_mantle_programs_metadata.rs', 'src/source_built_mantle_programs_metadata_helper.rs',
                      'src/rust_plan.rs', 'crates/mantle-rust-plan-core/src/features.rs', 'src/source_built_recipe.rs')
    consumers = {name: {'path': name, 'size_bytes': (source / name).stat().st_size, 'blake3': g['b3'](source / name)}
                 for name in consumer_names}
    result = audit_data(recipes, unit_facts=facts, source_member=source_member)
    for stage in STAGES:
        missing = derivation_evidence[stage].get('uncovered_count', len(derivation_evidence[stage].get('uncovered', [])))
        if missing:
            result['bounded_verdict'] = False
            result['per_stage'][stage]['source_plan_uncovered'] = missing
        for bucket in (result['coverage']['feature_cfg'], result['per_stage'][stage]['coverage']['feature_cfg']):
            bucket['total'] += derivation_evidence[stage].get('orphan_derivations', 0)
            bucket['uncovered'] += derivation_evidence[stage].get('orphan_derivations', 0)
        if derivation_evidence[stage].get('orphan_derivations', 0) and len(result['diagnostics']) < LIMIT:
            result['diagnostics'].append({'stage': stage, 'category': 'feature_cfg', 'identity': 'source-unit-graph',
                                          'reason': 'source-planned derivations lack owning recipe compiler',
                                          'count': derivation_evidence[stage]['orphan_derivations']})
    result['diagnostics_suppressed'] = sum(bucket['invalid'] + bucket['uncovered'] for bucket in result['coverage'].values()) - len(result['diagnostics'])
    result['fact_provenance'] = 'authenticated source-profile Mantle role member; derivation.args precede recipe projection'
    result.update({'scope': 'Inert current-data static selectors only; no runtime-generated bytes, recipe/product/helper/provider/compiler/controller execution or proof.',
                   'subject_binding': {'path': str(subject_path), 'size_bytes': subject_path.stat().st_size, 'blake3': observed_subject},
                   'inputs': {'graph_report': {'path': str(graph_path), 'size_bytes': len(raw_graph), 'blake3': g['b3'](graph_path)},
                              'plan': selected['plan'], 'route': selected['cold_route'], 'recipes': inputs,
                              'source_profile': {'path': subject['profile_file'], 'size_bytes': subject['profile_file_bytes'],
                                                 'subject_raw_blake3': subject['profile_file_raw_blake3'],
                                                 'raw_blake3_verified': bool(bindings)},
                              'authenticated_unit_plans': bindings,
                              'source_members': {'vendor-inputs/nickel-lang-parser/Cargo.toml': {'path': str(manifest_path), 'size_bytes': manifest_path.stat().st_size, 'blake3': g['b3'](manifest_path), 'package_name': package.get('name'), 'build': package.get('build')},
                                                 'vendor-inputs/nickel-lang-parser/src/grammar.rs': {'path': str(grammar_path), 'size_bytes': grammar_path.stat().st_size, 'blake3': g['b3'](grammar_path), 'admitted_source_digest_blake3': admitted['vendor-inputs']['digest_blake3'], 'present': source_member('vendor-inputs', 'nickel-lang-parser/src/grammar.rs')}}},
                   'source_plan_derivations': derivation_evidence,
                   'consumer_blake3': consumers,
                   'inspector_blake3': g['b3'](Path(__file__)), 'graph_inspector_blake3': g['b3'](helper_path),
                   'commands': ['python3 mantle-selector-audit.py COHORT FROZEN_SOURCE REVIEW_SCRATCH SUBJECT_JSON', 'b3sum --no-names <bound inputs>'],
                   'limitations': ['Runtime-produced invocation bytes are absent; only static symbolic operands are inspected.']})
    out = scratch / 'mantle-selector-audit.json'
    with out.open('x') as stream:
        json.dump(result, stream, indent=2, sort_keys=True)
        stream.write('\n')
    print(json.dumps({'bounded_verdict': result['bounded_verdict'], 'coverage': result['coverage'], 'diagnostics': result['diagnostics'][:LIMIT]}, sort_keys=True))
    if not result['bounded_verdict']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
