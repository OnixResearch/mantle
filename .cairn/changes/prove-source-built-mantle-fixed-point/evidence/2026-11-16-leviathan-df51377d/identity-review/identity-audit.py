#!/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14
"""Bounded read-only physical identity audit for a caller-bound fresh cohort."""
from __future__ import annotations
import gc
import json
import os
import re
import resource
import stat
import subprocess
import sys
from pathlib import Path, PurePosixPath

RUN = FROZEN = COHORT = RUNNER = PLAN_PATH = REPORT_PATH = PROFILE_PATH = None
PREAUTHOR_PROFILE_PATH = ORIGINAL_CLOSURE = AUGMENTED_CLOSURE = None
PREAUTHOR_PLAN_CLOSURE = PROGRAMS = AWS_DB = None
AWS_COMMIT = AWS_TREE = AWS_TREE_B3 = None
FROZEN_B3 = RUNNER_B3 = PROFILE_MANIFEST_B3 = INPUT_PROFILE_B3 = None
REPORT_RAW_B3 = PROFILE_RAW_B3 = LINEAGE_B3 = NATIVE_PROVIDER_B3 = None
B3 = JQ = GIT = TAR = NIX = PYTHON = None
SUBJECT = SUBJECT_PATH = SCRATCH = COHORT_NAME = None
PLAN_DIGEST_CONTEXT = 'mantle-source-built-fixed-point-plan-v2'
PROGRAM_IDS = {'fixed-point-native-source-program', 'fixed-point-rust-source-program',
               'source-built-mantle-programs', 'source-built-stagex-programs'}
ROLE_CLASSES = {'mantle-source', 'bootstrap-source', 'proof-input', 'stagex-lineage',
                'stagex-seed', 'stagex-source-bundle', 'vendored-cargo-inputs'}
ROLE_BY_CLASS = {}
PLAN_FILE_RELATIVE = {'stagex-seed': 'hex0-seed', 'stagex-lineage': 'stagex-transition-lineage.json',
                      'stagex-source-bundle': 'stagex-source-closure.json'}
PRODUCT_REVISION = 'df51377d592bf6a3412b436a10e8fc708e8dbe3d'
RUN_ROOT = Path('/home/brittonr/sbfp-20261113-berdKH')
COHORT_ROOT = RUN_ROOT / 'prepared-cohort-df51377d-unit-plans'
SOURCE_ROOT = RUN_ROOT / 'runner-source-df51377d-unit-plans'
ORIGINAL_COMMIT = '02561621ffa4cf17c0c4f70bc11a82df36b42ae9'
ORIGINAL_TREE = '969d75db5c0494de4a26bbb2dcbf6837058054c1'
ORIGINAL_TREE_B3 = '8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89'
MAX_RECORD_BYTES = 20 * 1024**3
MAX_MEM_BYTES = 120 * 1024**3
MAX_SOURCE_RECORDS = 0

# Constrain accidental whole-manifest or source-payload allocations.  The
# largest selected record is expected below this explicit ceiling.
resource.setrlimit(resource.RLIMIT_AS, (MAX_MEM_BYTES, MAX_MEM_BYTES))

class AuditError(RuntimeError):
    pass

def require(cond: bool, message: str) -> None:
    if not cond:
        raise AuditError(message)
def configure() -> None:
    global RUN, FROZEN, COHORT, RUNNER, PLAN_PATH, REPORT_PATH, PROFILE_PATH
    global PREAUTHOR_PROFILE_PATH, ORIGINAL_CLOSURE, AUGMENTED_CLOSURE, PREAUTHOR_PLAN_CLOSURE, PROGRAMS
    global AWS_DB, AWS_COMMIT, AWS_TREE, AWS_TREE_B3, FROZEN_B3, RUNNER_B3
    global PROFILE_MANIFEST_B3, INPUT_PROFILE_B3, REPORT_RAW_B3, PROFILE_RAW_B3
    global LINEAGE_B3, NATIVE_PROVIDER_B3, B3, JQ, GIT, TAR, NIX, PYTHON
    global SUBJECT, SUBJECT_PATH, SCRATCH, COHORT_NAME, ROLE_BY_CLASS, MAX_SOURCE_RECORDS
    require(len(sys.argv) == 5, 'usage: identity-audit.py COHORT FROZEN_SOURCE REVIEW_SCRATCH SUBJECT_JSON')
    COHORT, FROZEN, SCRATCH, SUBJECT_PATH = map(Path, sys.argv[1:])
    require(SUBJECT_PATH.is_file() and not SUBJECT_PATH.is_symlink(), 'subject must be a regular JSON file')
    SUBJECT = json.loads(SUBJECT_PATH.read_bytes(), object_pairs_hook=json_pairs_no_dupes)
    require(str(COHORT) == SUBJECT['cohort'] and str(FROZEN) == SUBJECT['frozen_source'],
            'explicit cohort/source arguments differ from subject bindings')
    require(all(path.is_absolute() for path in (COHORT, FROZEN, SCRATCH, SUBJECT_PATH)), 'absolute review paths required')
    require(SUBJECT['frozen_product_revision'] == PRODUCT_REVISION and COHORT == COHORT_ROOT
            and FROZEN == SOURCE_ROOT and SUBJECT['run'] == str(RUN_ROOT),
            'subject is not the published replacement frozen-source cohort')
    require(not SCRATCH.is_symlink() and not SCRATCH.resolve().is_relative_to(COHORT.resolve())
            and not SCRATCH.resolve().is_relative_to(FROZEN.resolve()), 'scratch must be outside immutable source/cohort')
    SCRATCH.mkdir(parents=True, exist_ok=True)
    require(not (SCRATCH / 'identity-review.json').exists(), 'refusing to replace an earlier physical review')
    RUN = Path(SUBJECT['run'])
    require(COHORT.parent == RUN and COHORT.name.startswith('prepared-cohort-'), 'cohort is outside declared run')
    COHORT_NAME = COHORT.name[len('prepared-cohort-'):]
    require(COHORT_NAME != '', 'cohort name is empty')
    RUNNER = Path(SUBJECT['runner'])
    PLAN_PATH, REPORT_PATH, PROFILE_PATH = (Path(SUBJECT[name]) for name in ('input_plan', 'preparation_report', 'profile_file'))
    require((PLAN_PATH, REPORT_PATH, PROFILE_PATH) == (COHORT / 'source-program-preparation-input-plan.json',
            COHORT / 'source-program-preparation.json', COHORT / 'source-profile.json'), 'subject metadata paths differ from cohort')
    require(all(path.is_file() and not path.is_symlink() for path in (PLAN_PATH, REPORT_PATH, PROFILE_PATH, RUNNER)),
            'subject data/runner paths must be regular files')
    PREAUTHOR_PROFILE_PATH = RUN / f'source-profile-{COHORT_NAME}.json'
    ORIGINAL_CLOSURE = SCRATCH / 'original-native-source-closure.json'
    require(not ORIGINAL_CLOSURE.exists() and not ORIGINAL_CLOSURE.is_symlink(), 'original closure scratch already exists')
    AUGMENTED_CLOSURE = COHORT / 'inputs/native-source-authority/native-source-closure.json'
    PREAUTHOR_PLAN_CLOSURE, PROGRAMS = COHORT / 'native-source-closure.json', COHORT / 'program-sources'
    aws = SUBJECT['original_aws']
    AWS_DB, AWS_COMMIT, AWS_TREE, AWS_TREE_B3 = Path(aws['capture']), aws['commit'], aws['git_tree_sha1'], aws['normalized_tree_blake3']
    require(AWS_DB == RUN_ROOT / 'aws-original-source-02561621.TGISLN/aws-lc.git'
            and (AWS_COMMIT, AWS_TREE, AWS_TREE_B3) == (ORIGINAL_COMMIT, ORIGINAL_TREE, ORIGINAL_TREE_B3),
            'original AWS source differs from unchanged independently frozen pins')
    FROZEN_B3, RUNNER_B3 = SUBJECT['frozen_complete_source_blake3'], SUBJECT['runner_blake3']
    PROFILE_MANIFEST_B3, INPUT_PROFILE_B3 = SUBJECT['prepared_manifest_blake3'], SUBJECT['input_manifest_blake3']
    REPORT_RAW_B3, PROFILE_RAW_B3 = SUBJECT['preparation_report_raw_blake3'], SUBJECT['profile_file_raw_blake3']
    LINEAGE_B3, NATIVE_PROVIDER_B3 = SUBJECT['expected_stagex_lineage_blake3'], SUBJECT['expected_native_provider_blake3']
    require(SUBJECT['profile_transport_equal'] is True and SUBJECT['full_profile_ssh_stream_blake3'] == PROFILE_RAW_B3,
            'complete raw-profile transport parity is not bound')
    B3, PYTHON = SUBJECT['inert_tools']['b3sum'], sys.executable
    require(Path(PYTHON).resolve() == Path(SUBJECT['inert_tools']['python']).resolve(), 'interpreter differs from subject tool')
    JQ, GIT, TAR, NIX = ('/run/current-system/sw/bin/' + name for name in ('jq', 'git', 'tar', 'nix'))
    MAX_SOURCE_RECORDS = SUBJECT['resource_bounds']['source_records_max']
    require(type(MAX_SOURCE_RECORDS) is int and 0 < MAX_SOURCE_RECORDS <= 65_536, 'invalid declared source record bound')
    ROLE_BY_CLASS = {
        'mantle-source': ('mantle-source', COHORT / 'inputs/mantle-source', 'directory'),
        'bootstrap-source': ('native-source-bundle', COHORT / 'inputs/native-source-authority', 'regular-file'),
        'proof-input': ('rust-source-archive-set', COHORT / 'inputs/rust-source-archives', 'directory'),
        'stagex-lineage': ('stagex-lineage', COHORT / 'inputs/stagex-lineage', 'regular-file'),
        'stagex-seed': ('stagex-seed', COHORT / 'inputs/stagex-seed', 'regular-file'),
        'stagex-source-bundle': ('stagex-source-bundle', COHORT / 'inputs/stagex-source-bundle', 'regular-file'),
        'vendored-cargo-inputs': ('vendor-inputs', COHORT / 'inputs/vendor-deps', 'directory'),
    }

def run(cmd: list[str], *, input_bytes: bytes | None = None, timeout: int = 3600) -> bytes:
    p = subprocess.run(cmd, input=input_bytes, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout)
    if p.returncode:
        raise AuditError(f'command failed rc={p.returncode}: {cmd!r}: {p.stderr.decode("utf-8", "replace")[-2000:]}')
    return p.stdout

def b3_file(path: Path) -> str:
    out = run([B3, '--no-names', str(path)]).decode().strip()
    require(bool(re.fullmatch(r'[0-9a-f]{64}', out)), f'invalid b3sum output for {path}: {out!r}')
    return out

def b3_bytes(data: bytes) -> str:
    out = run([B3, '--no-names'], input_bytes=data).decode().strip()
    require(bool(re.fullmatch(r'[0-9a-f]{64}', out)), f'invalid b3sum stdin output: {out!r}')
    return out

def verify_file_blake3(files: list[tuple[Path, str]]) -> None:
    for start in range(0, len(files), 64):
        batch = files[start:start + 64]
        observed = run([B3, '--no-names', *[str(path) for path, _ in batch]]).decode().splitlines()
        require(len(observed) == len(batch), 'physical file BLAKE3 result count mismatch')
        for (path, expected), digest in zip(batch, observed):
            require(digest == expected, f'physical file BLAKE3 differs from decoded source record: {path}')

