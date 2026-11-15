from pathlib import Path
p = Path('/home/brittonr/git/OnixResearch/mantle/.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-71c2bd4d/identity-review/identity-audit.py')
s = p.read_text()
def rep(a,b,n=1):
    global s
    if s.count(a) < n: raise SystemExit(f'missing patch target ({n}): {a!r}')
    s = s.replace(a,b,n)
rep("'bounded_identity_verdict': 'ACCEPT'}</n    out_path", "'bounded_identity_verdict': 'ACCEPT'}\n    out_path")
rep("Path(meta.get('url', '')).as_uri() == Path(archive).as_uri()", "meta.get('url') == archive.as_uri()")
rep('''                        require(obj['kind'] == expected_kind or (klass == 'bootstrap-source' and obj['kind'] == 'bootstrap-archive'),
                                f'unexpected profile role kind {klass}: {obj["kind"]}')
                        require(obj['identity'] == role_id or klass == 'bootstrap-source',
                                f'role identity/class mismatch {klass}: {identity}')
                        require(obj['kind'] == ('local-path' if expected_kind == 'directory' else obj['kind']) or expected_kind == 'regular-file',
                                f'unexpected role record type {klass}')
''','''                        require(obj['kind'] in {'local-path', 'vcs-snapshot', 'package-mirror', 'bootstrap-archive',
                                                'provider-manifest', 'toolchain-source-root', 'proof-input'},
                                f'unrecognized source record kind {klass}: {obj["kind"]}')
''')
rep("    require(plan['policies']['expected_stagex_lineage_blake3'] if 'expected_stagex_lineage_blake3' in plan['policies'] else LINEAGE_B3,\n            'missing selected lineage pin')", "    lineage_plan = next(x for x in plan['source_inputs'] if x['role'] == 'stagex-lineage')\n    require(lineage_plan['digest_blake3'] == LINEAGE_B3, 'actual source plan lineage identity differs from independent pin')")
rep("for member in ('source-built/rust/program.json', 'cairn/rust/compiler-environment.rs',\n                   'cairn/rust/mrustc-native-translation.c'):", "for member in ('source-built/rust/program.json', 'cairn/rust/compiler-environment.rs'):")
rep("        proc.stdin.close()\n        out, err = proc.communicate(timeout=3600)", "        out, err = proc.communicate(timeout=3600)")
rep("    p.stdin.close()\n    out, err = p.communicate(timeout=3600)", "    out, err = p.communicate(timeout=3600)")
rep("    proc.stdin.close()\n    out, err = proc.communicate(timeout=3600)", "    out, err = proc.communicate(timeout=3600)")
rep('MAX_RECORD_BYTES = 12 * 1024**3', 'MAX_RECORD_BYTES = 20 * 1024**3')
rep("require(s1['aws'] == s2['aws'], 'stage1/stage2 AWS original-script selection or source/pipe route differs')\n    require(s1['ring'] == s2['ring'], 'stage1/stage2 Ring original-script/authenticator selection differs')", "aws_projection = lambda rows: [(x['id'], x['script_path'], x['assembly_output']) for x in rows]\n    ring_projection = lambda rows: [(x['id'], x['script_path'], x['assembly_output']) for x in rows]\n    require(aws_projection(s1['aws']) == aws_projection(s2['aws']), 'stage1/stage2 AWS original-script selection differs')\n    require(ring_projection(s1['ring']) == ring_projection(s2['ring']), 'stage1/stage2 Ring original-script/authenticator selection differs')")
rep("            package_ref = produced_ref(step, 'package')", "            package_ref = step['inputs'].get('package')\n            require(isinstance(package_ref, dict) and package_ref.get('kind') in ('produced', 'source', 'manifest-source'),\n                    f'Ring authenticator package is not source authority: {step[\"id\"]}: {package_ref}')")
rep("'package_source_action': package_ref['action_id']", "'package_source_ref': package_ref")
rep("for member in ('source-built/rust/program.json', 'cairn/rust/compiler-environment.rs'):", "for member in ('source-built/rust/program.json', 'cairn/rust/compiler-environment.rs')")
rep("            require(gen and trans and result, f'Ring authenticated product has incomplete generator/translator/result chain {step[\"id\"]}')", """            require(gen and trans and result, f'Ring authenticated product has incomplete generator/translator/result chain {step[\"id\"]}')
            require(gen['operation'].get('kind') == 'exec' and gen['operation'].get('executable_input') == 'perl'
                    and gen['operation'].get('cwd_input') == 'source', f'Ring generator is not an original Perl/source-cwd Exec {gen[\"id\"]}')
            pipe_ref = produced_ref(gen, 'pipe-source')
            require(word_part(gen, 0).get('kind') == 'input' and word_part(gen, 0).get('name') == 'pipe-source'
                    and word_part(gen, 0).get('path') == '', f'Ring generator does not consume pipe-source {gen[\"id\"]}')
            pipe_step_id = prefix + '.pipe-source'
            pipe_step = steps.get(pipe_step_id)
            require(pipe_step is not None and pipe_ref['action_id'] == f'{stage_name}/{pipe_step_id}'
                    and pipe_ref['output_id'] == 'source' and pipe_step['operation'].get('text') == pipe_source,
                    f'Ring generator does not consume exact frozen PIPE_SOURCE product {gen[\"id\"]}')
            pipes.append(pipe_step)""")
