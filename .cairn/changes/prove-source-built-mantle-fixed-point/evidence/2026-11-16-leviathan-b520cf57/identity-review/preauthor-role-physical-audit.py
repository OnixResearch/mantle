#!/usr/bin/env python3
"""Streaming physical identity check for preauthor source-profile roles.

Derives preauthor role tree/file identities directly from authenticated embedded
leaf bytes under the frozen release_tree_copy/source_bundle hash layouts. No
multi-gigabyte role copies or product/helper execution; results are written only
to caller-owned scratch outside the immutable source and cohort roots.

Usage: preauthor-role-physical-audit.py COHORT FROZEN_SOURCE REVIEW_SCRATCH SUBJECT_JSON
"""
from __future__ import annotations
import collections, json, os, re, subprocess, sys
from pathlib import Path, PurePosixPath

if len(sys.argv) != 5:
    raise SystemExit('usage: preauthor-role-physical-audit.py COHORT FROZEN_SOURCE REVIEW_SCRATCH SUBJECT_JSON')
COHORT, FROZEN, SCRATCH, SUBJECT_PATH = map(Path, sys.argv[1:])
PRODUCT_REVISION='b520cf5791fb7638ab78ed59f42d890ccd75dc30'
RUN_ROOT=Path('/home/brittonr/sbfp-20261113-berdKH')
COHORT_ROOT=RUN_ROOT/'prepared-cohort-b520cf57-captured-operator-sources'
SOURCE_ROOT=RUN_ROOT/'runner-source-b520cf57-captured-operator-sources'
ROLE_CLASS_TO_PLAN_ROLE = {
    'stagex-seed':'stagex-seed', 'stagex-lineage':'stagex-lineage',
    'stagex-source-bundle':'stagex-source-bundle', 'bootstrap-source':'native-source-bundle',
    'proof-input':'rust-source-archive-set', 'mantle-source':'mantle-source',
    'vendored-cargo-inputs':'vendor-inputs',
}
DIRECTORY_CLASSES={'mantle-source','proof-input','vendored-cargo-inputs'}
CHUNK_BYTES=64*1024*1024
MAX_RECORD_BYTES=20*1024**3

class AuditError(RuntimeError): pass

def require(ok,msg):
    if not ok: raise AuditError(msg)

def object_no_dupes(pairs):
    out={}
    for k,v in pairs:
        require(k not in out,f'duplicate JSON key {k!r}')
        out[k]=v
    return out

def load_json(path):
    require(path.is_file() and not path.is_symlink(),f'missing/nonregular JSON: {path}')
    return json.loads(path.read_bytes(),object_pairs_hook=object_no_dupes)

def run(cmd,data=None):
    p=subprocess.run(cmd,input=data,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=86400)
    require(p.returncode==0,f'command failed rc={p.returncode}: {cmd!r}: {p.stderr.decode("utf-8","replace")[-1200:]}')
    return p.stdout

def b3_bytes(tool,data):
    value=run([tool,'--no-names'],data).decode().strip()
    require(re.fullmatch(r'[0-9a-f]{64}',value) is not None,f'invalid BLAKE3 output {value!r}')
    return value

def b3_file(tool,path):
    value=run([tool,'--no-names',str(path)]).decode().strip()
    require(re.fullmatch(r'[0-9a-f]{64}',value) is not None,f'invalid BLAKE3 output for {path}: {value!r}')
    return value

def canonical_plan_digest(tool,plan):
    data=dict(plan); data['plan_digest_blake3']=''
    raw=json.dumps(data,ensure_ascii=False,separators=(',',':')).encode()
    return run([tool,'--derive-key','mantle-source-built-fixed-point-plan-v2','--no-names'],raw).decode().strip()

def footer_manifest(path):
    with path.open('rb') as f:
        f.seek(max(0,path.stat().st_size-65536)); tail=f.read()
    match=re.search(rb'^  "manifest_blake3"\s*:\s*"([0-9a-f]{64})"\s*,?\s*$',tail,re.M)
    require(match is not None,f'profile manifest footer missing: {path}')
    return match.group(1).decode()

