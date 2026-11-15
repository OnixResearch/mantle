from pathlib import Path
p = Path('/home/brittonr/git/OnixResearch/mantle/.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-71c2bd4d/identity-review/identity-audit.py')
s = p.read_text()
def rep(old, new, count=1):
    global s
    if s.count(old) != count:
        raise SystemExit(f'expected {count} occurrences, found {s.count(old)}: {old[:140]!r}')
    s = s.replace(old, new, count)
rep('import hashlib\n', '')
old = """def fingerprint(value) -> str:
    h = hashlib.sha256()
    encoder = json.JSONEncoder(ensure_ascii=False, sort_keys=True, separators=(',', ':'))
    for part in encoder.iterencode(value):
        h.update(part.encode('utf-8'))
    return h.hexdigest()
"""
new = """def fingerprint(value) -> str:
    p = subprocess.Popen([B3, '--no-names'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert p.stdin is not None
    encoder = json.JSONEncoder(ensure_ascii=False, sort_keys=True, separators=(',', ':'))
    for part in encoder.iterencode(value):
        p.stdin.write(part.encode('utf-8'))
    out, err = p.communicate(timeout=3600)
    require(p.returncode == 0, f'canonical body BLAKE3 failed: {err.decode(\"utf-8\",\"replace\")[-1000:]}')
    digest = out.decode().strip()
    require(bool(re.fullmatch(r'[0-9a-f]{64}', digest)), f'invalid canonical body BLAKE3: {digest!r}')
    return digest
"""
rep(old, new)
rep("canonical_body_sha256", "canonical_body_blake3")
rep("record_body_sha256", "record_body_blake3")
rep("""    for entry in record['files']:
        require(entry['file_type'] == 'regular' and entry.get('symlink_target') is None,
""", """    require(record['kind'] == 'fixed-url', f'{identity} program record is not fixed-url: {record[\"kind\"]}')
    require(meta.get('hash_algo') == 'blake3' and meta.get('hash_mode') == 'recursive' and meta.get('unpack') == '1',
            f'{identity} source archive metadata does not declare recursive BLAKE3 unpack identity')
    for entry in record['files']:
        require(entry['file_type'] == 'regular' and entry.get('symlink_target') is None,
""")
rep("""            require(isinstance(package_ref, dict) and package_ref.get('kind') in ('produced', 'source', 'manifest-source'),
                    f'Ring authenticator package is not source authority: {step[\"id\"]}: {package_ref}')
""", """            require(package_ref == {'kind': 'source', 'role': 'vendor-inputs', 'path': 'ring'},
                    f'Ring authenticator is not routed through the original vendor Ring tree: {step[\"id\"]}: {package_ref}')
""")
rep("""    original_raw = b3_file(ORIGINAL_CLOSURE)
    require(original_raw == plan_native['digest_blake3'] and ORIGINAL_CLOSURE.stat().st_size == plan_native['size_bytes'],
""", """    require(ORIGINAL_CLOSURE.is_file() and not ORIGINAL_CLOSURE.is_symlink()
            and AUGMENTED_CLOSURE.is_file() and not AUGMENTED_CLOSURE.is_symlink(),
            'preauthor/augmented native-source closure is missing, nonregular, or symlinked')
    original_raw = b3_file(ORIGINAL_CLOSURE)
    require(original_raw == plan_native['digest_blake3'] and ORIGINAL_CLOSURE.stat().st_size == plan_native['size_bytes'],
""")
rep("""            leaf = root / rel
            observed = b3_file(leaf)
            require(observed == role['digest_blake3'] and leaf.stat().st_size == role['size_bytes'],
""", """            leaf = root / rel
            leaf_stat = os.lstat(leaf)
            require(stat.S_ISREG(leaf_stat.st_mode), f'unmutated preauthor file role is not a regular file: {role_id}')
            observed = b3_file(leaf)
            require(observed == role['digest_blake3'] and leaf_stat.st_size == role['size_bytes'],
""")
p.write_text(s)
print('switched native body comparisons to BLAKE3 and bound Ring/file-role routing')