rep("            translator = word_part(tr, 0)\n            require(translator.get('kind') == 'input' and translator.get('name') == 'source'\n                    and translator.get('path') == 'crypto/perlasm/x86_64-xlate.pl',\n                    f'AWS translator path changed in {tr[\"id\"]}')", """            translator = word_part(tr, 0)
            require(translator.get('kind') == 'input' and translator.get('name') == 'source'
                    and translator.get('path') == 'crypto/perlasm/x86_64-xlate.pl',
                    f'AWS translator path changed in {tr[\"id\"]}')
            if '/rsaz-' in rel:
                for phase_step in (step, tr):
                    probe_word = phase_step['operation'].get('environment', {}).get('MANTLE_AWS_AS_PROBE')
                    require(probe_word is not None and len(probe_word.get('parts', [])) == 1
                            and probe_word['parts'][0].get('kind') == 'input'
                            and probe_word['parts'][0].get('name') == 'aws-as-probe',
                            f'RSaz source phase does not consume authentic assembler probe {phase_step[\"id\"]}')
                    probe_ref = phase_step['inputs'].get('aws-as-probe')
                    require(probe_ref == {'kind': 'produced', 'action_id': f'{stage_name}/unit-00037-script.assembler-version', 'output_id': 'stderr'},
                            f'RSaz source phase probe product is not the observed assembler result {phase_step[\"id\"]}')""")
rep("            require(gen['inputs'].get('source') == package_ref and trans['inputs'].get('source') == package_ref,\n                    f'Ring generator/translator source package differs from authenticated original package {step[\"id\"]}')", "            require(gen['inputs'].get('source') == package_ref and trans['inputs'].get('source') == package_ref,\n                    f'Ring generator/translator source package differs from authenticated original package {step[\"id\"]}')")
rep("    require(all(stage['pipe_source_count'] == len(stage['aws']) for stage in stage_results),\n            'not every selected AWS source generator consumes one frozen PIPE_SOURCE product')", "    require(all(stage['pipe_source_count'] == len(stage['aws']) + len(stage['ring']) for stage in stage_results),\n            'not every selected AWS/Ring source generator consumes one frozen PIPE_SOURCE product')")
rep("            files = [x for x in current['physical_tree']['counts'].items() if x[0] == 'regular']\n", '')
rep("            require(len(current['physical_tree']['counts']) == 3, 'invalid role tree counts')\n", '')
p.write_text(s)
print('patched', p, 'bytes', p.stat().st_size)