def iter_pretty_records(path,max_records):
    in_records=False; in_record=False; buf=bytearray(); size=0; count=0
    with path.open('rb',buffering=1024*1024) as stream:
        for line in stream:
            if not in_records:
                if re.match(rb'^\s*"records"\s*:\s*\[\s*$',line.strip()): in_records=True
                continue
            if not in_record:
                if line in (b'    {\n',b'    {\r\n'):
                    in_record=True; buf=bytearray(line); size=len(line)
                elif line.lstrip().startswith(b']'): return
                continue
            buf.extend(line); size+=len(line)
            require(size<=MAX_RECORD_BYTES,f'profile record exceeded bounded parser limit: {path}')
            if line in (b'    },\n',b'    },\r\n',b'    }\n',b'    }\r\n'):
                raw=buf.rstrip()
                if raw.endswith(b','): raw=raw[:-1]
                record=json.loads(raw,object_pairs_hook=object_no_dupes)
                require(isinstance(record,dict) and isinstance(record.get('identity'),str),f'malformed profile record in {path}')
                count+=1; require(count<=max_records,f'profile record count exceeds subject bound {max_records}')
                yield record
                in_record=False; buf=bytearray(); size=0
    require(in_records and not in_record,f'truncated/unrecognized profile records in {path}')

