from pathlib import Path
p = Path('/home/brittonr/git/OnixResearch/mantle/.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-71c2bd4d/identity-review/identity-audit.py')
s = p.read_text()
def rep(old, new, count=1):
    global s
    if s.count(old) != count:
        raise SystemExit(f'expected {count} occurrences, found {s.count(old)}: {old[:120]!r}')
    s = s.replace(old, new, count)
rep("PROFILE_MANIFEST_B3 = 'c22381a887c2d1ebb9105e7c4e9c5a2a010779c64355d882a3d80da932071c9c'\nPROFILE_RAW_B3", "PROFILE_MANIFEST_B3 = 'c22381a887c2d1ebb9105e7c4e9c5a2a010779c64355d882a3d80da932071c9c'\nINPUT_PROFILE_B3 = '6e3a630fbc19fe7038ed46b6871a102ca8a9d86aa2ca16196b0eb5866d86f723'\nREPORT_RAW_B3 = '55588f77ead28264195490a2aec50a57f6dad9863d69194389f11ced47df296b'\nPLAN_DIGEST_CONTEXT = 'mantle-source-built-fixed-point-plan-v2'\nPROFILE_RAW_B3")
rep("def b3_file(path: Path) -> str:\n", "def b3_file(path: Path) -> str:\n")
anchor = """def source_record_content_blake3(record: dict) -> str:
    p = subprocess.Popen([B3, '--no-names'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert p.stdin is not None
    def send(data: bytes) -> None:
        p.stdin.write(data)
    send(b'kind\\0')
    send(json.dumps(record['kind'], ensure_ascii=False, separators=(',', ':')).encode())
    send(b'\\nmetadata\\0')
    send(json.dumps(record.get('metadata', {}), ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode())
    send(b'\\nfiles\\0')
    for entry in record['files']:
        send(json.dumps(entry, ensure_ascii=False, separators=(',', ':')).encode())
        send(b'\\n')
    out, err = p.communicate(timeout=3600)
    require(p.returncode == 0, f'canonical content hash failed: {err.decode("utf-8","replace")[-1000:]}')
    return out.decode().strip()
"""
insert = anchor + """
def canonical_plan_digest(plan: dict) -> str:
    payload = dict(plan)
    payload['plan_digest_blake3'] = ''
    encoded = json.dumps(payload, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
    out = run([B3, '--derive-key', PLAN_DIGEST_CONTEXT, '--no-names'], input_bytes=encoded).decode().strip()
    require(bool(re.fullmatch(r'[0-9a-f]{64}', out)), f'invalid canonical plan digest output: {out!r}')
    return out
"""
rep(anchor, insert)
rep("                actual = f.read(len(data))\n                require(actual == data, f'decoded embedded payload bytes differ from physical role {record[\"identity\"]}/{rel}:{idx}')", "                require(b3_bytes(data) == entry['blake3'],\n                        f'embedded role chunk BLAKE3 differs from record {record[\"identity\"]}/{rel}:{idx}')\n                actual = f.read(len(data))\n                require(actual == data, f'decoded embedded payload bytes differ from physical role {record[\"identity\"]}/{rel}:{idx}')")
rep("    require(report['schema'] == 'mantle-source-built-source-programs-v1', 'actual preparation report schema mismatch')\n    require(report['source_profile'] == str(PROFILE_PATH), 'actual report source-profile path mismatch')\n    require(report['output_profile_blake3'] == PROFILE_MANIFEST_B3, 'actual preparation profile identity mismatch')", "    report_raw = b3_file(REPORT_PATH)\n    require(report_raw == REPORT_RAW_B3, f'actual source-program report raw BLAKE3 differs from completed receipt: {report_raw}')\n    require(report['schema'] == 'mantle-source-built-source-programs-v1', 'actual preparation report schema mismatch')\n    require(report['input_profile_blake3'] == INPUT_PROFILE_B3, 'actual report input-profile identity mismatch')\n    require(report['source_profile'] == str(PROFILE_PATH), 'actual report source-profile path mismatch')\n    require(report['output_profile_blake3'] == PROFILE_MANIFEST_B3, 'actual preparation profile identity mismatch')")
rep("    require(plan['plan_digest_blake3'] == '1e74d77c263bcd6d6d781683e99c9664b1551486fc5fd3da5c1eaaa815456d3a', 'unexpected actual input plan digest')", "    require(plan['plan_digest_blake3'] == '1e74d77c263bcd6d6d781683e99c9664b1551486fc5fd3da5c1eaaa815456d3a', 'unexpected actual input plan digest')\n    canonical_plan = canonical_plan_digest(plan)\n    require(canonical_plan == plan['plan_digest_blake3'],\n            f'actual canonical input plan digest mismatch: computed={canonical_plan} recorded={plan[\"plan_digest_blake3\"]}')")
rep("    require(set(plan['source_inputs'][i]['role'] for i in range(len(plan['source_inputs']))) == set(x['id'] for x in plan['source_inputs']),\n            'source plan role IDs differ from expected identity IDs')\n    require(len(plan['source_inputs']) == 7, f'actual source plan role count {len(plan[\"source_inputs\"])} != 7')", "    expected_plan_roles = {'mantle-source', 'native-source-bundle', 'rust-source-archive-set',\n                           'stagex-lineage', 'stagex-seed', 'stagex-source-bundle', 'vendor-inputs'}\n    observed_plan_roles = {x['role'] for x in plan['source_inputs']}\n    require(len(plan['source_inputs']) == 7 and observed_plan_roles == expected_plan_roles,\n            f'actual source plan roles differ from the seven expected roles: {sorted(observed_plan_roles)}')\n    require(observed_plan_roles == {x['id'] for x in plan['source_inputs']},\n            'source plan role IDs differ from identity IDs')")
rep("    require(report['program_records'] == report['program_records'] and set(report['program_records']) == PROGRAM_IDS, 'actual report does not list exactly four new programs')", "    require(set(report['program_records']) == PROGRAM_IDS, 'actual report does not list exactly four new programs')") if "require(report['program_records'] == report['program_records']" in s else None
rep("    require(set(report['program_records']) == PROGRAM_IDS, 'actual report does not list exactly four new programs')", "    require(set(report['program_records']) == PROGRAM_IDS, 'actual report does not list exactly four new programs')")
rep("    # Compare complete native source record bodies, not only identity/digest labels.\n", "    # Compare complete native source record bodies, not only identity/digest labels.\n")
rep("    closure_programs = {}\n    for identity in sorted(PROGRAM_IDS):", "    expected_program_entries = PROGRAM_IDS | {f'{identity}.tar' for identity in PROGRAM_IDS}\n    actual_program_entries = {entry.name for entry in PROGRAMS.iterdir()}\n    require(actual_program_entries == expected_program_entries,\n            f'program-sources contains unexpected/missing physical entries: {sorted(actual_program_entries ^ expected_program_entries)}')\n    closure_programs = {}\n    for identity in sorted(PROGRAM_IDS):")
rep("    require(original_step['inputs'].get('aws-helper', {}).get('kind') == 'produced',\n            f'{stage_name} AWS original source helper not Produced')\n    return", "    require(original_step['inputs'].get('aws-helper', {}).get('kind') == 'produced',\n            f'{stage_name} AWS original source helper not Produced')\n    perlasm_source = steps.get('unit-00037-script.perlasm-source')\n    require(perlasm_source is not None, f'{stage_name} lacks AWS perlasm source-tree helper')\n    original_output = original_step['outputs'].get('source')\n    require(original_output is not None, f'{stage_name} AWS original source route lacks a source output')\n    original_tree_ref = produced_ref(perlasm_source, 'aws-original')\n    require(original_tree_ref == {'kind': 'produced', 'action_id': f'{stage_name}/aws-original-source', 'output_id': 'source'},\n            f'{stage_name} perlasm source-tree helper does not consume the routed pinned AWS original: {original_tree_ref}')\n    return")
rep("    aws_scripts = [verify_perlasm_boundary(aws_root / item['script_path']) for item in s1['aws']]\n    ring_scripts", "    aws_scripts = [verify_perlasm_boundary(aws_root / item['script_path']) for item in s1['aws']]\n    selected_aws_paths = {item['script_path'] for item in s1['aws']}\n    expected_probe_relpaths = {f'crypto/fipsmodule/bn/asm/rsaz-{n}-avx512.pl' for n in ('2k', '3k', '4k')}\n    require(expected_probe_relpaths <= selected_aws_paths,\n            f'one or more probed RSaz scripts are not selected in stage1/stage2 recipes: {sorted(expected_probe_relpaths-selected_aws_paths)}')\n    ring_scripts")
rep("    launch_outputs = [proof_out, RUN / 'cold-proof-71c2bd4d-fresh-target.log',\n                      RUN / 'cold-proof-report-71c2bd4d-fresh-target.json',", "    launch_outputs = [proof_out, RUN / 'cold-proof-71c2bd4d-fresh-target.log',\n                      RUN / 'cold-proof-71c2bd4d-fresh-target.exit',\n                      RUN / 'cold-proof-report-71c2bd4d-fresh-target.json',")
rep("        'actual_preparation_report': {'path': str(REPORT_PATH), 'raw_blake3': b3_file(REPORT_PATH),", "        'actual_preparation_report': {'path': str(REPORT_PATH), 'raw_blake3': report_raw,")
p.write_text(s)
print('patched canonical plan, role chunks, exact source routing, and report pins')