def normalized_tar_b3(root: Path) -> str:
    cmd = [TAR, '--sort=name', '--mtime=@0', '--owner=0', '--group=0', '--numeric-owner',
           '--format=gnu', '--create', '--file', '-', '--directory', str(root), '.']
    p1 = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert p1.stdout is not None
    p2 = subprocess.Popen([B3, '--no-names'], stdin=p1.stdout, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    p1.stdout.close()
    assert p2.stdout is not None
    out2, err2 = p2.communicate(timeout=3600)
    err1 = p1.stderr.read() if p1.stderr else b''
    rc1 = p1.wait(timeout=3600)
    require(p2.returncode == 0 and rc1 == 0,
            f'normalized GNU tar hash failed for {root}: tar={rc1} b3={p2.returncode} '
            f'{err1.decode("utf-8","replace")[-1000:]} {err2.decode("utf-8","replace")[-1000:]}')
    return out2.decode().strip()

def safe_rel(text: str) -> tuple[str, ...]:
    require(isinstance(text, str) and text and '\x00' not in text and '\\' not in text,
            f'unsafe relative source path {text!r}')
    p = PurePosixPath(text)
    require(not p.is_absolute() and all(c not in ('', '.', '..') for c in p.parts),
            f'unsafe relative source path {text!r}')
    require(p.as_posix() == text, f'noncanonical relative source path {text!r}')
    return p.parts

def ensure_nofollow_parents(root: Path, parts: tuple[str, ...]) -> Path:
    current = root
    for part in parts[:-1]:
        current = current / part
        st = os.lstat(current)
        require(stat.S_ISDIR(st.st_mode) and not stat.S_ISLNK(st.st_mode),
                f'non-directory/symlink parent in materialized role: {current}')
    return current / parts[-1]

def walk_tree(root: Path) -> tuple[list[tuple[str, Path, os.stat_result]], set[str]]:
    require(root.is_dir() and not root.is_symlink(), f'materialized role root absent/not directory: {root}')
    entries: list[tuple[str, Path, os.stat_result]] = []
    dirs_seen: set[str] = set()
    stack = [(root, '')]
    while stack:
        current, relative = stack.pop()
        with os.scandir(current) as it:
            children = sorted(list(it), key=lambda e: e.name, reverse=True)
        for child in children:
            rel = f'{relative}/{child.name}' if relative else child.name
            safe_rel(rel)
            path = Path(child.path)
            st = os.lstat(path)
            if stat.S_ISDIR(st.st_mode):
                dirs_seen.add(rel)
                entries.append((rel, path, st))
                stack.append((path, rel))
            elif stat.S_ISREG(st.st_mode) or stat.S_ISLNK(st.st_mode):
                entries.append((rel, path, st))
            else:
                raise AuditError(f'special file in source tree {path}: mode={st.st_mode:o}')
    entries.sort(key=lambda e: e[0].encode('utf-8'))
    return entries, dirs_seen

def materialized_tree_identity(root: Path) -> dict:
    entries, dirs_seen = walk_tree(root)
    proc = subprocess.Popen([B3, '--no-names'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert proc.stdin is not None
    total = 0
    counts = {'directory': 0, 'regular': 0, 'symlink': 0}
    try:
        for rel, path, st in entries:
            raw = rel.encode('utf-8')
            proc.stdin.write(len(raw).to_bytes(8, 'little'))
            proc.stdin.write(raw)
            proc.stdin.write(st.st_mode.to_bytes(4, 'little'))
            if stat.S_ISDIR(st.st_mode):
                counts['directory'] += 1
                proc.stdin.write(b'dir\0')
            elif stat.S_ISREG(st.st_mode):
                counts['regular'] += 1
                proc.stdin.write(b'file\0')
                proc.stdin.write(st.st_size.to_bytes(8, 'little'))
                total += st.st_size
                with open(path, 'rb', buffering=1024 * 1024) as f:
                    while True:
                        buf = f.read(1024 * 1024)
                        if not buf:
                            break
                        proc.stdin.write(buf)
                now = os.lstat(path)
                require(now.st_size == st.st_size and now.st_mode == st.st_mode,
                        f'source changed during materialized tree hash: {path}')
            else:
                counts['symlink'] += 1
                target = os.readlink(path).encode('utf-8')
                proc.stdin.write(b'symlink\0')
                proc.stdin.write(len(target).to_bytes(8, 'little'))
                proc.stdin.write(target)
        out, err = proc.communicate(timeout=3600)
    except BaseException:
        if proc.poll() is None:
            proc.kill()
        raise
    require(proc.returncode == 0, f'materialized hash b3sum failed: {err.decode("utf-8","replace")}')
    if total == 0:
        total = 1
    return {'digest_blake3': out.decode().strip(), 'payload_file_bytes': total,
            'entry_count': len(entries), 'counts': counts}

def json_pairs_no_dupes(pairs):
    d = {}
    for k, v in pairs:
        if k in d:
            raise AuditError(f'duplicate JSON object key {k!r}')
        d[k] = v
    return d

def fingerprint(value) -> str:
    proc = subprocess.Popen([B3, '--no-names'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert proc.stdin is not None
    encoder = json.JSONEncoder(ensure_ascii=False, sort_keys=True, separators=(',', ':'))
    for part in encoder.iterencode(value):
        proc.stdin.write(part.encode('utf-8'))
    out, err = proc.communicate(timeout=3600)
    require(proc.returncode == 0, f'canonical body BLAKE3 failed: {err.decode("utf-8", "replace")[-1000:]}')
    digest = out.decode().strip()
    require(bool(re.fullmatch(r'[0-9a-f]{64}', digest)), f'invalid canonical body BLAKE3: {digest!r}')
    return digest

def source_record_content_blake3(record: dict) -> str:
    p = subprocess.Popen([B3, '--no-names'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert p.stdin is not None
    def send(data: bytes) -> None:
        p.stdin.write(data)
    metadata = record.get('metadata', {})
    if metadata and not record['files']:
        send(json.dumps([record['kind'], metadata], ensure_ascii=False, separators=(',', ':')).encode())
    elif metadata:
        send(b'kind\0')
        send(json.dumps(record['kind'], ensure_ascii=False, separators=(',', ':')).encode())
        send(b'\nmetadata\0')
        send(json.dumps(metadata, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode())
        send(b'\nfiles\0')
    for entry in record['files']:
        send(json.dumps(entry, ensure_ascii=False, separators=(',', ':')).encode())
        send(b'\n')
    out, err = p.communicate(timeout=3600)
    require(p.returncode == 0, f'canonical content hash failed: {err.decode("utf-8","replace")[-1000:]}')
    return out.decode().strip()

def canonical_plan_digest(plan: dict) -> str:
    payload = dict(plan)
    payload['plan_digest_blake3'] = ''
    encoded = json.dumps(payload, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
    out = run([B3, '--derive-key', PLAN_DIGEST_CONTEXT, '--no-names'], input_bytes=encoded).decode().strip()
    require(bool(re.fullmatch(r'[0-9a-f]{64}', out)), f'invalid canonical plan digest output: {out!r}')
    return out

def iter_pretty_records(path: Path):
    """Yield one top-level pretty-serialized source record object at a time."""
    in_records = False
    in_record = False
    current = bytearray()
    total = 0
    identities = set()
    with open(path, 'rb', buffering=1024 * 1024) as f:
        for line in f:
            if not in_records:
                if re.match(rb'^\s*"records"\s*:\s*\[\s*$', line.strip()):
                    in_records = True
                continue
            if not in_record:
                if line == b'    {\n' or line == b'    {\r\n':
                    in_record = True
                    current = bytearray(line)
                    total = len(line)
                elif line.lstrip().startswith(b']'):
                    return
                continue
            current.extend(line)
            total += len(line)
            require(total <= MAX_RECORD_BYTES, f'source record exceeds bounded parser limit: {path} > {MAX_RECORD_BYTES}')
            if line in (b'    },\n', b'    },\r\n', b'    }\n', b'    }\r\n'):
                raw = current.rstrip()
                if raw.endswith(b','):
                    raw = raw[:-1]
                obj = json.loads(raw, object_pairs_hook=json_pairs_no_dupes)
                require(isinstance(obj, dict) and isinstance(obj.get('identity'), str),
                        f'invalid source record object in {path}')
                require(obj['identity'] not in identities, f'duplicate source record identity in {path}: {obj["identity"]}')
                identities.add(obj['identity'])
                require(len(identities) <= MAX_SOURCE_RECORDS, f'source records exceed declared bound in {path}')
                yield obj
                in_record = False
                current = bytearray()
    require(in_records and not in_record, f'truncated or unrecognized source records in {path}')

def verify_record_payload(record: dict, root: Path) -> dict:
    """Decode each actual content_hex payload and compare its bytes to disk."""
    require(root.is_dir() and not root.is_symlink(), f'physical role root missing: {root}')
    grouped: dict[str, list[dict]] = {}
    payload = 0
    for entry in record['files']:
        parts = safe_rel(entry['path'])
        grouped.setdefault('/'.join(parts), []).append(entry)
        payload += int(entry['size'])
    expected_files = set(grouped)
    entries, actual_dirs = walk_tree(root)
    actual_files = {rel for rel, _, st in entries if not stat.S_ISDIR(st.st_mode)}
    require(actual_files == expected_files,
            f'role inventory differs for {record["identity"]}: missing={len(expected_files-actual_files)} extra={len(actual_files-expected_files)}')
    expected_dirs = set()
    for rel in expected_files:
        p = PurePosixPath(rel)
        for parent in p.parents:
            if parent.as_posix() != '.':
                expected_dirs.add(parent.as_posix())
    require(actual_dirs == expected_dirs,
            f'role directory inventory differs for {record["identity"]}: missing={len(expected_dirs-actual_dirs)} extra={len(actual_dirs-expected_dirs)}')
    chunks_verified = 0
    files_verified = 0
    whole_file_digests = []
    for rel in sorted(grouped, key=lambda s: s.encode('utf-8')):
        parts = safe_rel(rel)
        path = ensure_nofollow_parents(root, parts)
        lst = os.lstat(path)
        items = grouped[rel]
        items.sort(key=lambda e: (-1 if e.get('chunk_index') is None else int(e['chunk_index'])))
        file_type = items[0]['file_type']
        require(all(e['file_type'] == file_type for e in items), f'inconsistent file type chunks for {record["identity"]}/{rel}')
        if file_type == 'symlink':
            require(len(items) == 1 and stat.S_ISLNK(lst.st_mode), f'symlink mismatch {root}/{rel}')
            target = items[0].get('symlink_target')
            require(isinstance(target, str) and os.readlink(path) == target, f'symlink target mismatch {root}/{rel}')
            require(items[0].get('content_hex') is None and items[0]['size'] == 0,
                    f'invalid symlink payload fields {root}/{rel}')
            continue
        require(file_type == 'regular' and stat.S_ISREG(lst.st_mode), f'regular-file mismatch {root}/{rel}')
        exec_bits = (lst.st_mode & 0o111) != 0
        require(all(bool(e['executable']) == exec_bits for e in items), f'executable bit mismatch {root}/{rel}')
        indexes = [e.get('chunk_index') for e in items]
        counts = [e.get('chunk_count') for e in items]
        if indexes == [None] and counts == [None]:
            require(len(items) == 1, f'duplicate unchunked source path {root}/{rel}')
        else:
            require(len(items) >= 2 and all(isinstance(i, int) for i in indexes)
                    and all(isinstance(c, int) for c in counts)
                    and len(set(counts)) == 1 and counts[0] == len(items)
                    and indexes == list(range(len(items))), f'invalid chunk layout {root}/{rel}')
            require(all(int(e['size']) == 67_108_864 for e in items[:-1]), f'noncanonical nonfinal chunk {root}/{rel}')
            require(0 < int(items[-1]['size']) <= 67_108_864, f'invalid final chunk size {root}/{rel}')
            chunks_verified += len(items)
        physical_size = sum(int(e['size']) for e in items)
        require(lst.st_size == physical_size, f'physical size mismatch {root}/{rel}: {lst.st_size}!={physical_size}')
        with open(path, 'rb', buffering=1024 * 1024) as f:
            for idx, entry in enumerate(items):
                content_hex = entry.get('content_hex')
                require(isinstance(content_hex, str) and len(content_hex) == 2 * int(entry['size']),
                        f'missing/invalid embedded payload length {record["identity"]}/{rel}:{idx}')
                require(content_hex == content_hex.lower() and re.fullmatch(r'[0-9a-f]*', content_hex) is not None,
                        f'noncanonical embedded hex payload {record["identity"]}/{rel}:{idx}')
                data = bytes.fromhex(content_hex)
                require(len(data) == int(entry['size']), f'decoded payload size mismatch {record["identity"]}/{rel}:{idx}')
                if entry.get('chunk_index') is None:
                    whole_file_digests.append((path, entry['blake3']))
                else:
                    require(b3_bytes(data) == entry['blake3'],
                            f'embedded role chunk BLAKE3 differs from record {record["identity"]}/{rel}:{idx}')
                actual = f.read(len(data))
                require(actual == data, f'decoded embedded payload bytes differ from physical role {record["identity"]}/{rel}:{idx}')
            require(f.read(1) == b'', f'physical file trailing bytes {root}/{rel}')
        files_verified += 1
    # Each whole file was byte-compared to its decoded payload; batch the same hash obligation.
    verify_file_blake3(whole_file_digests)
    require(payload == int(record['payload_bytes']), f'role payload byte count mismatch {record["identity"]}')
    tree = materialized_tree_identity(root)
    return {'physical_root': str(root), 'content_files': files_verified,
            'content_entries': len(record['files']), 'chunk_entries': chunks_verified,
            'embedded_payload_bytes_compared': payload, 'physical_tree': tree}

def bind_profile_native_record(record: dict, expected: dict, found: dict) -> None:
    """Bind one complete launch-input original body to the verified closure."""
    identity = record['identity']
    require(identity in expected, f'unexpected original native record in output source profile: {identity}')
    require(identity not in found, f'duplicate original native record in output source profile: {identity}')
    body = fingerprint(record)
    require(body == expected[identity]['canonical_body_blake3'],
            f'output profile original native body differs from augmented closure: {identity}')
    found[identity] = {'canonical_body_blake3': body, 'record_content_blake3': record['content_blake3'],
                       'payload_bytes': record['payload_bytes'], 'entry_count': len(record['files'])}

def profile_record_scan(path: Path, plan: dict, report: dict, closure_fingerprints: dict) -> dict:
    """Decode every bounded role, program and original record; field order grants no authority."""
    found_roles: dict[str, dict] = {}
    found_programs: dict[str, dict] = {}
    expected_native = {identity: row for identity, row in closure_fingerprints.items() if identity not in PROGRAM_IDS}
    require(expected_native, 'verified augmented closure contains no original native records')
    found_native: dict[str, dict] = {}
    with open(path, 'rb', buffering=1024 * 1024) as source:
        source.seek(0, os.SEEK_END)
        tail_bytes = min(source.tell(), 65536)
        source.seek(-tail_bytes, os.SEEK_END)
        tail = source.read(tail_bytes)
    match = re.search(rb'^  "manifest_blake3"\s*:\s*"([0-9a-f]{64})"\s*,?\s*$', tail, re.M)
    require(match is not None, 'current profile lacks its bounded root manifest footer')
    root_manifest = match.group(1).decode()
    total_records = 0
    for obj in iter_pretty_records(path):
        total_records += 1
        klass = obj.get('metadata', {}).get('bootstrap_profile_class')
        require(klass is None or klass in ROLE_CLASSES, f'unknown output profile role class {klass!r}')
        identity = obj['identity']
        if klass in ROLE_CLASSES:
            require(klass not in found_roles, f'duplicate current source role class {klass}')
            role_id, root, expected_kind = ROLE_BY_CLASS[klass]
            require(obj['kind'] in {'local-path', 'vcs-snapshot', 'package-mirror', 'bootstrap-archive',
                                    'provider-manifest', 'toolchain-source-root', 'proof-input'},
                    f'unrecognized source record kind {klass}: {obj["kind"]}')
            canon = source_record_content_blake3(obj)
            require(canon == obj['content_blake3'], f'current role canonical record identity mismatch {klass}: {canon}!={obj["content_blake3"]}')
            physical = verify_record_payload(obj, root)
            role_plan = next(entry for entry in plan['source_inputs'] if entry['role'] == role_id)
            found_roles[klass] = {'profile_class': klass, 'record_identity': identity,
                                  'record_kind': obj['kind'], 'record_content_blake3': obj['content_blake3'],
                                  'record_payload_bytes': obj['payload_bytes'], 'preauthor_plan': role_plan,
                                  **physical}
            print(f'profile_role_bound={klass}', flush=True)
        elif identity in PROGRAM_IDS:
            require(identity not in found_programs, f'duplicate program record in output source profile: {identity}')
            body = fingerprint(obj)
            require(body == closure_fingerprints[identity]['canonical_body_blake3'],
                    f'output profile program body differs from augmented native closure: {identity}')
            expected = report['program_records'][identity]
            require(obj['content_blake3'] == expected, f'output profile program record differs from report: {identity}')
            found_programs[identity] = {'record_content_blake3': obj['content_blake3'],
                                        'record_body_blake3': body, 'payload_bytes': obj['payload_bytes'],
                                        'entry_count': len(obj['files'])}
        else:
            bind_profile_native_record(obj, expected_native, found_native)
        del obj
        gc.collect()
    require(root_manifest == report['output_profile_blake3'] == PROFILE_MANIFEST_B3,
            f'source profile canonical manifest identity mismatch {root_manifest}')
    require(set(found_roles) == ROLE_CLASSES, f'current source role profile classes incomplete: {ROLE_CLASSES-set(found_roles)}')
    require(set(found_programs) == PROGRAM_IDS, f'current profile lacks one or more of the four program records: {PROGRAM_IDS-set(found_programs)}')
    require(set(found_native) == set(expected_native),
            f'output profile original native identities differ: missing={sorted(set(expected_native)-set(found_native))}')
    require(total_records == len(found_roles) + len(found_programs) + len(found_native),
            'output profile record coverage is incomplete')
    return {'manifest_blake3': root_manifest, 'role_records': found_roles,
            'program_records': found_programs, 'original_native_records': found_native,
            'target_records_verified': len(found_roles) + len(found_programs) + len(found_native),
            'total_records_decoded': total_records}

def closure_records(path: Path) -> dict:
    result = {}
    for record in iter_pretty_records(path):
        identity = record['identity']
        require(identity not in result, f'duplicate native closure record identity in {path}: {identity}')
        fp = fingerprint(record)
        item = {'canonical_body_blake3': fp, 'kind': record['kind'],
                'content_blake3': record['content_blake3'], 'payload_bytes': record['payload_bytes'],
                'entry_count': len(record['files']), 'metadata': record.get('metadata', {})}
        if identity in PROGRAM_IDS:
            item['record'] = record
        result[identity] = item
        del record
        gc.collect()
    require(result, f'no source records parsed from {path}')
    return result

def preauthor_native_profile_records(path: Path, native_authority_path: Path) -> tuple[dict, dict]:
    """Decode the current input's bootstrap authority only into owned inert scratch."""
    require(path.is_file() and not path.is_symlink(), f'preauthor input profile missing/nonregular: {path}')
    with path.open('rb') as source:
        source.seek(max(0, path.stat().st_size - 65_536))
        tail = source.read()
    match = re.search(rb'^  "manifest_blake3"\s*:\s*"([0-9a-f]{64})"\s*,?\s*$', tail, re.M)
    require(match is not None and match.group(1).decode() == INPUT_PROFILE_B3, 'preauthor input manifest mismatch')
    raw_profile = b3_file(path)
    role_classes, native = {}, {}
    total = 0
    for record in iter_pretty_records(path):
        total += 1
        identity = record['identity']
        klass = record.get('metadata', {}).get('bootstrap_profile_class')
        require(klass is None or klass in ROLE_CLASSES, f'unknown input profile role class {klass!r}')
        if klass is not None:
            require(klass in ROLE_CLASSES and klass not in role_classes, f'unknown/duplicate preauthor role class {klass}')
            role_classes[klass] = {'profile_class': klass, 'record_identity': identity, 'record_kind': record['kind'],
                                   'record_content_blake3': record['content_blake3'],
                                   'record_payload_bytes': record['payload_bytes'], 'entry_count': len(record['files'])}
            if klass == 'bootstrap-source':
                require(record['kind'] == 'bootstrap-archive' and record['metadata'] == {
                    'bootstrap_profile_class': 'bootstrap-source', 'bootstrap_profile_mode': 'source-built-fixed-point'},
                    'preauthor native authority kind/metadata mismatch')
                require(source_record_content_blake3(record) == record['content_blake3'], 'preauthor authority canonical identity mismatch')
                entries = record['files']
                require(bool(entries) and all(entry['path'] == 'native-source-closure.json'
                        and entry['file_type'] == 'regular' and entry['executable'] is False for entry in entries),
                        'preauthor native authority file inventory mismatch')
                if len(entries) == 1:
                    require(entries[0].get('chunk_index') is None and entries[0].get('chunk_count') is None,
                            'noncanonical single-file chunk markers')
                else:
                    require(all(entry.get('chunk_index') == index and entry.get('chunk_count') == len(entries)
                            for index, entry in enumerate(entries)), 'preauthor native authority chunk sequence mismatch')
                    require(all(int(entry['size']) == 67_108_864 for entry in entries[:-1])
                            and 0 < int(entries[-1]['size']) <= 67_108_864, 'noncanonical native authority chunk geometry')
                payload = 0
                with native_authority_path.open('xb', buffering=1024 * 1024) as extracted:
                    for index, entry in enumerate(entries):
                        text = entry.get('content_hex')
                        require(isinstance(text, str) and text == text.lower() and len(text) == 2 * int(entry['size'])
                                and re.fullmatch(r'[0-9a-f]*', text) is not None, f'malformed native authority payload {index}')
                        data = bytes.fromhex(text)
                        require(b3_bytes(data) == entry['blake3'], f'native authority chunk identity mismatch {index}')
                        extracted.write(data)
                        payload += len(data)
                require(payload == record['payload_bytes'] == native_authority_path.stat().st_size,
                        'preauthor native authority payload size mismatch')
                role_classes[klass].update({'physical_file': str(native_authority_path),
                    'physical_file_blake3': b3_file(native_authority_path), 'physical_file_bytes': payload,
                    'chunks_verified': len(entries), 'embedded_payload_bytes_compared': payload})
        else:
            require(identity not in native and record['files'], f'duplicate/empty native record {identity}')
            native[identity] = {'canonical_body_blake3': fingerprint(record), 'kind': record['kind'],
                                'content_blake3': record['content_blake3'], 'payload_bytes': record['payload_bytes'],
                                'entry_count': len(record['files'])}
        del record
        gc.collect()
    require(set(role_classes) == ROLE_CLASSES, f'preauthor role classes incomplete: {ROLE_CLASSES-set(role_classes)}')
    require(total == len(role_classes) + len(native), 'preauthor record count is inconsistent')
    require(native and not (set(native) & PROGRAM_IDS), 'missing original records or preexisting program authority')
    return native, {'path': str(path), 'bytes': path.stat().st_size, 'raw_blake3': raw_profile,
                    'manifest_blake3': INPUT_PROFILE_B3, 'total_records': total,
                    'profile_role_record_count': len(role_classes), 'native_source_record_count': len(native),
                    'profile_role_records': role_classes, 'bootstrap_source_role': role_classes['bootstrap-source']}

def verify_program_record(identity: str, record: dict, report_digest: str, report: dict) -> dict:
    require(record['content_blake3'] == report_digest, f'{identity} current record content identity differs from report')
    canonical = source_record_content_blake3(record)
    require(canonical == record['content_blake3'], f'{identity} canonical content identity mismatch: {canonical}!={record["content_blake3"]}')
    meta = record.get('metadata', {})
    archive = PROGRAMS / f'{identity}.tar'
    root = PROGRAMS / identity
    require(meta.get('url') == archive.as_uri(), f'{identity} archive URL not exact file path: {meta.get("url")}')
    require(archive.is_file() and not archive.is_symlink() and root.is_dir() and not root.is_symlink(),
            f'{identity} archive or source directory missing/not regular')
    require(record['kind'] == 'fixed-url', f'{identity} program record is not fixed-url: {record["kind"]}')
    require(meta.get('hash_algo') == 'blake3' and meta.get('hash_mode') == 'recursive' and meta.get('unpack') == '1',
            f'{identity} source archive metadata does not declare recursive BLAKE3 unpack identity')
    expected_files: dict[str, list[dict]] = {}
    payload = 0
    for entry in record['files']:
        require(entry['file_type'] == 'regular' and entry.get('symlink_target') is None,
                f'{identity} program record contains nonregular member {entry["path"]}')
        parts = safe_rel(entry['path'])
        expected_files.setdefault('/'.join(parts), []).append(entry)
        payload += int(entry['size'])
    require(payload == record['payload_bytes'], f'{identity} record payload byte total mismatch')
    entries, actual_dirs = walk_tree(root)
    actual_files = {rel for rel, _, st in entries if stat.S_ISREG(st.st_mode)}
    require(actual_files == set(expected_files),
            f'{identity} physical file inventory mismatch: missing={len(set(expected_files)-actual_files)} extra={len(actual_files-set(expected_files))}')
    expected_dirs = set()
    for rel in expected_files:
        for parent in PurePosixPath(rel).parents:
            if parent.as_posix() != '.':
                expected_dirs.add(parent.as_posix())
    require(actual_dirs == expected_dirs, f'{identity} physical directory inventory mismatch')
    chunk_entries = 0
    files_checked = 0
    for rel in sorted(expected_files, key=lambda s: s.encode('utf-8')):
        parts = safe_rel(rel)
        path = ensure_nofollow_parents(root, parts)
        st = os.lstat(path)
        require(stat.S_ISREG(st.st_mode) and stat.S_IMODE(st.st_mode) == 0o644,
                f'{identity} noncanonical program file mode {path}: {st.st_mode:o}')
        items = expected_files[rel]
        items.sort(key=lambda e: -1 if e.get('chunk_index') is None else int(e['chunk_index']))
        require(all(e.get('content_hex') is not None and e.get('executable') is False for e in items),
                f'{identity} file entry lacks exact embedded payload/nonexecutable mode {rel}')
        indices = [e.get('chunk_index') for e in items]
        counts = [e.get('chunk_count') for e in items]
        if indices == [None] and counts == [None]:
            require(len(items) == 1, f'{identity} duplicate unchunked member {rel}')
        else:
            require(len(items) >= 2 and indices == list(range(len(items))) and len(set(counts)) == 1
                    and counts[0] == len(items), f'{identity} invalid physical chunk layout {rel}')
            require(all(int(e['size']) == 67_108_864 for e in items[:-1]), f'{identity} noncanonical full chunk {rel}')
            require(0 < int(items[-1]['size']) <= 67_108_864, f'{identity} invalid final chunk {rel}')
            chunk_entries += len(items)
        require(sum(int(e['size']) for e in items) == st.st_size,
                f'{identity} physical file size mismatch {rel}: record={sum(int(e["size"]) for e in items)} actual={st.st_size}')
        with open(path, 'rb', buffering=1024 * 1024) as physical:
            for index, entry in enumerate(items):
                text = entry['content_hex']
                require(isinstance(text, str) and len(text) == int(entry['size']) * 2
                        and text == text.lower() and re.fullmatch(r'[0-9a-f]*', text) is not None,
                        f'{identity} embedded hex malformed {rel}:{index}')
                payload_bytes = bytes.fromhex(text)
                require(len(payload_bytes) == int(entry['size']), f'{identity} embedded payload decoded size mismatch {rel}:{index}')
                observed = physical.read(len(payload_bytes))
                require(observed == payload_bytes, f'{identity} embedded payload bytes differ from physical file {rel}:{index}')
                require(b3_bytes(payload_bytes) == entry['blake3'], f'{identity} embedded/physical chunk BLAKE3 mismatch {rel}:{index}')
            require(physical.read(1) == b'', f'{identity} physical file trailing bytes {rel}')
        files_checked += 1
    physical_tree = materialized_tree_identity(root)
    nar = run([NIX, '--extra-experimental-features', 'blake3-hashes', 'hash', 'path',
               '--type', 'blake3', '--base16', str(root)]).decode().strip()
    require(nar == meta.get('hash'), f'{identity} recursive NAR mismatch: actual={nar} metadata={meta.get("hash")}')
    # Compare actual archive member inventory and every decompressed member byte
    # directly with its corresponding physical source file, not only aggregate hashes.
    import tarfile
    members = []
    with tarfile.open(archive, mode='r:') as tf:
        seen = set()
        for member in tf:
            require(member.isfile(), f'{identity} archive contains a non-regular member {member.name!r}')
            require(member.name.startswith('source/') and not member.name.startswith('source/../'),
                    f'{identity} unsafe archive member {member.name!r}')
            rel = member.name[len('source/'):]
            safe_rel(rel)
            require(rel not in seen, f'{identity} duplicate archive member {member.name!r}')
            seen.add(rel)
            require(member.uid == 0 and member.gid == 0 and member.mtime == 0 and stat.S_IMODE(member.mode) == 0o644,
                    f'{identity} noncanonical tar metadata for {member.name!r}')
            expected = expected_files.get(rel)
            require(expected is not None and member.size == sum(int(e['size']) for e in expected),
                    f'{identity} archive member/record size mismatch {member.name!r}')
            src = ensure_nofollow_parents(root, safe_rel(rel))
            extracted = tf.extractfile(member)
            require(extracted is not None, f'{identity} cannot read tar member {member.name!r}')
            with open(src, 'rb', buffering=1024 * 1024) as fsrc:
                while True:
                    a = extracted.read(1024 * 1024)
                    b = fsrc.read(1024 * 1024)
                    require(a == b, f'{identity} actual tar member bytes differ from physical file {rel}')
                    if not a:
                        break
                require(fsrc.read(1) == b'', f'{identity} physical file has trailing bytes {rel}')
            members.append(rel)
    require(set(members) == set(expected_files) and len(members) == len(expected_files),
            f'{identity} archive member inventory differs: members={len(members)} record-files={len(expected_files)}')
    actual_archive_b3 = b3_file(archive)
    return {'identity': identity, 'record_content_blake3': record['content_blake3'],
            'canonical_record_content_blake3': canonical, 'report_content_blake3': report_digest,
            'payload_bytes': payload, 'entry_count': len(record['files']), 'unique_files': files_checked,
            'chunk_entries': chunk_entries, 'physical_tree': physical_tree,
            'recursive_nar_blake3': nar, 'record_metadata_nar_blake3': meta.get('hash'),
            'archive_path': str(archive), 'archive_bytes': archive.stat().st_size,
            'archive_raw_blake3': actual_archive_b3, 'archive_member_count': len(members),
            'archive_member_bytes_compared_exactly': True}

def verify_git_tree_copy(label: str, root: Path, git_entries: list[dict]) -> dict:
    require(root.is_dir() and not root.is_symlink(), f'{label} AWS-LC root absent/not directory')
    expected = {e['path']: e for e in git_entries}
    actual, dirs = walk_tree(root)
    actual_files = {rel for rel, _, st in actual if not stat.S_ISDIR(st.st_mode)}
    require(actual_files == set(expected),
            f'{label} AWS-LC Git inventory differs missing={len(set(expected)-actual_files)} extra={len(actual_files-set(expected))}')
    expected_dirs = set()
    for rel in expected:
        for parent in PurePosixPath(rel).parents:
            if parent.as_posix() != '.':
                expected_dirs.add(parent.as_posix())
    require(dirs == expected_dirs, f'{label} AWS-LC directory inventory differs missing={len(expected_dirs-dirs)} extra={len(dirs-expected_dirs)}')
    counts = {'100644': 0, '100755': 0}
    paths = []
    for rel, e in expected.items():
        require(e['type'] == 'blob' and e['mode'] in ('100644', '100755'), f'unexpected AWS-LC Git entry {rel}: {e}')
        path = ensure_nofollow_parents(root, safe_rel(rel))
        st = os.lstat(path)
        require(stat.S_ISREG(st.st_mode) and not stat.S_ISLNK(st.st_mode), f'{label} nonregular AWS-LC Git path {rel}')
        wanted_mode = 0o755 if e['mode'] == '100755' else 0o644
        require(stat.S_IMODE(st.st_mode) == wanted_mode, f'{label} wrong AWS-LC mode {rel}: {stat.S_IMODE(st.st_mode):o}!={wanted_mode:o}')
        counts[e['mode']] += 1
        paths.append((rel, path, e['oid']))
    # One inert Git process hashes all physical member bytes against the blob IDs
    # in the independently pinned commit tree; --no-filters avoids worktree filters.
    for rel, path, _ in paths:
        require('\n' not in str(path) and '\r' not in str(path), f'newline in AWS-LC physical path {path}')
    stdin_paths = b''.join(os.fsencode(path) + b'\n' for _, path, _ in paths)
    out = run([GIT, f'--git-dir={AWS_DB}', 'hash-object', '--no-filters', '--stdin-paths'],
              input_bytes=stdin_paths)
    observed = out.decode().splitlines()
    require(len(observed) == len(paths), f'{label} Git blob hash result count {len(observed)} != {len(paths)}')
    mismatches = [(paths[i][0], observed[i], paths[i][2]) for i in range(len(paths)) if observed[i] != paths[i][2]]
    require(not mismatches, f'{label} AWS-LC Git blob byte mismatch: {mismatches[:3]}')
    tar_b3 = normalized_tar_b3(root)
    require(tar_b3 == AWS_TREE_B3, f'{label} AWS-LC normalized GNU tar identity mismatch: {tar_b3}')
    return {'label': label, 'path': str(root), 'member_count': len(expected),
            'regular_0644': counts['100644'], 'executable_0755': counts['100755'],
            'git_blob_bytes_match': len(paths), 'extra_members': 0,
            'normalized_gnu_tar_blake3': tar_b3}

def word_part(step: dict, index: int) -> dict:
    op = step['operation']
    words = op.get('arguments', [])
    require(index < len(words), f'missing operation argument {index} in {step.get("id")}')
    parts = words[index].get('parts')
    require(isinstance(parts, list) and len(parts) == 1, f'non-atomic current recipe word {step.get("id")} arg={index}')
    return parts[0]

def produced_ref(step: dict, name: str) -> dict:
    ref = step['inputs'].get(name)
    require(isinstance(ref, dict) and ref.get('kind') == 'produced', f'{step.get("id")} input {name} is not Produced: {ref}')
    return ref

def verify_perlasm_boundary(path: Path) -> dict:
    text = path.read_text(encoding='utf-8')
    lines = text.splitlines()
    standalone = 'open OUT,"| \\"$^X\\" \\"$xlate\\" $flavour \\"$output\\"";'
    guarded = 'open OUT,"| \\"$^X\\" \\"$xlate\\" $flavour \\"$output\\""'
    guard_line = 'or die "can\'t call $xlate: $!";'
    accepted = []
    for i, line in enumerate(lines):
        norm = re.sub(r'^open OUT,\s*', 'open OUT,', line)
        norm = re.sub(r'\s+$', '', norm)
        mentions = 'open OUT' in norm and '$xlate' in norm and '$output' in norm
        if norm == standalone:
            accepted.append((i + 1, 'standalone'))
        elif norm == guarded:
            following = lines[i + 1].strip() if i + 1 < len(lines) else ''
            require(following == guard_line, f'guarded original Perl pipe continuation changed at {path}:{i+1}: {following!r}')
            accepted.append((i + 1, 'guarded'))
        elif mentions:
            raise AuditError(f'unsupported original Perl pipe boundary at {path}:{i+1}: {norm!r}')
    require(len(accepted) == 1, f'expected exactly one supported original Perl pipe boundary in {path}; got {accepted}')
    return {'path': str(path), 'boundary_line': accepted[0][0], 'boundary_kind': accepted[0][1],
            'bytes': path.stat().st_size, 'blake3': b3_file(path)}

def inspect_recipe_stage(stage_no: int, pipe_source: str, report: dict) -> dict:
    stage_name = f'mantle-stage{stage_no}'
    recipe_path = COHORT / f'inputs/mantle-source/bootstrap/source-built/{stage_name}.json'
    require(recipe_path.is_file() and not recipe_path.is_symlink(), f'missing selected physical recipe {recipe_path}')
    recipe = json.loads(recipe_path.read_bytes(), object_pairs_hook=json_pairs_no_dupes)
    require(recipe.get('stage_id') == stage_name, f'wrong stage recipe identity in {recipe_path}')
    steps = {step['id']: step for step in recipe['steps']}
    require(len(steps) == len(recipe['steps']), f'duplicate step IDs in {recipe_path}')
    aws_re = re.compile(r'^unit-00037-script\.perlasm-(\d+)\.generate$')
    ring_re = re.compile(r'^unit-00031-script\.sources\.perl-(\d+)\.authenticate$')
    aws = []
    ring = []
    pipes = []
    for step in recipe['steps']:
        m = aws_re.match(step['id'])
        if m:
            op = step['operation']
            require(op.get('kind') == 'exec' and op.get('executable_input') == 'perl' and op.get('cwd_input') == 'source',
                    f'{stage_name} AWS generator is not an original Perl/source-cwd Exec: {step["id"]}')
            original_word, source_word, output_word = word_part(step, 1), word_part(step, 2), word_part(step, 4)
            require(original_word.get('kind') == 'input' and original_word.get('name') == 'original-script', f'bad AWS original-script operand {step["id"]}')
            require(source_word.get('kind') == 'input' and source_word.get('name') == 'source', f'bad AWS source operand {step["id"]}')
            rel = source_word.get('path')
            require(original_word.get('path') == rel, f'AWS source path differs between original-script and cwd tree {step["id"]}')
            filename = output_word.get('text')
            require(output_word.get('kind') == 'literal' and isinstance(filename, str) and filename.endswith('.S') and '/' not in filename,
                    f'bad AWS original assembly output basename {step["id"]}')
            src_ref = produced_ref(step, 'source')
            org_ref = produced_ref(step, 'original-script')
            require(src_ref == org_ref and src_ref['action_id'] == f'{stage_name}/unit-00037-script.perlasm-source',
                    f'AWS generator does not consume the exact authenticated perlasm source tree {step["id"]}')
            pipe_ref = produced_ref(step, 'pipe-source')
            pword = word_part(step, 0)
            require(pword.get('kind') == 'input' and pword.get('name') == 'pipe-source' and pword.get('path') == '',
                    f'AWS generator does not consume its pipe-source product {step["id"]}')
            pipe_step = steps.get(f'{stage_name}/unit-00037-script.perlasm-{m.group(1)}.pipe-source')
            if pipe_step is None:
                pipe_step = steps.get(f'unit-00037-script.perlasm-{m.group(1)}.pipe-source')
            require(pipe_step is not None and pipe_ref['action_id'] == f'{stage_name}/{pipe_step["id"]}'
                    and pipe_ref['output_id'] == 'source' and pipe_step['operation'].get('text') == pipe_source,
                    f'AWS generator does not consume the exact frozen PIPE_SOURCE product {step["id"]}')
            tr = steps.get(f'unit-00037-script.perlasm-{m.group(1)}.translate')
            require(tr is not None and tr['operation'].get('kind') == 'exec'
                    and tr['inputs'].get('source') == src_ref,
                    f'AWS translator fails to consume the original selected source tree {step["id"]}')
            raw_ref = produced_ref(tr, 'raw')
            require(raw_ref['action_id'] == f'{stage_name}/{step["id"]}' and raw_ref['output_id'] == 'raw',
                    f'AWS translator does not consume the generated raw product {step["id"]}')
            translator = word_part(tr, 0)
            require(translator.get('kind') == 'input' and translator.get('name') == 'source'
                    and translator.get('path') == 'crypto/perlasm/x86_64-xlate.pl',
                    f'AWS translator path changed in {tr["id"]}')
            if '/rsaz-' in rel:
                for phase_step in (step, tr):
                    probe_word = phase_step['operation'].get('environment', {}).get('MANTLE_AWS_AS_PROBE')
                    require(probe_word is not None and len(probe_word.get('parts', [])) == 1
                            and probe_word['parts'][0].get('kind') == 'input'
                            and probe_word['parts'][0].get('name') == 'aws-as-probe',
                            f'RSaz source phase does not consume authentic assembler probe {phase_step["id"]}')
                    probe_ref = phase_step['inputs'].get('aws-as-probe')
                    require(probe_ref == {'kind': 'produced', 'action_id': f'{stage_name}/unit-00037-script.assembler-version', 'output_id': 'stderr'},
                            f'RSaz source phase probe product is not the observed assembler result {phase_step["id"]}')
            aws.append({'id': step['id'], 'script_path': rel, 'assembly_output': filename,
                        'source_tree_action': src_ref['action_id'], 'pipe_source_action': pipe_ref['action_id']})
            pipes.append(pipe_step)
        m = ring_re.match(step['id'])
        if m:
            op = step['operation']
            require(op.get('kind') == 'exec' and op.get('executable_input') == 'helper',
                    f'{stage_name} Ring authenticator is not a helper Exec: {step["id"]}')
            args = [word_part(step, i) for i in range(6)]
            require(args[0].get('kind') == 'literal' and args[0].get('text') == '--ring-request',
                    f'Ring authenticator selector changed {step["id"]}')
            require(args[1].get('kind') == 'input' and args[1].get('name') == 'request', f'Ring request product not consumed {step["id"]}')
            require(args[2].get('kind') == 'input' and args[2].get('name') == 'native-perl', f'Ring Perl executable not consumed {step["id"]}')
            require(args[3].get('kind') == 'input' and args[3].get('name') == 'package', f'Ring original package source not consumed {step["id"]}')
            rel, filename = args[3].get('path'), args[4].get('text')
            require(args[4].get('kind') == 'literal' and args[5].get('kind') == 'output'
                    and args[5].get('name') == 'authenticated', f'Ring authenticator output selector changed {step["id"]}')
            package_ref = step['inputs'].get('package')
            require(package_ref == {'kind': 'source', 'role': 'vendor-inputs', 'path': 'ring'},
                    f'Ring authenticator is not routed through the original vendor Ring tree: {step["id"]}: {package_ref}')
            auth_output = step['outputs'].get('authenticated')
            require(auth_output is not None and auth_output.get('kind') == 'text', f'Ring authenticated script is not a Text product {step["id"]}')
            prefix = step['id'][:-len('.authenticate')]
            gen = steps.get(prefix + '.generate')
            trans = steps.get(prefix + '.translate')
            result = steps.get(prefix + '.result')
            require(gen and trans and result, f'Ring authenticated product has incomplete generator/translator/result chain {step["id"]}')
            require(gen['operation'].get('kind') == 'exec' and gen['operation'].get('executable_input') == 'perl'
                    and gen['operation'].get('cwd_input') == 'source', f'Ring generator is not an original Perl/source-cwd Exec {gen["id"]}')
            pipe_ref = produced_ref(gen, 'pipe-source')
            require(word_part(gen, 0).get('kind') == 'input' and word_part(gen, 0).get('name') == 'pipe-source'
                    and word_part(gen, 0).get('path') == '', f'Ring generator does not consume pipe-source {gen["id"]}')
            pipe_step_id = prefix + '.pipe-source'
            pipe_step = steps.get(pipe_step_id)
            require(pipe_step is not None and pipe_ref['action_id'] == f'{stage_name}/{pipe_step_id}'
                    and pipe_ref['output_id'] == 'source' and pipe_step['operation'].get('text') == pipe_source,
                    f'Ring generator does not consume exact frozen PIPE_SOURCE product {gen["id"]}')
            pipes.append(pipe_step)
            auth_ref = gen['inputs'].get('original-script')
            require(auth_ref == {'kind': 'produced', 'action_id': f'{stage_name}/{step["id"]}', 'output_id': 'authenticated'},
                    f'Ring generator does not consume the authenticated source product {step["id"]}')
            require(gen['inputs'].get('source') == package_ref and trans['inputs'].get('source') == package_ref,
                    f'Ring generator/translator source package differs from authenticated original package {step["id"]}')
            require(word_part(gen, 1).get('kind') == 'input' and word_part(gen, 1).get('name') == 'original-script'
                    and word_part(gen, 1).get('path') == '', f'Ring generator consumes wrong authenticated source member {gen["id"]}')
            require(word_part(gen, 2).get('kind') == 'input' and word_part(gen, 2).get('name') == 'source'
                    and word_part(gen, 2).get('path') == rel, f'Ring generator source route changed {gen["id"]}')
            require(word_part(trans, 0).get('kind') == 'input' and word_part(trans, 0).get('name') == 'source'
                    and word_part(trans, 0).get('path') == 'crypto/perlasm/x86_64-xlate.pl',
                    f'Ring translator path changed {trans["id"]}')
            require(trans['inputs'].get('raw') == {'kind': 'produced', 'action_id': f'{stage_name}/{gen["id"]}', 'output_id': 'raw'},
                    f'Ring translator does not consume actual generator raw product {trans["id"]}')
            require(result['inputs'].get('assembly') == {'kind': 'produced', 'action_id': f'{stage_name}/{trans["id"]}', 'output_id': 'assembly'},
                    f'Ring result helper does not consume translated assembly {result["id"]}')
            for phase, phase_step in [('generate', gen), ('translate', trans)]:
                for suffix in ('stdout', 'stderr', 'exit-status'):
                    ref = result['inputs'].get(f'{phase}-{suffix}')
                    require(ref == {'kind': 'produced', 'action_id': f'{stage_name}/{phase_step["id"]}', 'output_id': suffix},
                            f'Ring result helper drops actual {phase} {suffix} product {result["id"]}')
            ring.append({'id': step['id'], 'script_path': rel, 'assembly_output': filename,
                         'authenticator_output': f'{stage_name}/{step["id"]}:authenticated',
                         'generator': gen['id'], 'translator': trans['id'], 'result': result['id'],
                         'package_source_ref': package_ref})
    require(aws and ring, f'{stage_name} selected AWS/Ring physical source recipes missing')
    # The AWS original source root is explicitly selected from the admitted Mantle source.
    original_step = steps.get('aws-original-source')
    require(original_step is not None, f'{stage_name} lacks original AWS source routing step')
    original_ref = original_step['inputs'].get('original')
    require(original_ref == {'kind': 'source', 'role': 'mantle-source',
                             'path': 'bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9'},
            f'{stage_name} AWS original source route is not the pinned physical Mantle member: {original_ref}')
    require(original_step['inputs'].get('aws-helper', {}).get('kind') == 'produced',
            f'{stage_name} AWS original source helper not Produced')
    perlasm_source = steps.get('unit-00037-script.perlasm-source')
    require(perlasm_source is not None, f'{stage_name} lacks AWS perlasm source-tree helper')
    original_output = original_step['outputs'].get('source')
    require(original_output is not None, f'{stage_name} AWS original source route lacks a source output')
    original_tree_ref = produced_ref(perlasm_source, 'aws-original')
    require(original_tree_ref == {'kind': 'produced', 'action_id': f'{stage_name}/aws-original-source', 'output_id': 'source'},
            f'{stage_name} perlasm source-tree helper does not consume the routed pinned AWS original: {original_tree_ref}')
    return {'stage': stage_name, 'recipe_path': str(recipe_path), 'recipe_bytes': recipe_path.stat().st_size,
            'recipe_raw_blake3': b3_file(recipe_path), 'steps': len(steps), 'aws_count': len(aws),
            'ring_count': len(ring), 'aws': sorted(aws, key=lambda x: x['id']),
            'ring': sorted(ring, key=lambda x: x['id']), 'pipe_source_count': len(pipes)}

def verify_selected_sources(stage_results: list[dict]) -> dict:
    s1, s2 = stage_results
    aws_projection = lambda rows: [(x['id'], x['script_path'], x['assembly_output']) for x in rows]
    ring_projection = lambda rows: [(x['id'], x['script_path'], x['assembly_output']) for x in rows]
    require(aws_projection(s1['aws']) == aws_projection(s2['aws']), 'stage1/stage2 AWS original-script selection differs')
    require(ring_projection(s1['ring']) == ring_projection(s2['ring']), 'stage1/stage2 Ring original-script/authenticator selection differs')
    aws_root = COHORT / 'inputs/mantle-source/bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9'
    ring_root = COHORT / 'inputs/vendor-deps/ring'
    aws_scripts = [verify_perlasm_boundary(aws_root / item['script_path']) for item in s1['aws']]
    selected_aws_paths = {item['script_path'] for item in s1['aws']}
    expected_probe_relpaths = {f'crypto/fipsmodule/bn/asm/rsaz-{n}-avx512.pl' for n in ('2k', '3k', '4k')}
    require(expected_probe_relpaths <= selected_aws_paths,
            f'one or more probed RSaz scripts are not selected in stage1/stage2 recipes: {sorted(expected_probe_relpaths-selected_aws_paths)}')
    ring_scripts = [verify_perlasm_boundary(ring_root / item['script_path']) for item in s1['ring']]
    aws_translator = aws_root / 'crypto/perlasm/x86_64-xlate.pl'
    ring_translator = ring_root / 'crypto/perlasm/x86_64-xlate.pl'
    require(aws_translator.is_file() and ring_translator.is_file(), 'selected original AWS/Ring translator missing')
    # Three original AWS scripts contain the single supported assembler probe cut.
    probes = []
    probe_literal = '`$ENV{CC} -Wa,-v -c -o /dev/null -x assembler /dev/null 2>&1`'
    expected_probe_paths = [aws_root / f'crypto/fipsmodule/bn/asm/rsaz-{n}-avx512.pl' for n in ('2k', '3k', '4k')]
    for path in expected_probe_paths:
        text = path.read_text(encoding='utf-8')
        require(text.count(probe_literal) == 1, f'original AWS assembler probe boundary count changed: {path}')
        probes.append({'path': str(path), 'bytes': path.stat().st_size, 'blake3': b3_file(path), 'literal_count': 1})
    # The frozen PIPE_SOURCE is the exact non-executed wrapper source authored into both recipes.
    frozen_native = FROZEN / 'src/source_built_mantle_programs_native.rs'
    native_text = frozen_native.read_text(encoding='utf-8')
    match = re.search(r'const PIPE_SOURCE: &str = r###"(.*?)"###;', native_text, re.S)
    require(match is not None, 'frozen PIPE_SOURCE literal not found')
    pipe_bytes = match.group(1).encode()
    require(all(stage['pipe_source_count'] == len(stage['aws']) + len(stage['ring']) for stage in stage_results),
            'not every selected AWS/Ring source generator consumes one frozen PIPE_SOURCE product')
    return {'stage_selection_equal': True, 'aws_selected_count': len(s1['aws']),
            'ring_selected_count': len(s1['ring']), 'aws_scripts': aws_scripts,
            'ring_scripts': ring_scripts, 'aws_translator': {'path': str(aws_translator), 'bytes': aws_translator.stat().st_size, 'blake3': b3_file(aws_translator)},
            'ring_translator': {'path': str(ring_translator), 'bytes': ring_translator.stat().st_size, 'blake3': b3_file(ring_translator)},
            'aws_probe_boundaries': probes, 'frozen_pipe_source_bytes': len(pipe_bytes),
            'frozen_pipe_source_blake3': b3_bytes(pipe_bytes)}

def frozen_source_capture_policy() -> dict:
    staged_path = FROZEN / 'src/self_build.rs'
    capture_path = FROZEN / 'src/source_bundle.rs'
    staged = staged_path.read_text()
    capture = capture_path.read_text()
    entries = re.findall(r'const STAGED_SOURCE_TOP_LEVEL_ENTRIES: &\[&str\] = &\[(.*?)\];', staged, re.S)
    require(len(entries) == 1, 'frozen staged source allowlist is not unique')
    names = json.loads('[' + re.sub(r',\s*$', '', entries[0]) + ']')
    require(names and len(names) == len(set(names)), 'frozen staged roots empty/duplicated')
    require(all(isinstance(name, str) and '/' not in name and name not in ('', '.', '..') for name in names),
            'frozen staged roots are not single members')
    vendor = re.findall(r'const VENDOR_DEPS_DIR_NAME: &str = "([^"]+)";', capture)
    git = re.findall(r'const DOT_GIT_DIR_NAME: &str = "([^"]+)";', capture)
    require(len(vendor) == len(git) == 1, 'frozen source exclusion constants absent/duplicated')
    require(vendor[0] in names, 'separate vendor role is not a staged root')
    return {'admitted_top_level_entries': sorted(set(names) - {vendor[0]}), 'skipped_git_name': git[0],
            'separate_vendor_root': vendor[0],
            'frozen_policy_blake3': {str(staged_path): b3_file(staged_path), str(capture_path): b3_file(capture_path)}}

def verify_frozen_source_preservation() -> dict:
    policy = frozen_source_capture_policy()
    allowed = set(policy['admitted_top_level_entries'])
    frozen, _ = walk_tree(FROZEN)
    leaves = [entry for entry in frozen if not stat.S_ISDIR(entry[2].st_mode)]
    original = [entry for entry in leaves if entry[0].split('/')[0] in allowed
                and policy['skipped_git_name'] not in entry[0].split('/')]
    require(original, 'frozen source capture has no admitted leaves')
    prepared_root = COHORT / 'inputs/mantle-source'
    prepared, _ = walk_tree(prepared_root)
    current = {rel: (path, metadata) for rel, path, metadata in prepared}
    original_names = set()
    regular_files = 0
    payload_bytes = 0
    for rel, source, original_metadata in original:
        require(rel in current, f'prepared Mantle source omits admitted frozen leaf {rel}')
        original_names.add(rel)
        for parent in PurePosixPath(rel).parents:
            if str(parent) == '.':
                continue
            require(str(parent) in current and stat.S_ISDIR(current[str(parent)][1].st_mode),
                    f'admitted frozen source parent missing/not directory: {parent}')
            original_names.add(str(parent))
        destination, actual_metadata = current[rel]
        require(stat.S_IFMT(original_metadata.st_mode) == stat.S_IFMT(actual_metadata.st_mode),
                f'prepared Mantle source changes frozen leaf kind {rel}')
        if stat.S_ISLNK(original_metadata.st_mode):
            require(os.readlink(source) == os.readlink(destination), f'frozen source symlink changed: {rel}')
        elif stat.S_ISREG(original_metadata.st_mode):
            require(bool(original_metadata.st_mode & 0o111) == bool(actual_metadata.st_mode & 0o111),
                    f'prepared Mantle source changes frozen executable flag {rel}')
            require(original_metadata.st_size == actual_metadata.st_size, f'frozen source size changed: {rel}')
            with open(source, 'rb') as before, open(destination, 'rb') as after:
                while True:
                    left = before.read(1024 * 1024)
                    right = after.read(1024 * 1024)
                    require(left == right, f'prepared Mantle source changes frozen leaf bytes: {rel}')
                    if not left:
                        break
            regular_files += 1
            payload_bytes += original_metadata.st_size
    additions = sorted(set(current) - original_names)
    return {'frozen_root': str(FROZEN), 'prepared_root': str(prepared_root), 'capture_policy': policy,
            'admitted_frozen_leaf_count': len(original), 'original_entries_preserved': len(original_names),
            'original_regular_files_compared': regular_files, 'original_payload_bytes_compared': payload_bytes,
            'policy_excluded_frozen_leaf_count': len(leaves) - len(original),
            'policy_excluded_top_level_entries': sorted({rel.split('/')[0] for rel, _, _ in frozen
                                                       if rel.split('/')[0] not in allowed}),
            'additional_member_count': len(additions), 'additional_member_paths': additions,
            'scope': 'Every frozen leaf admitted by the frozen capture policy preserves kind, executable flag, symlink target and regular-file bytes. Parent directories exist; empty directories are not source records. Policy exclusions and additions are separate, not a whole-checkout mirror or preauthor/postauthor identity equivalence.'}


def verify_source_copies() -> dict:
    mapping = [
        ('src/source_built_mantle_programs_aws_helper.rs', 'aws-source-helper.rs', 'source-built-mantle-programs'),
        ('src/source_built_mantle_programs_native_helper.rs', 'source_built_mantle_native_helper.rs', 'source-built-mantle-programs'),
        ('src/source_built_rust_programs_translation.c', 'cairn/rust/mrustc-native-translation.c', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_phase.c', 'cairn/rust/compiler-source-phase.c', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_link.rs.in', 'cairn/rust/compiler-native-link.rs', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_env.rs.in', 'cairn/rust/compiler-environment.rs', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_overlay.rs.in', 'cairn/rust/compiler-source-overlay.rs', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_patches.c', 'mantle-source-authoring/rust-patches.c', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_mrustc_metadata.h', 'mantle-source-authoring/mrustc-metadata.hpp', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_mrustc_proc_macro.h', 'mantle-source-authoring/mrustc-proc-macro.hpp', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_minicargo_helper.c', 'mantle-source-authoring/rust-minicargo.c', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_llvm_source.c', 'mantle-source-authoring/rust-llvm-products.c', 'fixed-point-rust-source-program'),
        ('src/source_built_rust_programs_llvm_cmake_patch.c', 'source-built/native/llvm-cmake-patch.c', 'fixed-point-native-source-program'),
        ('src/source_built_rust_programs_llvm_cmake_phase.h', 'source-built/native/llvm-cmake-phase.h', 'fixed-point-native-source-program'),
    ]
    rows = []
    for frozen_rel, member, program_id in mapping:
        src = FROZEN / frozen_rel
        dst = PROGRAMS / program_id / member
        require(src.is_file() and dst.is_file() and not dst.is_symlink(), f'frozen helper/translator copy missing: {frozen_rel} -> {dst}')
        src_b3, dst_b3 = b3_file(src), b3_file(dst)
        require(src.read_bytes() == dst.read_bytes(), f'frozen helper/translator bytes differ: {frozen_rel} -> {dst}')
        rows.append({'frozen_path': str(src), 'program_member': str(dst), 'bytes': src.stat().st_size, 'blake3': src_b3, 'exact_byte_match': True})
    # Authored Rust authority copies are duplicated into the original Rust source authority.
    rust_copy_rows = []
    for member in ('source-built/rust/program.json', 'cairn/rust/compiler-environment.rs'):
        authored = PROGRAMS / 'fixed-point-rust-source-program' / member
        copied = COHORT / 'inputs/rust-source-archives' / member
        require(authored.is_file() and copied.is_file() and not authored.is_symlink() and not copied.is_symlink(),
                f'authored Rust authority copy missing: {member}')
        require(authored.read_bytes() == copied.read_bytes(), f'authored Rust authority copy differs: {member}')
        rust_copy_rows.append({'member': member, 'bytes': authored.stat().st_size, 'blake3': b3_file(authored), 'exact_byte_match': True})
    route_frozen = FROZEN / 'bootstrap/rust-source-musl-host-plan.ncl'
    route_mantle = COHORT / 'inputs/mantle-source/bootstrap/rust-source-musl-host-plan.ncl'
    route_program = PROGRAMS / 'fixed-point-rust-source-program/cairn/rust/rust-source-musl-host-plan.ncl'
    require(route_frozen.is_file() and route_mantle.is_file() and route_program.is_file(), 'selected original Rust route copy missing')
    require(route_frozen.read_bytes() == route_mantle.read_bytes() == route_program.read_bytes(), 'original Rust route bytes differ across frozen/source/program authority')
    return {'frozen_helper_translator_copies': rows, 'rust_authority_copies': rust_copy_rows,
            'frozen_source_preservation': verify_frozen_source_preservation(),
            'original_route_copy': {'bytes': route_frozen.stat().st_size, 'blake3': b3_file(route_frozen), 'three_way_exact_byte_match': True}}

def main() -> None:
    configure()
    require(Path(B3).is_file() and Path(JQ).is_file() and Path(GIT).is_file()
            and Path(TAR).is_file() and Path(NIX).is_file(), 'pinned read-only inspection tools missing')
    require(PYTHON.endswith('python3.14'), f'unexpected Python interpreter {PYTHON}')
    require(FROZEN.is_dir() and not FROZEN.is_symlink() and COHORT.is_dir() and not COHORT.is_symlink(),
            'frozen source/cohort roots missing or symlinked')
    plan = json.loads(PLAN_PATH.read_bytes(), object_pairs_hook=json_pairs_no_dupes)
    report = json.loads(REPORT_PATH.read_bytes(), object_pairs_hook=json_pairs_no_dupes)
    report_raw = b3_file(REPORT_PATH)
    require(report_raw == REPORT_RAW_B3, f'actual source-program report raw BLAKE3 differs from completed receipt: {report_raw}')
    require(report['schema'] == 'mantle-source-built-source-programs-v1', 'actual preparation report schema mismatch')
    require(report['input_profile_blake3'] == INPUT_PROFILE_B3, 'actual report input-profile identity mismatch')
    require(report['source_profile'] == str(PROFILE_PATH), 'actual report source-profile path mismatch')
    require(report['output_profile_blake3'] == PROFILE_MANIFEST_B3, 'actual preparation profile identity mismatch')
    require(report['input_plan_blake3'] == plan['plan_digest_blake3'], 'actual preparation report/plan identity mismatch')
    require(plan['plan_digest_blake3'] == SUBJECT['input_plan_blake3'], 'actual input plan differs from subject')
    canonical_plan = canonical_plan_digest(plan)
    require(canonical_plan == plan['plan_digest_blake3'],
            f'actual canonical input plan digest mismatch: computed={canonical_plan} recorded={plan["plan_digest_blake3"]}')
    require(plan['source_inputs'] == SUBJECT['source_inputs'] and plan['resource_bounds'] == SUBJECT['resource_bounds'],
            'subject source-input or resource claims differ from actual plan')
    require(report['program_records'] == SUBJECT['program_records'], 'subject program identities differ from report')
    lineage_plan = next(x for x in plan['source_inputs'] if x['role'] == 'stagex-lineage')
    require(lineage_plan['digest_blake3'] == LINEAGE_B3, 'actual source plan lineage identity differs from independent pin')
    require(plan['policies']['expected_native_provider_digest_blake3'] == NATIVE_PROVIDER_B3,
            'actual plan native provider pin differs')
    require(set(report['program_records']) == PROGRAM_IDS, 'actual report does not list exactly four new programs')
    expected_plan_roles = {'mantle-source', 'native-source-bundle', 'rust-source-archive-set',
                           'stagex-lineage', 'stagex-seed', 'stagex-source-bundle', 'vendor-inputs'}
    observed_plan_roles = {x['role'] for x in plan['source_inputs']}
    require(len(plan['source_inputs']) == 7 and observed_plan_roles == expected_plan_roles,
            f'actual source plan roles differ from the seven expected roles: {sorted(observed_plan_roles)}')
    require(observed_plan_roles == {x['id'] for x in plan['source_inputs']},
            'source plan role IDs differ from identity IDs')
    print('phase=plan-bound', flush=True)

    # Bind the fresh installed runner and normalized complete source to frozen pins.
    runner_hash = b3_file(RUNNER)
    require(runner_hash == RUNNER_B3, f'fresh installed runner hash mismatch: {runner_hash}')
    frozen_tar_hash = normalized_tar_b3(FROZEN)
    require(frozen_tar_hash == FROZEN_B3, f'frozen normalized complete-source archive hash mismatch: {frozen_tar_hash}')
    frozen_source = {'path': str(FROZEN), 'normalized_gnu_tar_blake3': frozen_tar_hash,
                     'expected_blake3': FROZEN_B3, 'runner_path': str(RUNNER),
                     'runner_blake3': runner_hash, 'expected_runner_blake3': RUNNER_B3}

    # Bind the pinned original AWS-LC Git commit/tree and both complete copies.
    git_commit = run([GIT, f'--git-dir={AWS_DB}', 'rev-parse', f'{AWS_COMMIT}^{{commit}}']).decode().strip()
    git_tree = run([GIT, f'--git-dir={AWS_DB}', 'rev-parse', f'{AWS_COMMIT}^{{tree}}']).decode().strip()
    require(git_commit == AWS_COMMIT and git_tree == AWS_TREE, f'pinned AWS-LC Git identity mismatch {git_commit}/{git_tree}')
    raw = run([GIT, f'--git-dir={AWS_DB}', 'ls-tree', '-r', '-z', AWS_COMMIT])
    git_entries = []
    for entry in raw.split(b'\0'):
        if not entry:
            continue
        header, rel = entry.split(b'\t', 1)
        mode, typ, oid = header.decode().split(' ')
        require(b'\n' not in rel and b'\r' not in rel, 'newline in pinned AWS-LC Git path')
        git_entries.append({'mode': mode, 'type': typ, 'oid': oid, 'path': rel.decode('utf-8')})
    require(bool(git_entries), 'pinned AWS-LC Git inventory is empty')
    original_hash = verify_git_tree_copy('FROZEN', FROZEN / 'bootstrap/source-inputs' / f'aws-lc-original-{AWS_COMMIT}', git_entries)
    prepared_hash = verify_git_tree_copy('PREPARED', COHORT / 'inputs/mantle-source/bootstrap/source-inputs' / f'aws-lc-original-{AWS_COMMIT}', git_entries)

    print('phase=original-git-copies-bound', flush=True)
    # Compare complete native source record bodies, not only identity/digest labels.
    plan_native = next(x for x in plan['source_inputs'] if x['role'] == 'native-source-bundle')
    require(PREAUTHOR_PROFILE_PATH.is_file() and not PREAUTHOR_PROFILE_PATH.is_symlink()
            and AUGMENTED_CLOSURE.is_file() and not AUGMENTED_CLOSURE.is_symlink(),
            'preauthor source profile or augmented native closure is missing/nonregular')
    original_records, preauthor_profile = preauthor_native_profile_records(PREAUTHOR_PROFILE_PATH, ORIGINAL_CLOSURE)
    original_raw = preauthor_profile['bootstrap_source_role']['physical_file_blake3']
    require(PREAUTHOR_PLAN_CLOSURE.is_file() and not PREAUTHOR_PLAN_CLOSURE.is_symlink(),
            'preauthor plan native-source closure file missing/nonregular')
    preauthor_plan_raw = b3_file(PREAUTHOR_PLAN_CLOSURE)
    require(preauthor_plan_raw == plan_native['digest_blake3']
            and PREAUTHOR_PLAN_CLOSURE.stat().st_size == plan_native['size_bytes'],
            f'preauthor plan native-source closure physical bytes differ from plan: {preauthor_plan_raw}')
    preauthor_plan_records = closure_records(PREAUTHOR_PLAN_CLOSURE)
    preauthor_plan_ids = set(preauthor_plan_records)
    require(preauthor_plan_ids == set(original_records),
            f'preauthor plan native record identities differ from input profile: missing={len(set(original_records)-preauthor_plan_ids)} extra={len(preauthor_plan_ids-set(original_records))}')
    preauthor_plan_changed = [identity for identity in sorted(preauthor_plan_ids)
                              if preauthor_plan_records[identity]['canonical_body_blake3'] != original_records[identity]['canonical_body_blake3']]
    require(not preauthor_plan_changed, f'preauthor plan native source record bodies differ from input profile: {preauthor_plan_changed[:10]}')
    augmented_records = closure_records(AUGMENTED_CLOSURE)
    orig_ids, aug_ids = set(original_records), set(augmented_records)
    require(orig_ids <= aug_ids, f'original native record identities missing after preparation: {sorted(orig_ids-aug_ids)[:5]}')
    changed = [identity for identity in sorted(orig_ids) if original_records[identity]['canonical_body_blake3'] != augmented_records[identity]['canonical_body_blake3']]
    require(not changed, f'complete original native source record bodies changed: {changed[:10]}')
    added = aug_ids - orig_ids
    require(added == PROGRAM_IDS, f'augmented native closure additions are not exactly the four report programs: {sorted(added)}')
    require(len(augmented_records) == len(original_records) + len(PROGRAM_IDS), 'augmented native closure count differs from original+programs')
    require(set(report['program_records']) == PROGRAM_IDS, 'preparation report program IDs changed')
    expected_program_entries = PROGRAM_IDS | {f'{identity}.tar' for identity in PROGRAM_IDS}
    actual_program_entries = {entry.name for entry in PROGRAMS.iterdir()}
    require(actual_program_entries == expected_program_entries,
            f'program-sources contains unexpected/missing physical entries: {sorted(actual_program_entries ^ expected_program_entries)}')
    closure_programs = {}
    for identity in sorted(PROGRAM_IDS):
        entry = augmented_records[identity]
        record = entry.get('record')
        require(record is not None, f'missing parsed new program record {identity}')
        closure_programs[identity] = verify_program_record(identity, record, report['program_records'][identity], report)
        augmented_records[identity].pop('record', None)
    augmented_raw = b3_file(AUGMENTED_CLOSURE)
    augmented_size = AUGMENTED_CLOSURE.stat().st_size
    print('phase=native-records-and-programs-bound', flush=True)

    # Bind all launch-input roles and programs to physical bytes and bind every
    # complete original native body to the already verified augmented closure.
    profile_raw = b3_file(PROFILE_PATH)
    require(profile_raw == PROFILE_RAW_B3, f'complete source-profile raw BLAKE3 mismatch: {profile_raw}')
    require(PROFILE_PATH.stat().st_size == SUBJECT['profile_file_bytes'], 'profile size differs from full-stream measurement')
    profile_data = profile_record_scan(PROFILE_PATH, plan, report, augmented_records)
    print('phase=profile-role-program-and-original-native-bytes-bound', flush=True)

    # Verify seven materialized role identities against the source plan without
    # confusing preauthor directory digests with augmented postauthor roots.
    role_physical = {}
    for role in plan['source_inputs']:
        role_id, role_kind = role['role'], role['kind']
        if role_id in ('mantle-source', 'vendor-inputs', 'rust-source-archive-set'):
            profile_class = {'mantle-source': 'mantle-source', 'vendor-inputs': 'vendored-cargo-inputs',
                             'rust-source-archive-set': 'proof-input'}[role_id]
            role_result = profile_data['role_records'][profile_class]
            role_physical[role_id] = {'preauthor_plan_digest_blake3': role['digest_blake3'],
                                      'preauthor_plan_payload_bytes': role['size_bytes'],
                                      'postauthor_record_content_blake3': role_result['record_content_blake3'],
                                      'postauthor_physical_tree': role_result['physical_tree'],
                                      'postauthor_payload_bytes': role_result['embedded_payload_bytes_compared'],
                                      'preauthor_physical_reconstructed': False}
        elif role_id == 'native-source-bundle':
            current = profile_data['role_records']['bootstrap-source']
            require(current['content_files'] == 1, 'current native-source-bundle role is not a single closure file')
            role_physical[role_id] = {'preauthor_plan_digest_blake3': role['digest_blake3'],
                                      'preauthor_plan_payload_bytes': role['size_bytes'],
                                      'preauthor_input_native_record_count': len(original_records),
                                      'input_profile_bootstrap_source_role_raw_blake3': original_raw,
                                      'postauthor_record_content_blake3': current['record_content_blake3'],
                                      'postauthor_physical_tree': current['physical_tree'],
                                      'postauthor_payload_bytes': current['embedded_payload_bytes_compared'],
                                      'augmented_closure_raw_blake3': augmented_raw,
                                      'augmented_closure_bytes': augmented_size}
        else:
            profile_class = role_id
            current = profile_data['role_records'][profile_class]
            root = Path(current['physical_root'])
            rel = PLAN_FILE_RELATIVE[role_id]
            leaf = root / rel
            leaf_stat = os.lstat(leaf)
            require(stat.S_ISREG(leaf_stat.st_mode), f'unmutated preauthor file role is not a regular file: {role_id}')
            observed = b3_file(leaf)
            require(observed == role['digest_blake3'] and leaf_stat.st_size == role['size_bytes'],
                    f'unmutated preauthor file role differs from actual plan: {role_id} {observed}')
            role_physical[role_id] = {'preauthor_plan_digest_blake3': role['digest_blake3'],
                                      'preauthor_physical_file_blake3': observed,
                                      'postauthor_record_content_blake3': current['record_content_blake3'],
                                      'postauthor_physical_tree': current['physical_tree'],
                                      'postauthor_payload_bytes': current['embedded_payload_bytes_compared'],
                                      'preauthor_physical_reconstructed': True}
    require(set(role_physical) == {x['role'] for x in plan['source_inputs']}, 'not all seven source plan roles physically reviewed')

    # The source-profile's four program bodies must have matched closure bodies.
    for identity in PROGRAM_IDS:
        require(profile_data['program_records'][identity]['record_content_blake3'] == report['program_records'][identity],
                f'output profile program identity differs from report: {identity}')

    # Rebind all frozen helper/translator/Rust-authority copies and original route.
    source_copies = verify_source_copies()

    # Inspect actual current selectors, perlasm boundaries, authenticator outputs,
    # consumed generated/translated products and exact stage1/stage2 parity.
    # The exact source text comes from the pinned frozen source tree; it is never run.
    native_text = (FROZEN / 'src/source_built_mantle_programs_native.rs').read_text(encoding='utf-8')
    pipe_match = re.search(r'const PIPE_SOURCE: &str = r###"(.*?)"###;', native_text, re.S)
    require(pipe_match is not None, 'frozen PIPE_SOURCE missing')
    stage_results = [inspect_recipe_stage(n, pipe_match.group(1), report) for n in (1, 2)]
    source_selection = verify_selected_sources(stage_results)
    print('phase=selected-authenticated-sources-bound', flush=True)

    # Confirm preparation leaves only source/plan/report material and the
    # selected launch OUT, provider/store roots and first-launch outputs absent.
    expected_root = {'inputs', 'program-sources', 'native-source-closure.json', 'source-profile.json',
                     'source-program-preparation-input-plan.json', 'source-program-preparation.json'}
    actual_root = {p.name for p in COHORT.iterdir()}
    require(actual_root == expected_root, f'cohort has unexpected/missing top-level authority paths: {sorted(actual_root ^ expected_root)}')
    expected_input_roles = {'mantle-source', 'native-source-authority', 'rust-source-archives', 'stagex-lineage', 'stagex-seed', 'stagex-source-bundle', 'vendor-deps'}
    actual_input_roles = {p.name for p in (COHORT / 'inputs').iterdir()}
    require(actual_input_roles == expected_input_roles, f'materialized source role roots differ: {sorted(actual_input_roles ^ expected_input_roles)}')
    absent_names = ['native-store', 'native-state', 'transcripts', 'full-source-rust-host-tools',
                    'rust-provider', 'rust-provider-scratch', 'cargo-free-fixed-point',
                    'stagex-transition-execution', 'stagex-transition-handoff-replay', 'stagex-provider-replay']
    absent_paths = [COHORT / name for name in absent_names]
    for name in ('native-store', 'native-state', 'transcripts', 'full-source-rust-host-tools',
                 'rust-provider', 'rust-provider-scratch', 'cargo-free-fixed-point',
                 'stagex-transition-execution', 'stagex-transition-handoff-replay', 'stagex-provider-replay'):
        absent_paths.append(COHORT / 'inputs' / name)
    proof_out = RUN / f'proof-{COHORT_NAME}'
    launch_outputs = [proof_out, RUN / f'cold-proof-{COHORT_NAME}.log', RUN / f'cold-proof-{COHORT_NAME}.exit',
                      RUN / f'cold-proof-report-{COHORT_NAME}.json', RUN / f'cold-proof-trust-report-{COHORT_NAME}.json',
                      RUN / f'cold-proof-{COHORT_NAME}-home', RUN / f'cold-proof-{COHORT_NAME}-tmp']
    present = [str(p) for p in absent_paths + launch_outputs if os.path.lexists(p)]
    require(not present, f'initial proof output/provider/store authority already exists: {present}')
    authority = {'cohort_top_level_entries': sorted(actual_root),
                 'absent_provider_store_roots': [str(p) for p in absent_paths],
                 'absent_first_launch_outputs': [str(p) for p in launch_outputs],
                 'all_absent': True}

    result = {
        'schema': 'mantle-fresh-source-cohort-identity-review-v1',
        'scope': 'inert physical identity/authenticated-source review only',
        'host': SUBJECT['host'],
        'run': str(RUN), 'cohort': str(COHORT),
        'frozen_product_revision': SUBJECT['frozen_product_revision'],
        'frozen_source_and_runner': frozen_source,
        'actual_plan': {'path': str(PLAN_PATH), 'raw_blake3': b3_file(PLAN_PATH),
                        'digest_blake3': plan['plan_digest_blake3'], 'source_roles': plan['source_inputs'],
                        'lineage_pin': LINEAGE_B3, 'native_provider_pin': NATIVE_PROVIDER_B3},
        'actual_preparation_report': {'path': str(REPORT_PATH), 'raw_blake3': report_raw,
                                      'schema': report['schema'], 'input_profile_blake3': report['input_profile_blake3'],
                                      'input_plan_blake3': report['input_plan_blake3'],
                                      'output_profile_blake3': report['output_profile_blake3'],
                                      'program_records': report['program_records']},
        'input_source_profile': preauthor_profile,
        'source_profile': {'path': str(PROFILE_PATH), 'bytes': PROFILE_PATH.stat().st_size,
                           'raw_blake3': profile_raw, 'parent_full_ssh_stream_blake3': PROFILE_RAW_B3,
                           'parent_profile_transfer_equal': profile_raw == PROFILE_RAW_B3,
                           'manifest_blake3': profile_data['manifest_blake3']},
        'aws_lc_original_git': {'git_dir': str(AWS_DB), 'commit': git_commit, 'tree': git_tree,
                                'normalized_tree_blake3_expected': AWS_TREE_B3,
                                'copies': [original_hash, prepared_hash]},
        'native_closure': {'preauthor_input_profile_path': str(PREAUTHOR_PROFILE_PATH),
                           'preauthor_input_profile_manifest_blake3': preauthor_profile['manifest_blake3'],
                           'preauthor_input_profile_raw_blake3': preauthor_profile['raw_blake3'],
                           'preauthor_input_profile_bytes': preauthor_profile['bytes'],
                           'preauthor_native_record_count': len(original_records),
                           'preauthor_plan_source_bundle_digest_blake3': plan_native['digest_blake3'],
                           'preauthor_plan_source_bundle_bytes': plan_native['size_bytes'],
                           'preauthor_plan_bundle_raw_file_available_after_authoring': True,
                           'preauthor_plan_closure_path': str(PREAUTHOR_PLAN_CLOSURE),
                           'preauthor_plan_closure_raw_blake3': preauthor_plan_raw,
                           'preauthor_plan_closure_bytes': PREAUTHOR_PLAN_CLOSURE.stat().st_size,
                           'preauthor_plan_closure_record_count': len(preauthor_plan_records),
                           'preauthor_plan_native_body_mismatches': [],
                           'preauthor_bootstrap_source_input_role': preauthor_profile['bootstrap_source_role'],
                           'augmented_path': str(AUGMENTED_CLOSURE),
                           'augmented_raw_blake3': augmented_raw, 'augmented_bytes': augmented_size,
                           'augmented_record_count': len(augmented_records),
                           'original_record_body_fingerprints_compared': len(orig_ids),
                           'output_profile_original_record_body_comparisons': len(profile_data['original_native_records']),
                           'original_record_body_mismatches': [], 'new_record_ids': sorted(added)},
        'program_records': closure_programs,
        'source_roles': role_physical,
        'profile_role_and_program_records': profile_data,
        'source_copy_bindings': source_copies,
        'selected_authenticated_perlasm': {'stages': stage_results, **source_selection},
        'initial_authority_absence': authority,
        'commands_and_tools': {'python': PYTHON, 'b3sum': B3, 'jq': JQ, 'git': GIT, 'tar': TAR, 'nix': NIX,
                               'source_hash_command': f'{TAR} --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory <source> . | {B3} --no-names',
                               'program_nar_command': f'{NIX} --extra-experimental-features blake3-hashes hash path --type blake3 --base16 <program-source-dir>',
                               'git_blob_command': f'{GIT} --git-dir={AWS_DB} hash-object --no-filters --stdin-paths'},
        'non_claims': ['no product/helper/generator/provider/controller/recipe was executed',
                       'no proof/build/test/lint/formatter was run',
                       'no compiler-correctness, runtime-success, fixed-point, reproducibility or release claim',
                       'overall launch authority remains with parent integration and protected execution'],
        'supplemental_required_criteria': ['preauthor directory-role identities from embedded input-profile bytes'],
        'bounded_identity_verdict': 'STATIC_IDENTITY_CHECKS_CLEAR'}
    result['subject_binding'] = {'path': str(SUBJECT_PATH), 'raw_blake3': b3_file(SUBJECT_PATH)}
    result['inspector'] = {'path': str(Path(__file__).resolve()), 'blake3': b3_file(Path(__file__).resolve()),
                           'command': [PYTHON, str(Path(__file__).resolve()), *sys.argv[1:]]}
    result['coverage'] = {
        'source_roles': {'total': len(plan['source_inputs']), 'covered': len(role_physical), 'uncovered': []},
        'profile_embedded_role_entries': {'total': sum(x['content_entries'] for x in profile_data['role_records'].values()),
                                          'covered': sum(x['content_entries'] for x in profile_data['role_records'].values()), 'uncovered': []},
        'program_records': {'total': len(PROGRAM_IDS), 'covered': len(closure_programs), 'uncovered': []},
        'program_embedded_entries': {'total': sum(p['entry_count'] for p in closure_programs.values()),
                                     'covered': sum(p['entry_count'] for p in closure_programs.values()), 'uncovered': []},
        'program_recursive_nars': {'total': len(PROGRAM_IDS), 'covered': len(closure_programs), 'uncovered': []},
        'original_native_body_comparisons': {'total': len(orig_ids) * 3,
                                             'covered': len(orig_ids) * 2 + len(profile_data['original_native_records']),
                                             'comparison_sources': ['preauthor-plan', 'augmented-closure', 'launch-input-profile'],
                                             'uncovered': []},
        'archive_members': {'total': sum(p['unique_files'] for p in closure_programs.values()),
                            'covered': sum(p['archive_member_count'] for p in closure_programs.values()), 'uncovered': []},
        'preauthor_directory_roles': {'total': len(('mantle-source', 'rust-source-archive-set', 'vendor-inputs')),
                                      'covered': 0, 'uncovered': ['preauthor embedded directory reconstruction: mantle-source',
                                                                'preauthor embedded directory reconstruction: rust-source-archive-set',
                                                                'preauthor embedded directory reconstruction: vendor-inputs']},
        'original_git_leaves': {'total': len(git_entries) * 2,
                                'covered': original_hash['git_blob_bytes_match'] + prepared_hash['git_blob_bytes_match'], 'uncovered': []},
        'admitted_frozen_source_leaves': {'total': source_copies['frozen_source_preservation']['admitted_frozen_leaf_count'],
                                         'covered': source_copies['frozen_source_preservation']['admitted_frozen_leaf_count'], 'uncovered': []},
        'selected_aws_and_ring_script_routes': {'total': sum(s['aws_count'] + s['ring_count'] for s in stage_results),
                                                'covered': sum(len(s['aws']) + len(s['ring']) for s in stage_results), 'uncovered': []},
        'selected_pipe_sources': {'total': sum(s['aws_count'] + s['ring_count'] for s in stage_results),
                                  'covered': sum(s['pipe_source_count'] for s in stage_results), 'uncovered': []},
        'selected_assembler_probe_boundaries': {'total': len(source_selection['aws_probe_boundaries']),
                                                 'covered': len(source_selection['aws_probe_boundaries']), 'uncovered': []},
        'frozen_helper_translator_copies': {'total': len(source_copies['frozen_helper_translator_copies']),
                                            'covered': len(source_copies['frozen_helper_translator_copies']), 'uncovered': []},
        'native_owned_llvm_members': {'total': len([x for x in source_copies['frozen_helper_translator_copies']
                                                  if 'llvm-cmake-' in x['program_member']]),
                                      'covered': len([x for x in source_copies['frozen_helper_translator_copies']
                                                      if 'llvm-cmake-' in x['program_member']]), 'uncovered': []},
        'initial_authority_paths': {'total': len(absent_paths) + len(launch_outputs),
                                    'covered': len(absent_paths) + len(launch_outputs), 'uncovered': []},
    }
    criterion7 = ('source_roles', 'profile_embedded_role_entries', 'program_records',
                  'program_embedded_entries', 'program_recursive_nars', 'original_native_body_comparisons',
                  'archive_members', 'preauthor_directory_roles')
    criterion8 = ('original_git_leaves', 'admitted_frozen_source_leaves',
                  'selected_aws_and_ring_script_routes', 'selected_pipe_sources',
                  'selected_assembler_probe_boundaries', 'frozen_helper_translator_copies',
                  'native_owned_llvm_members', 'initial_authority_paths')
    result['criterion_coverage'] = {
        criterion: {'total': sum(result['coverage'][key]['total'] for key in keys),
                    'covered': sum(result['coverage'][key]['covered'] for key in keys),
                    'uncovered': [item for key in keys for item in result['coverage'][key]['uncovered']]}
        for criterion, keys in (('7', criterion7), ('8', criterion8))}
    result['tool_hashes'] = {name: {'path': path, 'blake3': b3_file(Path(path))}
                             for name, path in [('python', PYTHON), ('b3sum', B3), ('jq', JQ), ('git', GIT), ('tar', TAR), ('nix', NIX)]}
    result['frozen_identity_consumers'] = {path: b3_file(FROZEN / path) for path in
        ('src/release_tree_copy.rs', 'src/source_bundle.rs', 'src/self_build.rs',
         'src/source_built_fixed_point_planning.rs', 'src/source_built_fixed_point_shell.rs',
         'src/source_built_fixed_point_authoring.rs', 'src/source_built_mantle_programs_native.rs',
         'src/source_built_mantle_programs_graph.rs')}
    out_path = SCRATCH / 'identity-review.json'
    with out_path.open('x', encoding='utf-8') as output:
        json.dump(result, output, indent=2, sort_keys=True)
        output.write('\n')
    print(json.dumps({'result': str(out_path), 'status': result['bounded_identity_verdict'],
                      'roles': len(role_physical), 'original_records_preserved': len(orig_ids),
                      'new_program_records': len(closure_programs),
                      'aws_scripts': source_selection['aws_selected_count'],
                      'ring_scripts': source_selection['ring_selected_count'],
                      'source_profile_bytes': PROFILE_PATH.stat().st_size,
                      'source_profile_raw_blake3': profile_raw}, sort_keys=True))

if __name__ == '__main__':
    try:
        main()
    except Exception as e:
        print(f'BLOCKER {type(e).__name__}: {e}', file=sys.stderr)
        raise