def canonical_record_digest(tool,record):
    """Matches digest_source_record_content for ordinary profile records."""
    proc=subprocess.Popen([tool,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    try:
        metadata=record.get('metadata',{})
        if not metadata:
            for entry in record['files']:
                proc.stdin.write(json.dumps(entry,ensure_ascii=False,separators=(',',':')).encode()); proc.stdin.write(b'\n')
        elif not record['files']:
            raise AuditError('role source record has no file entries; virtual source digest requires its specific decoder')
        else:
            proc.stdin.write(b'kind\0')
            proc.stdin.write(json.dumps(record['kind'],ensure_ascii=False,separators=(',',':')).encode()); proc.stdin.write(b'\n')
            proc.stdin.write(b'metadata\0')
            proc.stdin.write(json.dumps(metadata,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()); proc.stdin.write(b'\n')
            proc.stdin.write(b'files\0')
            for entry in record['files']:
                proc.stdin.write(json.dumps(entry,ensure_ascii=False,separators=(',',':')).encode()); proc.stdin.write(b'\n')
        out,err=proc.communicate(timeout=86400)
    except BaseException:
        if proc.poll() is None: proc.kill()
        raise
    require(proc.returncode==0,f'canonical record BLAKE3 failed: {err.decode("utf-8","replace")[-1200:]}')
    digest=out.decode().strip(); require(re.fullmatch(r'[0-9a-f]{64}',digest) is not None,f'invalid record digest {digest!r}')
    return digest

def safe_rel(value):
    require(isinstance(value,str) and value and '\x00' not in value and '\\' not in value,f'unsafe source leaf path {value!r}')
    p=PurePosixPath(value)
    require(not p.is_absolute() and p.as_posix()==value and all(x not in ('','.','..') for x in p.parts),f'noncanonical source leaf path {value!r}')
    return value

def hash_embedded_file(tool,entries,tree_stdin=None):
    """Hash one logical file while optionally feeding exact bytes into tree hash."""
    indexes=[e.get('chunk_index') for e in entries]; counts=[e.get('chunk_count') for e in entries]
    if indexes==[None] and counts==[None]: require(len(entries)==1,f'duplicate unchunked file {entries[0]["path"]}')
    else:
        require(len(entries)>=2 and indexes==list(range(len(entries))) and len(set(counts))==1 and counts[0]==len(entries),
                f'invalid chunk sequence for {entries[0]["path"]}')
        require(all(int(e['size'])==CHUNK_BYTES for e in entries[:-1]) and 0<int(entries[-1]['size'])<=CHUNK_BYTES,
                f'noncanonical chunk sizes for {entries[0]["path"]}')
    proc=subprocess.Popen([tool,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    total=0
    try:
        for entry in entries:
            text=entry.get('content_hex')
            require(isinstance(text,str) and text==text.lower() and len(text)==2*int(entry['size'])
                    and re.fullmatch(r'[0-9a-f]*',text) is not None,f'malformed embedded payload {entry["path"]}')
            data=bytes.fromhex(text)
            require(len(data)==int(entry['size']),f'embedded payload size mismatch {entry["path"]}')
            if entry.get('chunk_index') is not None:
                require(b3_bytes(tool,data)==entry['blake3'],f'embedded chunk identity mismatch {entry["path"]}#{entry["chunk_index"]}')
            proc.stdin.write(data)
            if tree_stdin is not None: tree_stdin.write(data)
            total+=len(data)
        out,err=proc.communicate(timeout=86400)
    except BaseException:
        if proc.poll() is None: proc.kill()
        raise
    require(proc.returncode==0,f'embedded file BLAKE3 failed: {err.decode("utf-8","replace")[-1200:]}')
    digest=out.decode().strip()
    if indexes==[None]: require(digest==entries[0]['blake3'],f'embedded file identity mismatch {entries[0]["path"]}')
    return total,digest

def materialized_tree_digest(tool,record):
    """Reproduce hash_directory_tree_with_limits ordering and file/directory modes."""
    by_path=collections.defaultdict(list)
    for entry in record['files']: by_path[safe_rel(entry['path'])].append(entry)
    require(bool(by_path),f'{record["identity"]}: empty source directory record')
    dirs=set()
    for rel in by_path:
        parent=PurePosixPath(rel).parent
        while parent.as_posix()!='.': dirs.add(parent.as_posix()); parent=parent.parent
    require(not (dirs & set(by_path)), 'source file collides with a required parent directory')
    entries=[(rel,'directory',None) for rel in dirs]
    for rel,items in by_path.items():
        items.sort(key=lambda e:-1 if e.get('chunk_index') is None else int(e['chunk_index']))
        kind=items[0]['file_type']; require(all(x['file_type']==kind for x in items),f'{rel}: mixed file types')
        if kind=='regular':
            require(all(bool(x['executable'])==bool(items[0]['executable']) for x in items),f'{rel}: mixed executable flags')
            entries.append((rel,'regular',items))
        elif kind=='symlink':
            require(len(items)==1 and items[0].get('chunk_index') is None and items[0].get('content_hex') is None
                    and int(items[0]['size'])==0 and isinstance(items[0].get('symlink_target'),str),f'{rel}: malformed symlink')
            target=items[0]['symlink_target']
            require(target and '\x00' not in target and not os.path.isabs(target), f'{rel}: invalid internal link')
            resolved=os.path.normpath(os.path.join(os.path.dirname(rel),target))
            require(resolved!='..' and not resolved.startswith('../') and not os.path.isabs(resolved),
                    f'{rel}: symlink target escapes source root')
            entries.append((rel,'symlink',items[0]['symlink_target']))
        else: raise AuditError(f'{rel}: unsupported file type {kind!r}')
    entries.sort(key=lambda x:x[0].encode('utf-8'))
    proc=subprocess.Popen([tool,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    total=0; file_count=symlink_count=0
    try:
        for rel,kind,value in entries:
            raw=rel.encode('utf-8'); proc.stdin.write(len(raw).to_bytes(8,'little')); proc.stdin.write(raw)
            if kind=='directory':
                proc.stdin.write((0o040755).to_bytes(4,'little')); proc.stdin.write(b'dir\0')
            elif kind=='symlink':
                proc.stdin.write((0o120777).to_bytes(4,'little')); target=value.encode('utf-8')
                proc.stdin.write(b'symlink\0'); proc.stdin.write(len(target).to_bytes(8,'little')); proc.stdin.write(target); symlink_count+=1
            else:
                mode=0o100000|(0o755 if value[0]['executable'] else 0o644)
                proc.stdin.write(mode.to_bytes(4,'little'))
                size=sum(int(e['size']) for e in value)
                proc.stdin.write(b'file\0'); proc.stdin.write(size.to_bytes(8,'little'))
                actual,_=hash_embedded_file(tool,value,proc.stdin)
                require(actual==size,f'{rel}: reconstructed file size mismatch')
                total+=size; file_count+=1
        out,err=proc.communicate(timeout=86400)
    except BaseException:
        if proc.poll() is None: proc.kill()
        raise
    require(proc.returncode==0,f'materialized tree BLAKE3 failed: {err.decode("utf-8","replace")[-1200:]}')
    digest=out.decode().strip(); require(re.fullmatch(r'[0-9a-f]{64}',digest) is not None,f'invalid tree identity {digest!r}')
    if total==0: total=1
    return {'digest_blake3':digest,'payload_file_bytes':total,'entry_count':len(entries),
            'directory_entries':len(dirs),'regular_file_count':file_count,'symlink_count':symlink_count,
            'source_leaf_record_entries':len(record['files'])}

def inspect_role(tool,record,planrow):
    canonical=canonical_record_digest(tool,record)
    require(canonical==record['content_blake3'],f'{record["identity"]}: canonical record body digest mismatch')
    payload=sum(int(e['size']) for e in record['files'])
    require(payload==int(record['payload_bytes']),f'{record["identity"]}: payload total mismatch')
    klass=record['metadata']['bootstrap_profile_class']
    entries=collections.defaultdict(list)
    for entry in record['files']: entries[safe_rel(entry['path'])].append(entry)
    if klass=='bootstrap-source':
        require(planrow is not None and planrow['kind']=='regular-file',
                'bootstrap source must have a regular-file native plan role')
        require(len(entries)==1,'bootstrap-source profile role must be one file')
        path,items=next(iter(entries.items())); items.sort(key=lambda e:-1 if e.get('chunk_index') is None else int(e['chunk_index']))
        require(path=='native-source-closure.json', 'bootstrap source role has an unexpected file path')
        size,digest=hash_embedded_file(tool,items)
        return {'role':'native-source-bundle','profile_class':klass,'record_identity':record['identity'],
                'record_content_blake3':record['content_blake3'],'canonical_record_body_verified':True,
                'input_role_path':path,'input_role_raw_blake3':digest,'input_role_bytes':size,
                'record_file_entries_expected':len(record['files']),'record_file_entries_covered':len(record['files']),
                'plan_comparison':'distinct input role: assembled native-source-bundle plan closure is independently bound by the current identity audit.'}
    require(planrow is not None,f'{klass}: source role missing from preauthor plan')
    require(planrow['kind']==('directory' if klass in DIRECTORY_CLASSES else 'regular-file'),
            f'{klass}: unrecognized source role kind {planrow["kind"]!r}')
    expected=planrow['digest_blake3']; expected_size=int(planrow['size_bytes'])
    if planrow['kind']=='regular-file':
        require(len(entries)==1,f'{klass}: regular-file role has multiple paths')
        path,items=next(iter(entries.items())); items.sort(key=lambda e:-1 if e.get('chunk_index') is None else int(e['chunk_index']))
        size,digest=hash_embedded_file(tool,items)
        matches=digest==expected and size==expected_size
        return {'role':planrow['role'],'profile_class':klass,'record_identity':record['identity'],
                'record_content_blake3':record['content_blake3'],'canonical_record_body_verified':True,
                'physical_file_path':path,'preauthor_digest_blake3':digest,'preauthor_bytes':size,
                'plan_digest_blake3':expected,'plan_bytes':expected_size,'matches_plan':matches,
                'record_file_entries_expected':len(record['files']),'record_file_entries_covered':len(record['files'])}
    tree=materialized_tree_digest(tool,record)
    matches=tree['digest_blake3']==expected and tree['payload_file_bytes']==expected_size
    return {'role':planrow['role'],'profile_class':klass,'record_identity':record['identity'],
            'record_content_blake3':record['content_blake3'],'canonical_record_body_verified':True,
            'preauthor_tree':tree,'plan_digest_blake3':expected,'plan_bytes':expected_size,'matches_plan':matches,
            'record_file_entries_expected':len(record['files']),'record_file_entries_covered':len(record['files'])}

def main():
    require(all(path.is_absolute() for path in (COHORT,FROZEN,SCRATCH,SUBJECT_PATH)), 'absolute review paths required')
    require(not SCRATCH.is_symlink() and not SCRATCH.resolve().is_relative_to(COHORT.resolve())
            and not SCRATCH.resolve().is_relative_to(FROZEN.resolve()), 'scratch must be outside immutable source/cohort')
    SCRATCH.mkdir(parents=True,exist_ok=True)
    subject=load_json(SUBJECT_PATH); run_root=Path(subject['run']); cohort=COHORT
    require(str(cohort)==subject['cohort'] and str(FROZEN)==subject['frozen_source'], 'explicit roots differ from subject')
    require(subject['frozen_product_revision']==PRODUCT_REVISION and run_root==RUN_ROOT
            and cohort==COHORT_ROOT and FROZEN==SOURCE_ROOT,
            'subject is not the published replacement frozen-source cohort')
    require(subject['original_aws']=={
        'capture':str(RUN_ROOT/'aws-original-source-02561621.TGISLN/aws-lc.git'),
        'commit':'02561621ffa4cf17c0c4f70bc11a82df36b42ae9',
        'git_tree_sha1':'969d75db5c0494de4a26bbb2dcbf6837058054c1',
        'normalized_tree_blake3':'8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89'},
        'original source pins differ from frozen inputs')
    require(cohort.is_dir() and not cohort.is_symlink() and FROZEN.is_dir() and not FROZEN.is_symlink(),
            'source/cohort root missing, non-directory or symlinked')
    tools=subject.get('inert_tools',{}); b3=tools.get('b3sum'); python=tools.get('python')
    require(isinstance(b3,str) and Path(b3).is_file() and isinstance(python,str) and Path(python).is_file(),
            'subject exact Python/BLAKE3 tool paths are missing')
    require(Path(sys.executable).resolve()==Path(python).resolve(),'Python binary differs from subject tool binding')
    plan_path=Path(subject['input_plan']); plan=load_json(plan_path)
    require(plan_path==cohort/'source-program-preparation-input-plan.json', 'plan path is outside declared cohort')
    plan_digest=canonical_plan_digest(b3,plan)
    require(plan_digest==subject['input_plan_blake3']==plan.get('plan_digest_blake3'),
            f'input plan mismatch: canonical={plan_digest} subject={subject["input_plan_blake3"]}')
    require(plan['source_inputs']==subject['source_inputs'] and plan['resource_bounds']==subject['resource_bounds'],
            'subject input/resource claims differ from actual plan')
    plan_rows=plan.get('source_inputs'); require(isinstance(plan_rows,list),'input plan lacks source_inputs')
    plan_roles={x['role']:x for x in plan_rows}
    expected_plan_roles=set(ROLE_CLASS_TO_PLAN_ROLE.values())
    require(set(plan_roles)==expected_plan_roles,f'plan role set mismatch: {sorted(plan_roles)}')
    require(len(plan_rows)==len(plan_roles)==len(expected_plan_roles), 'duplicate source role or unexpected source input')
    bounds=subject['resource_bounds']; record_limit=bounds['source_records_max']
    require(type(record_limit) is int and 0<record_limit<=65_536, 'invalid declared source record bound')
    require(cohort.parent==run_root and cohort.name.startswith('prepared-cohort-'), 'cohort is outside declared run')
    name=cohort.name[len('prepared-cohort-'):]
    require(name!='', 'empty cohort name')
    profile=run_root/f'source-profile-{name}.json'
    require(profile.is_file() and not profile.is_symlink(), 'current preauthor source profile is absent/nonregular')
    require(footer_manifest(profile)==subject['input_manifest_blake3'], 'preauthor profile manifest differs from subject')
    input_profile_raw=b3_file(b3,profile)
    profile_roles={}; all_record_count=0; nonrole_records=0
    identities=set()
    for record in iter_pretty_records(profile,record_limit):
        all_record_count+=1
        require(record['identity'] not in identities, f'duplicate input source record {record["identity"]}')
        identities.add(record['identity'])
        klass=record.get('metadata',{}).get('bootstrap_profile_class')
        require(klass is None or klass in ROLE_CLASS_TO_PLAN_ROLE, f'unknown preauthor profile role class {klass!r}')
        if klass in ROLE_CLASS_TO_PLAN_ROLE:
            require(klass not in profile_roles,f'duplicate preauthor source role class {klass}')
            profile_roles[klass]=inspect_role(b3,record,plan_roles[ROLE_CLASS_TO_PLAN_ROLE[klass]])
        else: nonrole_records+=1
    require(set(profile_roles)==set(ROLE_CLASS_TO_PLAN_ROLE),
            f'preauthor profile classes incomplete: missing={sorted(set(ROLE_CLASS_TO_PLAN_ROLE)-set(profile_roles))}')
    checks=[]
    for klass,role in ROLE_CLASS_TO_PLAN_ROLE.items():
        observed=profile_roles[klass]
        covered=bool(observed.get('canonical_record_body_verified')) if role=='native-source-bundle' else bool(observed.get('matches_plan'))
        leaf_count=observed['record_file_entries_expected']
        checks.append({'role':role,'profile_class':klass,'expected':1,'covered':int(covered),
                       'leaf_records_expected':leaf_count,'leaf_records_covered':observed['record_file_entries_covered'] if covered else 0,
                       'criterion':'input-role canonical body/payload (native plan closure checked separately)' if role=='native-source-bundle'
                                   else 'preauthor physical identity equals actual plan'})
    missing=[x['role'] for x in checks if x['expected']!=x['covered']]
    source_dir=Path(subject['frozen_source'])
    consumer_hashes={}
    for rel in ('src/release_tree_copy.rs','src/source_bundle.rs','src/source_built_fixed_point_shell.rs','src/source_built_fixed_point_planning.rs'):
        path=source_dir/rel
        require(path.is_file() and not path.is_symlink(),f'frozen identity consumer missing: {path}')
        consumer_hashes[rel]={'path':str(path),'bytes':path.stat().st_size,'blake3':b3_file(b3,path)}
    tool_hashes={'python':{'path':python,'blake3':b3_file(b3,Path(python))},
                 'b3sum':{'path':b3,'blake3':b3_file(b3,Path(b3))}}
    result={'schema':'mantle-preauthor-source-role-physical-review-v1',
        'bounded_physical_identity_verdict':'STATIC_PREAUTHOR_IDENTITIES_CLEAR' if not missing else 'STATIC_PREAUTHOR_IDENTITIES_BLOCKED',
        'scope':'Current preauthor physical identities only; bootstrap input role is distinct from the native plan closure bound by the main identity checker.',
        'subject_path':str(SUBJECT_PATH),'subject_blake3':b3_file(b3,SUBJECT_PATH),'subject_cohort':str(cohort),
        'input_plan':{'path':str(plan_path),'raw_blake3':b3_file(b3,plan_path),'canonical_blake3':plan_digest},
        'input_profile':{'path':str(profile),'bytes':profile.stat().st_size,'manifest_blake3':footer_manifest(profile),
                         'raw_blake3':input_profile_raw},
        'profile_records_decoded':all_record_count,'nonrole_records_seen':nonrole_records,
        'profile_roles_expected':len(ROLE_CLASS_TO_PLAN_ROLE),'profile_roles_covered':sum(x['covered'] for x in checks),
        'roles':profile_roles,'role_checks':checks,'uncovered_roles':missing,
        'coverage':{
            'preauthor_directory_roles':{'total':len(DIRECTORY_CLASSES),
                'covered':sum(x['covered'] for x in checks if x['profile_class'] in DIRECTORY_CLASSES),
                'uncovered':[x['role'] for x in checks if x['profile_class'] in DIRECTORY_CLASSES and not x['covered']]},
            'preauthor_file_role_bodies':{'total':len(checks)-len(DIRECTORY_CLASSES),
                'covered':sum(x['covered'] for x in checks if x['profile_class'] not in DIRECTORY_CLASSES),
                'uncovered':[x['role'] for x in checks if x['profile_class'] not in DIRECTORY_CLASSES and not x['covered']]},
            'preauthor_embedded_leaf_entries':{'total':sum(x['leaf_records_expected'] for x in checks),
                'covered':sum(x['leaf_records_covered'] for x in checks),
                'uncovered':[x['role'] for x in checks if x['leaf_records_covered'] != x['leaf_records_expected']]}},
        'tool_paths_and_hashes':tool_hashes,'frozen_consumer_paths_and_hashes':consumer_hashes,
        'inspector':{'path':str(Path(__file__).resolve()),'blake3':b3_file(b3,Path(__file__).resolve()),
                     'command':[sys.executable,str(Path(__file__).resolve()),*sys.argv[1:]]},
        'commands':{'record_and_tree_hash':'<exact subject inert b3sum> --no-names (streaming stdin)',
                    'plan_digest':'<exact subject inert b3sum> --derive-key mantle-source-built-fixed-point-plan-v2 --no-names'},
        'non_claims':['No multi-gigabyte materialized scratch copies were created; directory identity is reconstructed from exact leaf bytes, required parent paths, source modes, and frozen release_tree_copy ordering.',
                      'The bootstrap-source input role is measured separately from the assembled native-source-bundle plan closure.',
                      'Does not repeat output-profile, original Git, program archive/NAR, selected-source, helper-copy, or initial-authority checks.',
                      'No proof launch, protected execution, compiler/runtime/fixed-point/reproducibility/release/native-row claim.']}
    result['supplemental_criterion7_coverage']=result['coverage']['preauthor_directory_roles']
    out=SCRATCH/'preauthor-role-physical-review.json'
    with out.open('x',encoding='utf-8') as f: json.dump(result,f,ensure_ascii=False,sort_keys=True,indent=2); f.write('\n')
    print(json.dumps({'result_path':str(out),'bounded_physical_identity_verdict':result['bounded_physical_identity_verdict'],
                      'profile_roles_expected':len(ROLE_CLASS_TO_PLAN_ROLE),'profile_roles_covered':sum(x['covered'] for x in checks),
                      'profile_records_decoded':all_record_count,'role_checks':checks},sort_keys=True))
    if missing: raise SystemExit(2)

if __name__=='__main__':
    try:
        main()
    except Exception as error:
        print(f'BLOCKER {type(error).__name__}: {error}',file=sys.stderr)
        raise
