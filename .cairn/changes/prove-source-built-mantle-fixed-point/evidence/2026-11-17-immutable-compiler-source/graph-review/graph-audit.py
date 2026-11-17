#!/usr/bin/env python3
"""Read-only static graph/guard/typed-template audit for an explicitly supplied frozen cohort."""
import collections, hashlib, json, os, re, stat, subprocess, sys
from pathlib import Path

if len(sys.argv) != 5:
    raise SystemExit('usage: graph-audit.py COHORT FROZEN_SOURCE REVIEW_SCRATCH SUBJECT_JSON')
COHORT, SOURCE, SCRATCH = map(Path, sys.argv[1:4])
SUBJECT_PATH = Path(sys.argv[4])
SUBJECT = json.loads(SUBJECT_PATH.read_bytes())
SUBJECT_KEYS = {'authority','cohort','evidence_root','expected_native_provider_blake3','expected_stagex_lineage_blake3','frozen_complete_source_blake3','frozen_product_revision','frozen_source','full_profile_ssh_stream_blake3','host','inert_tools','input_manifest_blake3','input_plan','input_plan_blake3','original_aws','prior_reference_identity_result','prior_reference_inspector','preparation_report','preparation_report_raw_blake3','prepared_manifest_blake3','profile_file','profile_file_bytes','profile_file_raw_blake3','profile_transport_equal','program_records','resource_bounds','run','runner','runner_blake3','source_inputs'}
SUBJECT_KEYS.add('prepared_native_source_manifest_raw_blake3')
if not isinstance(SUBJECT,dict) or set(SUBJECT)!=SUBJECT_KEYS:
    raise SystemExit('SUBJECT_JSON must use the shared review-subject.json key schema exactly')
if not isinstance(SUBJECT.get('source_inputs'),list) or any(not isinstance(x,dict) or set(x)!={'id','role','kind','digest_blake3','size_bytes'} for x in SUBJECT['source_inputs']):
    raise SystemExit('SUBJECT_JSON source_inputs must retain the shared id/role/kind/digest_blake3/size_bytes shape')
B3 = '/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
PYTHON = sys.executable
STAGES = ['stagex-transition','stagex-provider-publication','full-source-native-provider','full-source-rust-provider','mantle-stage1','mantle-stage2']
CONSUMERS = [
 'src/source_built_recipe.rs','src/source_built_recipe_exec.rs','src/source_built_recipe_controller.rs',
 'src/source_built_fixed_point_stagex_actions.rs','src/source_built_fixed_point_native_actions.rs',
 'src/source_built_fixed_point_rust_actions.rs','src/source_built_fixed_point_mantle_actions.rs',
 'src/source_built_fixed_point_cold.rs','src/source_built_fixed_point_action_trust.rs',
 'src/source_built_fixed_point_action_controller.rs','src/source_built_fixed_point_authoring.rs',
 'src/source_built_stagex_programs_controller.rs','src/source_built_mantle_programs.rs','src/source_built_mantle_programs_graph.rs',
 'src/source_built_mantle_programs_metadata_helper.rs','src/source_built_rust_programs.rs','src/source_built_rust_programs_translation.rs',
 'src/source_built_rust_programs_minicargo.rs','src/source_built_rust_programs_minicargo_resolve.rs','src/source_built_rust_programs_successors.rs',
 'src/source_built_rust_programs_successors_profile.rs','src/source_built_rust_programs_successors_invocation.rs',
 'src/source_built_rust_programs_successors_graph.rs','src/source_built_stagex_programs_controller_paths.c',
 'src/source_built_stagex_programs_controller_protocol.c','src/source_built_stagex_programs_transform.c',
 'src/source_bundle.rs','src/source_built_native_programs_host_root.rs','src/source_built_native_programs_host_controller.rs','src/source_built_native_programs_host_nested.c',
]

def b3(path):
    return subprocess.check_output([B3,'--no-names',str(path)],text=True).strip()
def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(1<<20),b''): h.update(b)
    return h.hexdigest()
def load(path): return json.loads(path.read_bytes())
def key(ref): return json.dumps(ref,sort_keys=True,separators=(',',':'))
def normal_rel(p,empty=True):
    if not isinstance(p,str) or (not p and not empty) or len(p)>4096 or '\0' in p or '\\' in p: return False
    if not p: return empty
    return not p.startswith('/') and all(x not in ('','.','..') for x in p.split('/'))
def out_kind(step, name):
    x=step.get('outputs',{}).get(name)
    return x.get('kind') if x else None
def used_refs(step):
    op=step['operation']; kind=op['kind']; inputs=step.get('inputs',{}); names=set(); direct=set()
    if kind=='controller':
        spec=op['specification']; direct.add(key(spec['controller'])); direct.update(key(x) for x in spec.get('inputs',{}).values())
        for slot in spec.get('slots',[]):
            direct.update(key(x) for x in slot.get('inputs',{}).values())
            ex=slot['executable']; variant=ex['kind']; val=ex.get('value')
            if variant=='fixed': direct.add(key(val))
            elif variant=='adapted': direct.update((key(val['executable']),key(val['original_image'])))
            elif variant=='inventory': direct.update(key(pair[1]) for pair in val)
            elif variant=='request-output': pass
            else: raise ValueError('unknown ControllerExecutable '+variant)
        guard=op.get('guard')
        if guard: direct.update(key(inputs[c['status_input']]) for c in guard['conditions'])
        return direct
    if kind=='exec':
        names.add(op['executable_input'])
        names.update(n for n in (op.get('cwd_input'),op.get('stdin_input'),op.get('invocation_input')) if n)
        names.update(op.get('runtime_mounts',{}).values()); names.update(op.get('output_seeds',{}).values())
        guard=op.get('guard')
        if guard: names.update(c['status_input'] for c in guard['conditions'])
        def response(w):
            p=w.get('parts',[])
            return len(p)==2 and p[0].get('kind')=='literal' and p[0].get('text')=='@' and p[1].get('kind')=='input'
        dynamic=bool(op.get('invocation_input')) or any(response(w) for w in op.get('arguments',[]))
        if dynamic: names.update(inputs)
        runtime=op.get('runtime_mounts',{})
        for w in list(op.get('arguments',[]))+list(op.get('environment',{}).values()):
            for p in w.get('parts',[]):
                if p.get('kind') in ('input','text_input','text_slice'): names.add(p['name'])
                elif p.get('kind')=='runtime' and p.get('mount') in runtime: names.add(runtime[p['mount']])
    elif kind in ('copy-text','extract-archive'): names.add(op['input'])
    elif kind=='assemble-tree': names.update(op.get('entries',{}).values())
    elif kind=='materialize-fetcher':
        role, ident=op['role'],op['record_identity']
        for name,r in inputs.items():
            if (r.get('kind')=='manifest-source' and r.get('role')==role and r.get('record_identity')==ident and r.get('member')=='') or (r.get('kind')=='source' and r.get('role')==role and r.get('path')==''): names.add(name)
    elif kind=='observe-phase':
        if op.get('state'): names.add(op['state']['skipped_input'])
    elif kind in ('write-text','materialize-record'): pass
    else: raise ValueError('unknown operation '+kind)
    return {key(inputs[n]) for n in names if n in inputs}|direct

def words_from_word(w): return [w]
def record_ref_issues(ref,stage,step_id,step_idx,owner,prior,stage_idx,origin,issues):
    k=ref.get('kind')
    if k in ('source','manifest-source'):
        if k=='source':
            if not isinstance(ref.get('role'),str) or not ref.get('role') or not normal_rel(ref.get('path',''),True): issues.append({'where':origin,'error':'invalid source scope/member','ref':ref})
        else:
            if not isinstance(ref.get('record_identity'),str) or not ref.get('record_identity') or not normal_rel(ref.get('member',''),True): issues.append({'where':origin,'error':'invalid manifest source identity/member','ref':ref})
        return
    if k!='produced':
        issues.append({'where':origin,'error':'unknown artifact reference kind','ref':ref}); return
    aid=ref.get('action_id'); oid=ref.get('output_id')
    # Validate full stage namespace before deriving a local step identity.
    m=re.fullmatch(r'([A-Za-z0-9_.:-]+)/([A-Za-z0-9_.:-]+)',aid) if isinstance(aid,str) else None
    if not m or not isinstance(oid,str) or not oid:
        issues.append({'where':origin,'error':'malformed producer namespace/output','ref':ref}); return
    producer_stage=m.group(1); producer_local=m.group(2)
    if producer_stage==stage:
        ps=owner.get(aid)
        if ps is None:
            issues.append({'where':origin,'error':'unknown local producer','ref':ref}); return
        if oid not in ps['outputs']:
            issues.append({'where':origin,'error':'unknown local output','ref':ref}); return
        if step_idx is not None and ps['index']>=step_idx:
            issues.append({'where':origin,'error':'late/cyclic local producer','ref':ref})
        return
    if producer_stage not in STAGES or STAGES.index(producer_stage)>=stage_idx:
        issues.append({'where':origin,'error':'foreign, same/future-stage, or undeclared producer namespace','ref':ref}); return
    ps=prior.get(producer_stage,{}).get(aid)
    if ps is None or oid not in ps['outputs']:
        issues.append({'where':origin,'error':'unowned prior-stage output','ref':ref})

SOURCE_KIND_CACHE={}
SOURCE_ROLE_ROOTS={}
SOURCE_RECORD_INVENTORIES={}
SOURCE_MANIFEST_OBSERVATIONS={}

def checked_source_path(root,member):
    if not normal_rel(member,True): raise ValueError('noncanonical source member')
    path=root
    for part in ('',*member.split('/')) if member else ('',):
        if part: path=path/part
        mode=path.lstat().st_mode
        if stat.S_ISLNK(mode): raise ValueError('source member traverses a symlink')
        if not (stat.S_ISREG(mode) or stat.S_ISDIR(mode)): raise ValueError('source member has unsupported filesystem kind')
    return path

def source_role_root(role):
    if role in SOURCE_ROLE_ROOTS: return SOURCE_ROLE_ROOTS[role]
    admitted=[x for x in SUBJECT['source_inputs'] if x['role']==role]
    if len(admitted)!=1: raise ValueError('source role is not uniquely admitted')
    directories={'mantle-source':'mantle-source','native-source-bundle':'native-source-authority',
        'rust-source-archive-set':'rust-source-archives','stagex-lineage':'stagex-lineage',
        'stagex-seed':'stagex-seed','stagex-source-bundle':'stagex-source-bundle','vendor-inputs':'vendor-deps'}
    root=checked_source_path(COHORT,'inputs/'+directories[role])
    if admitted[0]['kind']=='regular-file':
        # cold_source_roots selects the sole regular leaf, never its containing Tree.
        leaves=[]; pending=[root]
        while pending:
            path=pending.pop(); mode=path.lstat().st_mode
            if stat.S_ISLNK(mode): raise ValueError('symlink in single-file source authority')
            if stat.S_ISREG(mode): leaves.append(path)
            elif stat.S_ISDIR(mode): pending.extend(path.iterdir())
            else: raise ValueError('unsupported single-file source authority')
            if len(leaves)>1: raise ValueError('source role does not have exactly one file')
        if len(leaves)!=1: raise ValueError('source role has no sole physical file')
        root=leaves[0]
    elif admitted[0]['kind']!='directory' or not root.is_dir():
        raise ValueError('source role root filesystem class differs')
    SOURCE_ROLE_ROOTS[role]=root
    return root

def source_record_inventories(role):
    if role in SOURCE_RECORD_INVENTORIES: return SOURCE_RECORD_INVENTORIES[role]
    path=source_role_root(role)
    if not path.is_file(): raise ValueError('ManifestSource authority is not a manifest file')
    inventories={}; current=bytearray(); in_records=False; in_record=False; ended=False; manifest_identity=None
    with path.open('rb',buffering=1024*1024) as stream:
        for line in stream:
            if not in_records:
                if re.fullmatch(rb'\s*"records"\s*:\s*\[\s*',line): in_records=True
                continue
            if not in_record:
                if line in (b'    {\n',b'    {\r\n'): current=bytearray(line); in_record=True
                elif line.lstrip().startswith(b']'): ended=True; break
                continue
            current.extend(line)
            if len(current)>20*1024**3: raise ValueError('source record exceeds bounded inspector parser')
            if line not in (b'    },\n',b'    },\r\n',b'    }\n',b'    }\r\n'): continue
            record=json.loads(current.rstrip().removesuffix(b','))
            identity=record['identity']; files=record['files']; leaves={}; parents=set(); file_entries=collections.defaultdict(list)
            if identity in inventories or not files: raise ValueError('duplicate/empty source record')
            for entry in files:
                member=entry['path']; typ=entry['file_type']
                if not normal_rel(member,False): raise ValueError('noncanonical source record leaf')
                if member in leaves and (leaves[member]!=typ or entry.get('chunk_index') is None):
                    raise ValueError('duplicate source member not a consistent canonical chunk')
                leaves[member]=typ
                file_entries[member].append({k:entry[k] for k in ('file_type','blake3','chunk_index','chunk_count') if k in entry})
                parent=Path(member).parent
                while str(parent)!='.': parents.add(parent.as_posix()); parent=parent.parent
            inventories[identity]={'leaves':leaves,'parents':parents,'content_blake3':record['content_blake3'],
                'record_kind':record['kind'],'entry_count':len(files),'metadata':record.get('metadata',{}),'file_entries':dict(file_entries)}
            if len(inventories)>SUBJECT['resource_bounds']['source_records_max']: raise ValueError('source record count exceeds declared bound')
            in_record=False; current=bytearray(); del record,files
        for line in stream:
            match=re.fullmatch(rb'  "manifest_blake3": "([0-9a-f]{64})",?\r?\n?',line)
            if match:
                if manifest_identity is not None: raise ValueError('duplicate source manifest identity field')
                manifest_identity=match.group(1).decode()
    if not in_records or in_record or not ended: raise ValueError('truncated/unrecognized source record stream')
    SOURCE_MANIFEST_OBSERVATIONS[role]={'path':str(path),'size_bytes':path.stat().st_size,'blake3':b3(path),
        'record_count':len(inventories),'manifest_blake3':manifest_identity,'consumer':'hydrate_reference -> materialize_source_record_payload -> lower_step'}
    SOURCE_RECORD_INVENTORIES[role]=inventories
    return inventories

MANTLE_RECORD = 'source-built-mantle-programs'
MANTLE_ROLE = 'native-source-bundle'
MANTLE_MEMBERS = ('bootstrap/source-built/mantle-stage1.json', 'bootstrap/source-built/mantle-stage2.json')
ROUTE_MEMBER = 'bootstrap/source-built-cold-route.json'
PLAN_MEMBERS = tuple(f'bootstrap/source-built/{stage}.unit-plan.json' for stage in ('mantle-stage1','mantle-stage2'))

def exact_program_ref(ref, member):
    expected = {'kind':'manifest-source','role':MANTLE_ROLE,'record_identity':MANTLE_RECORD,'member':member}
    if ref != expected: raise ValueError(f'foreign, legacy or malformed Mantle program selector: {ref!r}')
    return expected

def program_member_path(member):
    if member not in (ROUTE_MEMBER,*MANTLE_MEMBERS,*PLAN_MEMBERS):
        raise ValueError('member is not an exact v2 program artifact')
    ref={'kind':'manifest-source','role':MANTLE_ROLE,'record_identity':MANTLE_RECORD,'member':member}
    inventory=source_record_inventories(MANTLE_ROLE).get(MANTLE_RECORD)
    if not inventory or inventory['record_kind']!='fixed-url' or inventory['content_blake3'] != SUBJECT['program_records'].get(MANTLE_RECORD):
        raise ValueError('Mantle program record is not the sealed admitted fixed-url record')
    if source_kind_resolution(ref)['resolved_artifact_kind']!='text':
        raise ValueError('Mantle program member is not an authenticated regular leaf')
    entries=inventory['file_entries'][member]
    if len(entries)==1:
        if entries[0].get('chunk_index') is not None or entries[0].get('chunk_count') is not None:
            raise ValueError('noncanonical single program member chunk')
    elif len(entries)<2 or [entry.get('chunk_index') for entry in entries]!=list(range(len(entries))) or any(entry.get('chunk_count')!=len(entries) for entry in entries):
        raise ValueError('noncanonical program member chunks')
    path=checked_source_path(COHORT/'program-sources'/MANTLE_RECORD,member)
    if not path.is_file() or len(entries)==1 and b3(path)!=entries[0]['blake3']:
        raise ValueError('hydrated program member differs from authenticated leaf')
    return path

def selected_route():
    path=program_member_path(ROUTE_MEMBER)
    route=load(path)
    if not isinstance(route,dict) or set(route)!={'schema','stagex_transition','stagex_publication','native','rust_provider','mantle_recipes'} or route['schema']!='mantle-source-built-cold-route-v2':
        raise ValueError('cold route is not the exact v2 schema')
    refs=route['mantle_recipes']
    if not isinstance(refs,list) or len(refs)!=2:
        raise ValueError('cold route must contain two exact Mantle program references')
    for ref,member in zip(refs,MANTLE_MEMBERS,strict=True): exact_program_ref(ref,member)
    return path,route

def source_kind_resolution(ref):
    encoded=key(ref)
    if encoded in SOURCE_KIND_CACHE: return SOURCE_KIND_CACHE[encoded]
    result={'resolved_artifact_kind':None,'basis':None}
    try:
        root=source_role_root(ref['role'])
        if ref['kind']=='source':
            path=checked_source_path(root,ref['path']); mode=path.lstat().st_mode
            result.update(resolved_artifact_kind='tree' if stat.S_ISDIR(mode) else 'text',path=str(path),
                basis='observed exact source member; lower_step maps directories to Tree and regular files to Text')
        elif ref['kind']=='manifest-source':
            inventory=source_record_inventories(ref['role']).get(ref['record_identity'])
            if inventory is None: raise ValueError('ManifestSource record identity absent from actual admitted manifest')
            member=ref['member']
            if not normal_rel(member,True): raise ValueError('noncanonical ManifestSource member')
            prefixes=['/'.join(member.split('/')[:n]) for n in range(1,len(member.split('/'))+1)] if member else []
            if any(inventory['leaves'].get(prefix)=='symlink' for prefix in prefixes): raise ValueError('ManifestSource member traverses symlink')
            if not member or member in inventory['parents']: kind='tree'
            elif inventory['leaves'].get(member)=='regular': kind='text'
            else: raise ValueError('ManifestSource member absent or unsupported')
            result.update(resolved_artifact_kind=kind,record_content_blake3=inventory['content_blake3'],
                record_kind=inventory['record_kind'],manifest_path=str(root),member=member,
                basis='actual authenticated record leaf/parent inventory; materialize_source_record_payload creates a directory root; lower_step maps directory/file to Tree/Text')
    except (KeyError,OSError,TypeError,ValueError) as error:
        result['unresolved_reason']=str(error)
    SOURCE_KIND_CACHE[encoded]=result
    return result

def kind_for(ref,owner,prior,stage):
    if ref.get('kind')=='produced':
        aid=ref.get('action_id'); oid=ref.get('output_id')
        s=owner.get(aid)
        if s is None:
            for st,vals in prior.items():
                if aid in vals: s=vals[aid]; break
        return s['outputs'].get(oid,{}).get('kind') if s else None
    if ref.get('kind') in ('source','manifest-source'):
        return source_kind_resolution(ref)['resolved_artifact_kind']
    return None

def inspect_output_seeds(stage, step_id, inputs, outputs, seeds, owner, prior, step_index, guard=None, template_owner=None):
    """Mirror validate_operation and lower_step's Produced kind/bound seed admission."""
    failures=[]; rows=[]; guard_names={c.get('status_input') for c in (guard or {}).get('conditions',[])}
    for output, name in seeds.items():
        where=f'{stage}/{step_id}.output_seeds.{output}'
        target=outputs.get(output); ref=inputs.get(name) if isinstance(name,str) else None; source=None; reasons=[]
        if not isinstance(target,dict) or output in ('stdout','stderr','exit-status'):
            reasons.append('seed target is not a declared ordinary output')
        if not isinstance(name,str) or not isinstance(ref,dict) or ref.get('kind')!='produced' or name in guard_names:
            reasons.append('seed input is not a genuine nonguard Produced reference')
        else:
            aid=ref.get('action_id'); oid=ref.get('output_id')
            match=re.fullmatch(r'([A-Za-z0-9_.:-]+)/([A-Za-z0-9_.:-]+)',aid) if isinstance(aid,str) else None
            if not match or not isinstance(oid,str) or not oid:
                reasons.append('malformed full producer namespace/output')
            else:
                namespace=match.group(1); producer=None
                if template_owner is not None and namespace=='controller-validation':
                    producer=template_owner.get(aid)
                elif namespace==stage:
                    producer=owner.get(aid)
                    if producer is not None and producer.get('index',step_index)>=step_index:
                        reasons.append('seed producer is not a prior local declaration')
                elif namespace in STAGES[:STAGES.index(stage)]:
                    producer=prior.get(namespace,{}).get(aid)
                else:
                    reasons.append('seed producer is outside permitted local/prior scope')
                if producer is None:
                    reasons.append('exact seed producer is unknown')
                else:
                    source=producer.get('outputs',{}).get(oid)
                    if not isinstance(source,dict): reasons.append('exact seed producer output is unknown')
        if isinstance(source,dict) and isinstance(target,dict):
            kinds={'text','object','archive','executable','tree'}
            if not isinstance(source.get('kind'),str) or source['kind'] not in kinds or not isinstance(target.get('kind'),str) or target['kind'] not in kinds:
                reasons.append('source or destination artifact kind is unknown')
            elif source['kind']!=target['kind']:
                reasons.append('Produced source and destination artifact kinds differ')
            src_bound=source.get('bytes_max'); dst_bound=target.get('bytes_max')
            if any(type(bound) is not int or not 0<=bound<1<<64 for bound in (src_bound,dst_bound)):
                reasons.append('source or destination bytes_max is unresolved/invalid')
            elif src_bound>dst_bound:
                reasons.append('Produced source bytes_max exceeds destination bytes_max')
        elif not isinstance(source,dict) and 'exact seed producer output is unknown' not in reasons:
            reasons.append('source Produced declaration and bound unresolved')
        row={'where':where,'input':name,'reference':ref,'source_declaration':source,'output_declaration':target,'failure_reasons':reasons}
        rows.append(row)
        if reasons: failures.append(row)
    def measured_kind(declaration):
        kind=declaration.get('kind') if isinstance(declaration,dict) else None
        return kind if isinstance(kind,str) else 'unresolved'
    return {'total':len(rows),'covered':len(rows)-len(failures),'invalid':len(failures),
            'source_kind_counts':dict(collections.Counter(measured_kind(row['source_declaration']) for row in rows)),
            'destination_kind_counts':dict(collections.Counter(measured_kind(row['output_declaration']) for row in rows)),
            'producer_namespace_counts':dict(collections.Counter(
                row['reference']['action_id'].split('/',1)[0]
                if isinstance(row['reference'],dict) and isinstance(row['reference'].get('action_id'),str)
                else '<unresolved>' for row in rows)),
            'failure_reason_counts':dict(collections.Counter(reason for row in failures for reason in row['failure_reasons'])),
            'failure_examples':failures[:100],
            'consumer':'src/source_built_recipe.rs::validate_operation output_seeds Produced/ordinary/nonguard; lower_step compares exact binding.kind and binding.bytes_max <= output.bytes_max'}

def static_word_errors(w,ctx,step,inputs,outputs,runtime,inv=None,owner=None,prior=None,stage=None,missing_kinds=None):
    errors=[]; parts=w.get('parts',[])
    if not parts or len(parts)>4096: return ['empty/excessive word parts']
    size=0; workspaces=(inv or {}).get('workspace_mounts',{})
    def typed_kind(p):
        k=p.get('kind')
        if k=='input':
            if p.get('name') not in inputs: errors.append('unknown Input name'); return None
            r=inputs[p['name']]
            if p.get('name') in (ctx.get('guard_names') or set()): errors.append('guard status used as ordinary word')
            path=p.get('path','')
            if not normal_rel(path,True): errors.append('noncanonical Input member')
            typ=kind_for(r,owner or {},prior or {},stage)
            if path and typ!='tree':
                if typ is None: missing_kinds.append({'where':ctx['where'],'part':p,'reason':'Input root kind unavailable','category':'unresolved-binding-kind'})
                else: errors.append('Input member is not rooted at Tree')
            return typ
        if k=='output':
            name=p.get('name'); path=p.get('path','')
            if name not in outputs: errors.append('unknown Output name'); return None
            if name=='exit-status': errors.append('exit-status used as Output path')
            state=(step.get('operation') or {}).get('command_state') or {}
            if name in (state.get('times_output'),state.get('descriptors_output'),state.get('descriptor_files_output')):
                errors.append('executor-owned command-state output used as command path')
            if not normal_rel(path,True): errors.append('noncanonical Output member')
            typ=outputs[name].get('kind')
            if path and typ!='tree': errors.append('Output member is not a Tree')
            return typ
        if k=='runtime':
            name=p.get('mount'); path=p.get('path','')
            if name not in runtime: errors.append('unknown Runtime mount'); return None
            if not normal_rel(path,True): errors.append('noncanonical Runtime member')
            typ=kind_for(inputs.get(runtime[name],{}),owner or {},prior or {},stage)
            if path and typ!='tree':
                if typ is None: missing_kinds.append({'where':ctx['where'],'part':p,'reason':'Runtime root kind unavailable','category':'unresolved-binding-kind'})
            return typ
        if k=='workspace':
            name=p.get('mount'); path=p.get('path',''); m=workspaces.get(name)
            if m is None: errors.append('unknown Workspace mount'); return None
            if not normal_rel(path,True): errors.append('noncanonical Workspace member')
            typ=outputs.get(m.get('output'),{}).get('kind')
            if m.get('output') not in outputs: errors.append('Workspace output missing')
            elif typ!='tree': errors.append('Workspace root is not a Tree')
            return typ
        return None
    for i,p in enumerate(parts):
        k=p.get('kind')
        if k=='literal':
            t=p.get('text',''); size+=len(t.encode());
            if '/' in t or '\\' in t or '\0' in t: errors.append('slash-bearing/NUL Literal')
        elif k=='literal-data':
            t=p.get('text',''); size+=len(t.encode())
            if '\0' in t: errors.append('NUL LiteralData')
        elif k in ('input','output','runtime','workspace'):
            typed_kind(p)
        elif k=='cwd_relative':
            path=p.get('path',''); member=path[2:] if path.startswith('./') else path
            if not normal_rel(member,False) or len(parts)!=1 or not (inv or {}).get('cwd'): errors.append('invalid CwdRelative/authenticated cwd')
        elif k=='directory_separator':
            valid=i>0 and (i+1==len(parts) or (parts[i+1].get('kind')=='literal' and parts[i+1].get('text') in (':',','))) and parts[i-1].get('kind') in ('input','output','runtime','workspace')
            if not valid: errors.append('invalid DirectorySeparator boundary')
            else:
                typ=typed_kind(parts[i-1]); size+=1
                if typ!='tree':
                    if typ is None: missing_kinds.append({'where':ctx['where'],'part':parts[i-1],'reason':'DirectorySeparator root kind unavailable','category':'unresolved-binding-kind'})
                    else: errors.append('DirectorySeparator root is not a Tree')
        elif k in ('text_input','text_slice'):
            name=p.get('name'); ref=inputs.get(name)
            if ref is None: errors.append('unknown TextInput/TextSlice input')
            elif ref.get('kind')!='produced' or ref.get('output_id')=='exit-status': errors.append('TextInput/TextSlice is not ordinary Produced')
            else:
                kind=kind_for(ref,owner or {},prior or {},stage)
                if kind!='text':
                    if kind is None: missing_kinds.append({'where':ctx['where'],'part':p,'reason':'text input output kind unavailable','category':'unresolved-binding-kind'})
                    else: errors.append('TextInput/TextSlice producer is not Text')
            if k=='text_slice' and (p.get('offset_bytes',0)+p.get('length_bytes',0)>64*1024*1024 or p.get('length_bytes',0)>65536): errors.append('TextSlice exceeds bound')
        else: errors.append('unknown WordPart variant')
    if size>65536: errors.append('word exceeds byte bound')
    return errors

def recipe_words(recipe,owner,prior,stage_idx):
    words=[]; unknown=[]; issues=[]; counts=collections.Counter(); all_inv=[]; controllers=[]; invocation_input_count=0
    stepmap={s['id']:s for s in recipe['steps']}
    for step in recipe['steps']:
        op=step['operation']; guard=op.get('guard') or {}; base={'where':f"{recipe['stage_id']}/{step['id']}",'guard_names':{c['status_input'] for c in guard.get('conditions',[])}}
        inputs=step.get('inputs',{}); outputs=step.get('outputs',{}); runtime=op.get('runtime_mounts',{}) if op['kind']=='exec' else {}
        if op['kind']=='exec':
            for arg in op.get('arguments',[]): words.append((step,arg,base,inputs,outputs,runtime,None))
            for name,arg in op.get('environment',{}).items(): words.append((step,arg,{**base,'where':base['where']+f'.env.{name}'},inputs,outputs,runtime,None))
            if op.get('invocation_input'):
                invocation_input_count+=1
                ref=inputs.get(op['invocation_input']); producer=owner.get(ref.get('action_id','')) if ref and ref.get('kind')=='produced' else None
                if producer:
                    pstep=stepmap.get(producer['local'])
                    if pstep and pstep['operation']['kind']=='write-text':
                        try:
                            req=json.loads(pstep['operation']['text'])
                            if not isinstance(req,dict): raise ValueError('invocation JSON is not an object')
                            all_inv.append((step,req,op.get('invocation_input')))
                        except Exception as e: issues.append({'where':base['where'],'error':'static invocation JSON decode failed','detail':str(e)})
                    else: unknown.append({'where':base['where'],'request_ref':ref,'producer_op':pstep['operation']['kind'] if pstep else 'prior-or-unknown','category':'runtime-symbolic-invocation'})
                else: unknown.append({'where':base['where'],'request_ref':ref,'producer_op':'external-or-unowned','category':'runtime-symbolic-invocation'})
        elif op['kind']=='controller':
            spec=op['specification']; controllers.append((step,spec))
            for arg in spec.get('start_arguments',[]): words.append((step,arg,{**base,'where':base['where']+'.controller.start'},inputs,outputs,{},None))
            for name,arg in spec.get('environment',{}).items(): words.append((step,arg,{**base,'where':base['where']+f'.controller.env.{name}'},inputs,outputs,{},None))
    for step,req,name in all_inv:
        op=step['operation']; req_words=[]
        allowed={'arguments','environment','cwd','argv0','executable_path','workspace_mounts','readonly_input_roots','process_attributes','descriptors'}
        if not {'arguments','environment'}.issubset(req) or set(req)-allowed:
            issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'static RecipeInvocation fields differ from consumer schema','missing':sorted({'arguments','environment'}-set(req)),'unknown':sorted(set(req)-allowed)})
        args=req.get('arguments',[]); env=req.get('environment',{})
        if not isinstance(args,list): issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invocation arguments are not an array'}); args=[]
        if not isinstance(env,dict): issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invocation environment is not a map'}); env={}
        if any(not isinstance(word,dict) for word in args): issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invocation argument is not a typed RecipeWord'}); args=[word for word in args if isinstance(word,dict)]
        if any(not isinstance(name,str) or not isinstance(word,dict) for name,word in env.items()): issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invocation environment entry is not a typed name/RecipeWord'}); env={name:word for name,word in env.items() if isinstance(name,str) and isinstance(word,dict)}
        attrs=req.get('process_attributes')
        if attrs is not None:
            allowed_attrs={'file_size_bytes_max','umask','timeout_scopes'}
            scopes=attrs.get('timeout_scopes',[]) if isinstance(attrs,dict) else []
            if not isinstance(attrs,dict) or set(attrs)-allowed_attrs or attrs.get('file_size_bytes_max')==0 or attrs.get('umask') is not None and attrs['umask']>0o777 or not isinstance(scopes,list) or len(scopes)>64:
                issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invalid process attributes or timeout scope count'})
            seen_scopes=set()
            for scope in scopes:
                if not isinstance(scope,dict) or set(scope)!={'id','seconds_max','monotonic_deadline_ns'} or not normal_rel(scope.get('id'),False) or scope.get('seconds_max')==0 or scope.get('monotonic_deadline_ns') is None or scope.get('id') in seen_scopes:
                    issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invalid or duplicate original timeout scope'})
                if isinstance(scope,dict): seen_scopes.add(scope.get('id'))
        for a in args: req_words.append((step,a,{'where':f"{recipe['stage_id']}/{step['id']}.invocation.argument"},step['inputs'],step['outputs'],op.get('runtime_mounts',{}),req))
        for envname,a in env.items(): req_words.append((step,a,{'where':f"{recipe['stage_id']}/{step['id']}.invocation.env.{envname}"},step['inputs'],step['outputs'],op.get('runtime_mounts',{}),req))
        for field in ('cwd','argv0','executable_path'):
            if req.get(field) is not None: req_words.append((step,req[field],{'where':f"{recipe['stage_id']}/{step['id']}.invocation.{field}"},step['inputs'],step['outputs'],op.get('runtime_mounts',{}),req))
        for slot in (req.get('descriptors') or {}).get('slots',[]):
            if slot and slot.get('target',{}).get('kind')=='file': req_words.append((step,slot['target']['path'],{'where':f"{recipe['stage_id']}/{step['id']}.invocation.descriptor"},step['inputs'],step['outputs'],op.get('runtime_mounts',{}),req))
        words.extend(req_words)
        used={op['executable_input'],name}; used.update(op.get('runtime_mounts',{}).values()); used.update(x for x in (op.get('cwd_input'),op.get('stdin_input')) if x); used.update(op.get('output_seeds',{}).values())
        if op.get('guard'): used.update(c['status_input'] for c in op['guard']['conditions'])
        used.update(req.get('readonly_input_roots',[]))
        for _,w,_,_,_,_,_ in req_words:
            for part in w.get('parts',[]):
                if part.get('kind') in ('input','text_input','text_slice'): used.add(part.get('name'))
                if part.get('kind')=='runtime': used.add(op.get('runtime_mounts',{}).get(part.get('mount')))
        unused=set(step['inputs'])-used
        if unused: issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'dynamic invocation has unused declared inputs','names':sorted(unused)[:20],'count':len(unused)})
        wsm=req.get('workspace_mounts',{})
        for dest,m in wsm.items():
            if not dest.startswith('/') or not normal_rel(dest[1:],False) or m.get('output') not in step['outputs'] or step['outputs'][m['output']]['kind']!='tree' or not normal_rel(m.get('path',''),True):
                issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invalid workspace mount','destination':dest,'mapping':m})
        destinations=list(wsm)
        def overlap(left,right): return left==right or left.startswith(right.rstrip('/')+'/') or right.startswith(left.rstrip('/')+'/')
        for i,dest in enumerate(destinations):
            if any(overlap(dest,other) for other in destinations[i+1:]) or any(overlap(dest,mount) for mount in op.get('runtime_mounts',{})):
                issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'overlapping workspace/runtime mount destinations','destination':dest})
        cwd=req.get('cwd')
        if cwd is not None and (op.get('cwd_input') or op.get('cwd_output') or len(cwd.get('parts',[]))!=1 or cwd['parts'][0].get('kind') not in ('input','output','runtime','workspace')):
            issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invalid invocation cwd scope'})
        image=req.get('executable_path')
        if image is not None:
            image_ref=step['inputs'].get(op.get('executable_input'))
            parts=image.get('parts',[])
            if not image_ref or image_ref.get('kind')!='produced' or len(parts)!=1 or parts[0].get('kind') not in ('input','runtime') or not normal_rel(parts[0].get('path',''),False):
                issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invalid installed executable image scope'})
        table=req.get('descriptors')
        if table is not None:
            slots=table.get('slots',[])
            if not op.get('command_state') or len(slots)!=64:
                issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'descriptor table requires command state and exactly 64 slots'})
            for fd,entry in enumerate(slots):
                if entry is None: continue
                alias=entry.get('alias',-1); target=entry.get('target',{}); access=entry.get('access')
                if not isinstance(alias,int) or alias<0 or alias>fd or entry.get('offset',0)>((1<<63)-1) or entry.get('flags',0)&~(os.O_APPEND|os.O_NONBLOCK):
                    issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invalid descriptor alias, offset, or flags','fd':fd})
                if access not in ('read','write','read-write'):
                    issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invalid descriptor access','fd':fd})
                if alias!=fd and (alias>=len(slots) or slots[alias]!=entry):
                    issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'descriptor alias differs from primary OFD','fd':fd})
                if target.get('kind')=='file':
                    parts=target.get('path',{}).get('parts',[])
                    valid_access=(len(parts)==1 and ((parts[0].get('kind') in ('input','runtime') and access=='read') or parts[0].get('kind') in ('output','workspace')))
                    if not valid_access: issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'descriptor target lacks exact typed read/write authority','fd':fd})
                elif target.get('kind')!='null-device':
                    issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'unknown descriptor target variant','fd':fd})
        roots=req.get('readonly_input_roots',[])
        if roots!=sorted(set(roots)) or any(n not in step['inputs'] or step['inputs'][n].get('kind')!='produced' or kind_for(step['inputs'][n],owner,prior,stage_idx)!='tree' for n in roots): issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invalid readonly_input_roots'})
        if not isinstance(req.get('environment',{}),dict) or any(not n or not all(c.isascii() and (c.isalnum() or c=='_') for c in n) for n in req.get('environment',{})): issues.append({'where':f"{recipe['stage_id']}/{step['id']}",'error':'invalid invocation environment name'})
    for step,w,ctx,ins,outs,rt,inv in words:
        counts['words']+=1; counts['parts']+=len(w.get('parts',[]))
        errs=static_word_errors(w,ctx,step,ins,outs,rt,inv,owner,prior,stage_idx,unknown)
        for e in errs: issues.append({'where':ctx['where'],'error':e,'word':w})
        for p in w.get('parts',[]): counts['parts:'+p.get('kind','?')]+=1
    runtime_invocations=sum(x.get('category')=='runtime-symbolic-invocation' for x in unknown)
    unresolved_kinds=sum(x.get('category')=='unresolved-binding-kind' for x in unknown)
    return {'word_count':counts['words'],'word_part_count':counts['parts'],'part_kinds':{k[6:]:v for k,v in counts.items() if k.startswith('parts:')},'issues':issues,'unknown_binding_kinds':unknown,'invocation_count':len(all_inv),'invocation_input_count':invocation_input_count,'dynamic_invocation_uninspectable_count':runtime_invocations,'controller_count':len(controllers),'controllers':controllers,'invocations':all_inv,'unresolved_binding_kind_count':unresolved_kinds}

def make_owner(recipe,idx,prior,stage_idx,issues):
    owner={}; stepmap={s['id']:s for s in recipe['steps']}
    for i,s in enumerate(recipe['steps']):
        aid=recipe['stage_id']+'/'+s['id']
        if aid in owner: issues.append({'where':aid,'error':'duplicate action identity'})
        owner[aid]={'local':s['id'],'index':i,'outputs':s.get('outputs',{})}
    # Every full producer reference is validated before any local lookup or stripping.
    def walk_refs(obj,path,stepid=None,step_index=None):
        if isinstance(obj,dict):
            if obj.get('kind') in ('produced','source','manifest-source'):
                record_ref_issues(obj,recipe['stage_id'],stepid,step_index,owner,prior,stage_idx,path,issues); return
            for k,v in obj.items(): walk_refs(v,path+'.'+k,stepid,step_index)
        elif isinstance(obj,list):
            for i,v in enumerate(obj): walk_refs(v,f'{path}[{i}]',stepid,step_index)
    for i,s in enumerate(recipe['steps']):
        walk_refs(s.get('inputs',{}),f"{recipe['stage_id']}/{s['id']}.inputs",s['id'],i)
        # Controller catalog refs are embedded in operation and are all authority-bearing.
        if s['operation']['kind']=='controller': walk_refs(s['operation']['specification'],f"{recipe['stage_id']}/{s['id']}.controller",s['id'],i)
    for n,r in enumerate(recipe.get('roots',[])): walk_refs(r,f"{recipe['stage_id']}.root[{n}]")
    for n,r in recipe.get('exports',{}).items(): walk_refs(r,f"{recipe['stage_id']}.export.{n}")
    for n,rs in recipe.get('semantic_roots',{}).items(): walk_refs(rs,f"{recipe['stage_id']}.semantic.{n}")
    return owner,stepmap

def exits(step):
    op=step['operation']; pol=op.get('exit_policy',{})
    k=pol.get('kind')
    if k=='exact': return {pol['code']}
    if k=='one-of': return set(pol.get('codes',[]))
    if k=='normal': return set(range(256))
    return set()
def graph_guard_report(recipe,owner,stepmap,prior=None,stage_idx=None):
    prior=prior or {}
    stage=recipe['stage_id']; idx={s['id']:i for i,s in enumerate(recipe['steps'])}; issues=[]; guard_count=0; obs_count=0; status_edges=0; phase_cases=[]; optional_outputs=0
    actual={s['id']:used_refs(s) for s in recipe['steps']}
    # Fully namespaced local dependency edges only; no namespace normalization before checking it.
    graph={s['id']:set() for s in recipe['steps']}
    for s in recipe['steps']:
        for r in s.get('inputs',{}).values():
            if r.get('kind')=='produced' and r.get('action_id','').startswith(stage+'/') and r.get('action_id') in owner:
                producer=owner[r['action_id']]['local']
                graph[producer].add(s['id'])
        if s['operation']['kind']=='observe-phase':
            phase=s['operation']['phase']
            if phase in graph: graph[phase].add(s['id'])
    # Cycle check independent of execution-order predicate.
    color={n:0 for n in graph}; cycles=[]
    for root in graph:
        if color[root]: continue
        color[root]=1; active=[root]; positions={root:0}; stack=[(root,iter(sorted(graph[root])))]
        while stack:
            node,children=stack[-1]
            try: child=next(children)
            except StopIteration:
                stack.pop(); active.pop(); positions.pop(node,None); color[node]=2; continue
            if color[child]==0:
                color[child]=1; positions[child]=len(active); active.append(child); stack.append((child,iter(sorted(graph[child]))))
            elif color[child]==1:
                cycles.append(active[positions[child]:]+[child])
    for s in recipe['steps']:
        op=s['operation']; guard=op.get('guard')
        if guard:
            guard_count+=1; conds=guard.get('conditions',[])
            codes=[c.get('accepted_codes',[]) for c in conds]
            if not 1<=len(conds)<=8: issues.append({'step':s['id'],'error':'guard condition count outside [1,8]'})
            prior_actions=[]
            for c in conds:
                aid=None; name=c.get('status_input'); r=s.get('inputs',{}).get(name)
                vals=c.get('accepted_codes',[])
                if not vals or len(vals)>8 or vals!=sorted(set(vals)) or any(not isinstance(x,int) or x<0 or x>255 for x in vals): issues.append({'step':s['id'],'error':'invalid accepted guard codes','condition':c})
                if not r or r.get('kind')!='produced' or r.get('output_id')!='exit-status': issues.append({'step':s['id'],'error':'guard status not exact Produced exit-status','condition':c}); continue
                aid=r.get('action_id'); producer=owner.get(aid)
                if not producer or producer['index']>=idx[s['id']]: issues.append({'step':s['id'],'error':'guard does not name prior local action','condition':c}); continue
                prior_actions.append(aid); pstep=stepmap[producer['local']]
                if pstep['operation']['kind'] not in ('exec','controller') or out_kind(pstep,'exit-status')!='text': issues.append({'step':s['id'],'error':'guard producer has no Text command exit status','condition':c})
                elif not set(vals).issubset(exits(pstep)): issues.append({'step':s['id'],'error':'guard accepts code outside producer exit policy','condition':c})
                status_edges+=1
            if len(prior_actions)!=len(set(prior_actions)): issues.append({'step':s['id'],'error':'duplicate guard producer'})
        if op['kind']=='observe-phase':
            obs_count+=1; phase=op['phase']; ps=stepmap.get(phase)
            if not ps or idx.get(phase,10**9)>=idx[s['id']] or not ps['operation'].get('guard'): issues.append({'step':s['id'],'error':'ObservePhase does not depend on preceding guarded command'})
            state=op.get('state')
            if state:
                r=s.get('inputs',{}).get(state.get('skipped_input')); aid=r.get('action_id') if r else None
                local_prior=aid in owner and owner[aid]['index']<idx[s['id']]
                earlier_stage=any(aid in producers for producers in prior.values())
                if not r or r.get('kind')!='produced' or r.get('output_id')=='exit-status' or not (local_prior or earlier_stage) or aid==stage+'/'+phase:
                    issues.append({'step':s['id'],'error':'skipped alternative not a distinct earlier Produced state'})
                elif kind_for(r,owner,prior,stage_idx)!='tree': issues.append({'step':s['id'],'error':'skipped alternative root kind is not proven Tree'})
                if not normal_rel(state.get('skipped_path',''),True): issues.append({'step':s['id'],'error':'skipped alternative path is not canonical'})
                if ps and (state.get('output') not in ps.get('outputs',{}) or out_kind(ps,state.get('output'))!='tree'): issues.append({'step':s['id'],'error':'executed alternative output is absent/non-Tree'})
                phase_cases.append({'observer':s['id'],'phase':phase,'executed_output':state.get('output'),'skipped_input':state.get('skipped_input'),'skipped_path':state.get('skipped_path')})
            else: phase_cases.append({'observer':s['id'],'phase':phase,'executed_output':'phase outcome only','skipped_input':None})
        if op['kind']=='exec':
            pol=op.get('exit_policy',{}); optional=pol.get('kind') in ('one-of','normal') and 'exit-status' in s.get('outputs',{})
            if optional:
                optional_outputs+=sum(1 for oid in s['outputs'] if oid not in ('stdout','stderr','exit-status'))
        # Check dominating guards for every local guarded/optional ordinary output dependency.
        for name,r in s.get('inputs',{}).items():
            if r.get('kind')!='produced' or r.get('action_id') not in owner: continue
            prod=owner[r['action_id']]; pstep=stepmap[prod['local']]; output=r.get('output_id')
            ordinary=output not in ('stdout','stderr','exit-status')
            pguard=pstep['operation'].get('guard') if pstep['operation']['kind'] in ('exec','controller') else None
            optional=pstep['operation'].get('exit_policy',{}).get('kind') in ('one-of','normal') and 'exit-status' in pstep['outputs'] if pstep['operation']['kind']=='exec' else False
            if ordinary and (pguard or optional):
                consumer_guard=op.get('guard') if op['kind'] in ('exec','controller') else None
                if not consumer_guard: issues.append({'step':s['id'],'error':'guarded/optional ordinary output lacks consumer dominance','producer':r})
                else:
                    acts=[s['inputs'].get(c['status_input'],{}).get('action_id') for c in consumer_guard.get('conditions',[])]
                    if r['action_id'] not in acts: issues.append({'step':s['id'],'error':'guarded/optional output lacks actual producer predicate','producer':r})
                    elif pguard:
                        pos=acts.index(r['action_id']); prefix=[c.get('action_id') for c in pguard.get('conditions',[])]
                        if acts[:pos]!=prefix: issues.append({'step':s['id'],'error':'guarded output prior guard prefix does not dominate','producer':r})
    if cycles: issues.append({'error':'dependency graph cycle','examples':cycles[:4]})
    # Required recipe reachability, using the frozen consumer's all-input topology plus implicit ObservePhase edge.
    reachable=set(); todo=[]
    for r in recipe.get('roots',[]):
        if r.get('kind')=='produced' and r.get('action_id','').startswith(stage+'/'):
            local=r['action_id'][len(stage)+1:]
            if local in graph: todo.append(local)
    pred={s['id']:set() for s in recipe['steps']}
    for a,outs in graph.items():
        for b in outs: pred[b].add(a)
    while todo:
        n=todo.pop()
        if n in reachable: continue
        reachable.add(n); todo.extend(pred[n])
    omitted=set(graph)-reachable
    # Separately compute actual used-input dependency closure from roots.
    # used_refs includes all declared inputs for dynamic Execs whose invocation
    # bytes are absent, but only operation-consumed inputs for inspectable steps.
    actual_pred={s['id']:set() for s in recipe['steps']}
    for s in recipe['steps']:
        for encoded in actual[s['id']]:
            r=json.loads(encoded)
            aid=r.get('action_id','')
            if r.get('kind')=='produced' and aid in owner and aid.startswith(stage+'/'):
                actual_pred[s['id']].add(owner[aid]['local'])
        if s['operation']['kind']=='observe-phase':
            phase=s['operation']['phase']
            # ObservePhase's phase is an explicit consumer edge even when its
            # phase output is not a normal RecipeStep input.
            if phase in actual_pred and phase in stepmap:
                actual_pred[s['id']].add(phase)
    actual_reach=set(); todo=[]
    for r in recipe.get('roots',[]):
        if r.get('kind')=='produced' and r.get('action_id','').startswith(stage+'/'):
            l=r['action_id'][len(stage)+1:]
            if l in actual_pred: todo.append(l)
    while todo:
        n=todo.pop()
        if n in actual_reach:continue
        actual_reach.add(n);todo.extend(actual_pred[n])
    actual_omitted=sorted(set(graph)-actual_reach)
    return {'guarded_command_count':guard_count,'guard_status_edge_count':status_edges,'optional_failure_ordinary_output_count':optional_outputs,'observe_phase_count':obs_count,'phase_state_alternatives':phase_cases,'cycle_count':len(cycles),'validator_unreachable_step_count':len(omitted),'validator_unreachable_examples':sorted(omitted)[:10],'actual_dependency_edge_count':sum(len(v) for v in actual_pred.values()),'actual_dependency_reachable_step_count':len(actual_reach),'actual_dependency_unreachable_step_count':len(actual_omitted),'actual_dependency_unreachable_examples':actual_omitted[:100],'issues':issues}

def artifact_refs(value):
    if isinstance(value,dict):
        if value.get('kind') in ('produced','source','manifest-source'):
            yield value
        else:
            for child in value.values(): yield from artifact_refs(child)
    elif isinstance(value,list):
        for child in value: yield from artifact_refs(child)

def recipe_report(path,expected_stage,prior,stage_idx,plan_stage):
    data=load(path); issues=[]
    if data.get('stage_id')!=expected_stage: issues.append({'error':'recipe stage mismatch','actual':data.get('stage_id'),'expected':expected_stage})
    owner,steps=make_owner(data,stage_idx,prior,stage_idx,issues)
    # Validate every declared input against authority plan before lowering; exact role names retained.
    allowed_sources={x['role'] for x in plan_stage.get('inputs',[]) if x['authority']=='source'}
    reference_count=0
    for r in artifact_refs(data):
        reference_count+=1
        if r.get('kind') in ('source','manifest-source') and r.get('role') not in allowed_sources:
            issues.append({'error':'source role not declared to stage','ref':r})
    # Root/output refs and output ownership checks.
    if not data.get('roots'): issues.append({'error':'recipe has no roots'})
    if not data.get('exports'): issues.append({'error':'recipe has no exports'})
    for r in data.get('roots',[]):
        if r.get('kind')!='produced' or r.get('action_id','').split('/',1)[0]!=expected_stage: issues.append({'error':'root not local Produced'})
    for name,refs in data.get('semantic_roots',{}).items():
        if not refs: issues.append({'error':'empty semantic root','root':name})
        for r in refs:
            if r.get('kind')!='produced': issues.append({'error':'semantic root not Produced','root':name,'ref':r})
    words=recipe_words(data,owner,prior,stage_idx)
    source_bindings=[]; unresolved_source_bindings=[]
    program_records=SUBJECT.get('program_records',{})
    for ref in artifact_refs(data):
        if ref.get('kind') not in ('source','manifest-source'): continue
        matches=[x for x in SUBJECT['source_inputs'] if x.get('role')==ref.get('role')]
        kind=kind_for(ref,owner,prior,stage_idx)
        identity=ref.get('record_identity')
        record_match=bool(isinstance(identity,str) and (identity in program_records or identity in program_records.values())) if isinstance(program_records,dict) else False
        entry={'reference_kind':ref.get('kind'),'role':ref.get('role'),'member':ref.get('path',ref.get('member','')),'record_identity':identity,'source_input_matches':len(matches),'program_record_identity_matched':record_match,**source_kind_resolution(ref)}
        source_bindings.append(entry)
        if kind is None:
            missing={'where':f"{expected_stage}:source-reference",'part':ref,'reason':'no exact admitted binding/member ArtifactKind resolved','category':'unresolved-binding-kind','source_provenance':entry}
            unresolved_source_bindings.append(missing); words['unknown_binding_kinds'].append(missing)
    words['unresolved_binding_kind_count']+=len(unresolved_source_bindings)
    graph=graph_guard_report(data,owner,steps,prior,stage_idx)
    # Actual source recipe validator accepts all local Produced references only after prior producer presence/order.
    # Cross-stage identity is checked above against the full namespace and prior owners.
    declared_edges=sum(len(s.get('inputs',{})) for s in data['steps'])
    op_counts=collections.Counter(s['operation']['kind'] for s in data['steps'])
    semantic=data.get('semantic_roots',{})
    sem_count=sum(len(v) for v in semantic.values())
    summary={'path':str(path),'size_bytes':path.stat().st_size,'blake3':b3(path),'stage_id':data['stage_id'],'step_count':len(data['steps']),'operation_counts':dict(op_counts),'declared_input_edge_count':declared_edges,'root_count':len(data.get('roots',[])),'export_count':len(data.get('exports',{})),'semantic_root_key_count':len(semantic),'semantic_product_ref_count':sem_count,'semantic_root_key_examples':list(sorted(semantic))[:12],'word_count':words['word_count'],'word_part_count':words['word_part_count'],'word_part_kinds':words['part_kinds'],'static_invocation_json_count':words['invocation_count'],'controller_catalog_count':words['controller_count'],'dynamic_invocation_uninspectable_count':words['dynamic_invocation_uninspectable_count'],'word_issues':words['issues'][:30],'word_issue_count':len(words['issues']),'unknown_binding_kinds':words['unknown_binding_kinds'][:30],'unknown_binding_kind_count':words['unresolved_binding_kind_count'],'graph_guards':graph,'reference_issue_count':len(issues),'reference_issue_examples':issues[:30],'coverage':{'recipe_steps':{'total':len(data['steps']),'covered':len(data['steps'])},'static_words':{'total':words['word_count'],'covered':words['word_count']},'semantic_root_refs':{'total':sem_count,'covered':sem_count},'declared_input_refs':{'total':declared_edges,'covered':declared_edges}}}
    seed_counts=collections.Counter(); seed_reasons=collections.Counter(); seed_failures=[]
    seed_kinds=collections.defaultdict(collections.Counter)
    for i,s in enumerate(data['steps']):
        if s['operation']['kind']!='exec': continue
        report=inspect_output_seeds(expected_stage,s['id'],s.get('inputs',{}),s.get('outputs',{}),
            s['operation'].get('output_seeds',{}),owner,prior,i,s['operation'].get('guard'))
        seed_counts.update({k:report[k] for k in ('total','covered','invalid')})
        seed_reasons.update(report['failure_reason_counts'])
        for category in ('source_kind_counts','destination_kind_counts','producer_namespace_counts'):
            seed_kinds[category].update(report[category])
        if len(seed_failures)<100: seed_failures.extend(report['failure_examples'][:100-len(seed_failures)])
    summary['output_seeds']={'total':seed_counts['total'],'covered':seed_counts['covered'],
        'invalid':seed_counts['invalid'],'failure_reason_counts':dict(seed_reasons),
        'source_kind_counts':dict(seed_kinds['source_kind_counts']),
        'destination_kind_counts':dict(seed_kinds['destination_kind_counts']),
        'producer_namespace_counts':dict(seed_kinds['producer_namespace_counts']),
        'failure_examples':seed_failures,'consumer':'src/source_built_recipe.rs::validate_operation and lower_step'}
    summary['coverage']['output_seeds']={k:summary['output_seeds'][k] for k in ('total','covered','invalid')}
    summary['source_binding_references']={'total':len(source_bindings),'resolved':len(source_bindings)-len(unresolved_source_bindings),'unresolved':len(unresolved_source_bindings),'examples':unresolved_source_bindings[:20]}
    summary['coverage']['source_binding_kinds']={'total':len(source_bindings),'covered':len(source_bindings)-len(unresolved_source_bindings)}
    summary['coverage']['artifact_refs']={'total':reference_count,'covered':reference_count}
    summary['coverage']['guard_status_edges']={'total':graph['guard_status_edge_count'],'covered':graph['guard_status_edge_count']}
    summary['coverage']['observe_phase']={'total':graph['observe_phase_count'],'covered':graph['observe_phase_count']}
    summary['coverage']['dependency_edges']={'total':graph['actual_dependency_edge_count'],'covered':graph['actual_dependency_edge_count']}
    summary['invocation_input_count']=words['invocation_input_count']
    summary['coverage']['invocation_requests']={'total':words['invocation_input_count'],'covered':words['invocation_count']}
    return data,owner,summary,issues,words

def depends(stage,steps,actual,out,target,implicit=True):
    if out==target:return True
    def local(r):
        if r.get('kind')!='produced':return None
        aid=r.get('action_id','')
        # caller validated producer namespace before this identity reduction
        if not aid.startswith(stage+'/'):return None
        return aid[len(stage)+1:]
    first=local(out); todo=[first] if first in steps else []; seen=set()
    while todo:
        sid=todo.pop()
        if sid in seen:continue
        seen.add(sid); s=steps[sid]
        for ref in actual[sid]:
            r=json.loads(ref)
            if r==target:return True
            p=local(r)
            if p in steps:todo.append(p)
        op=s['operation']
        if implicit and op['kind']=='observe-phase':
            phase=op['phase']; state=op.get('state')
            if state and target.get('kind')=='produced' and target.get('action_id')==stage+'/'+phase and target.get('output_id')==state['output'] and phase in steps and state['output'] in steps[phase]['outputs']:return True
            if phase in steps:todo.append(phase)
    return False

def source_identity_observation(reference):
    resolution=source_kind_resolution(reference)
    result={'reference':reference,'binding':resolution,'digest_blake3':None}
    if resolution['resolved_artifact_kind'] is None: return result
    if reference['kind']=='source':
        if reference['role']=='stagex-source-bundle' and reference['path']=='':
            source_record_inventories(reference['role'])
            result.update(digest_blake3=SOURCE_MANIFEST_OBSERVATIONS[reference['role']]['manifest_blake3'],
                basis='canonical source manifest identity, not raw JSON bytes; require_source_digest special case')
        elif resolution['resolved_artifact_kind']=='text':
            result.update(digest_blake3=b3(Path(resolution['path'])),basis='exact physical source file BLAKE3')
    elif reference['kind']=='manifest-source':
        inventory=source_record_inventories(reference['role'])[reference['record_identity']]
        member=reference['member']
        if not member and inventory['record_kind'] in ('fixed-url','vcs-snapshot'):
            result.update(digest_blake3=inventory['content_blake3'],basis='original fetcher canonical record content; not hydrated tree hash')
        elif member:
            entries=inventory['file_entries'].get(member,[])
            if len(entries)==1 and entries[0]['file_type']=='regular' and 'chunk_index' not in entries[0]:
                result.update(digest_blake3=entries[0]['blake3'],basis='exact regular source-record leaf; embedded bytes independently checked by physical audit')
    return result

def installed_member_projections(recipe):
    stage=recipe['stage_id']; projections={}
    for step in recipe['steps']:
        op=step['operation']; args=op.get('arguments',[])
        if op['kind']!='exec' or len(args)!=4: continue
        if args[0] not in ({'parts':[{'kind':'literal-data','text':'--mantle-native-installed'}]},
                           {'parts':[{'kind':'literal','text':'--mantle-native-installed'}]}): continue
        parts=args[1].get('parts',[]); dest=args[3].get('parts',[])
        if len(parts)!=1 or parts[0].get('kind')!='input' or len(dest)!=1 or dest[0].get('kind')!='output': continue
        tree=step.get('inputs',{}).get(parts[0].get('name')); output_id=dest[0].get('name')
        if not tree or dest[0].get('path') or step['outputs'].get(output_id,{}).get('kind')!='executable': continue
        if op.get('exit_policy')!={'kind':'exact','code':0}: continue
        produced={'kind':'produced','action_id':stage+'/'+step['id'],'output_id':output_id}
        projections[key(produced)]={'tree':tree,'member':parts[0].get('path'),'projection_step':step['id'],
            'produced':produced,'contract':'mantle_native_installed requires an original isolated ELF in the catalog, then copies exactly that tree member; runtime identity remains unobserved'}
    return projections

def partition(recipe,nodes,stage):
    steps={s['id']:s for s in recipe['steps']}; actual={sid:used_refs(s) for sid,s in steps.items()}
    assigned=[]; bad_outputs=[]; empty_nodes=[]; no_exec=[]; invalid_refs=[]; hydrated_nodes=[]; unsupported_mechanisms=[]
    for nid,node in nodes.items():
        ids=node.get('step_ids',[])
        if not ids or not node.get('outputs'):empty_nodes.append(nid)
        if not any(steps.get(sid,{}).get('operation',{}).get('kind')=='exec' for sid in ids):
            no_exec.append(nid)
            valid_fetch=len(ids)==1 and ids[0] in steps
            fetch=steps[ids[0]] if valid_fetch else {}; op=fetch.get('operation',{})
            valid_fetch=valid_fetch and op.get('kind')=='materialize-fetcher'
            inventory=None
            if valid_fetch:
                inventory=source_record_inventories(op['role']).get(op['record_identity'])
                valid_fetch=bool(inventory and inventory['record_kind'] in ('fixed-url','vcs-snapshot'))
            if valid_fetch:
                expected='tree' if inventory['record_kind']=='vcs-snapshot' or inventory['metadata'].get('unpack')=='1' else 'text'
                valid_fetch=fetch['outputs'].get(op['output'],{}).get('kind')==expected
                valid_fetch=valid_fetch and any(r.get('kind')=='manifest-source' and r.get('role')==op['role'] and r.get('record_identity')==op['record_identity'] and r.get('member')=='' for r in fetch.get('inputs',{}).values())
            if valid_fetch: hydrated_nodes.append(nid)
            else: unsupported_mechanisms.append(nid)
        assigned.extend(ids)
        for out,ref in node.get('outputs',{}).items():
            # Validate the entire namespace before local id resolution.
            m=re.fullmatch(re.escape(stage)+r'/([A-Za-z0-9_.:-]+)',ref.get('action_id','')) if ref.get('kind')=='produced' else None
            if not m: bad_outputs.append({'node':nid,'output':out,'reason':'nonlocal/malformed producer namespace','ref':ref});continue
            p=steps.get(m.group(1))
            if p is None or ref.get('output_id') not in p.get('outputs',{}) or m.group(1) not in ids:bad_outputs.append({'node':nid,'output':out,'reason':'not exact owned step output','ref':ref})
    dup=len(assigned)-len(set(assigned)); unclassified=sorted(set(steps)-set(assigned)); unknown=sorted(set(assigned)-set(steps))
    unused=[]
    for sid,s in steps.items():
        actual_refs=actual[sid]; declared={key(r) for r in s.get('inputs',{}).values()}
        if declared-actual_refs: unused.append({'step':sid,'unused_count':len(declared-actual_refs),'examples':[json.loads(x) for x in sorted(declared-actual_refs)[:2]]})
    duplicates=sorted(x for x,c in collections.Counter(assigned).items() if c>1)
    return steps,actual,{'recipe_step_count':len(steps),'node_count':len(nodes),'assignment_count':len(assigned),'duplicate_assignment_count':dup,'duplicate_assignment_examples':duplicates[:12],'unclassified_step_count':len(unclassified),'unclassified_step_examples':unclassified[:12],'unknown_assigned_step_count':len(unknown),'unknown_assigned_step_examples':unknown[:12],'empty_node_count':len(empty_nodes),'empty_node_examples':empty_nodes[:12],'node_without_exec_count':len(no_exec),'node_without_exec_examples':no_exec[:12],'authenticated_fetcher_node_count':len(hydrated_nodes),'authenticated_fetcher_nodes':hydrated_nodes,'unsupported_non_exec_node_count':len(unsupported_mechanisms),'unsupported_non_exec_nodes':unsupported_mechanisms,'output_reference_count':sum(len(n.get('outputs',{})) for n in nodes.values()),'invalid_node_output_count':len(bad_outputs),'invalid_node_output_examples':bad_outputs[:12],'steps_with_unused_declared_inputs':len(unused),'unused_input_examples':unused[:12],'unused_input_classification':'Static operand-use diagnostic, not a source-recipe admission rule. Exact dependency declarations, success-ordering artifacts, generated-invocation context and command-path usage remain separate evidence; no minimal-authority claim.'}

def stagex_check(route,files,priorowners):
    lineage=load(files['lineage']); tr=load(files['stagex_transition_recipe']); ix=load(files['stagex_transition_index']); pr=load(files['publication_recipe']); pix=load(files['publication_index']); stage=tr['stage_id']; issues=[]
    steps,actual,part=partition(tr,ix['tools'],stage)
    lt={x['id']:x for x in lineage['transition_tools']}; ls={x['id']:x for x in lineage['source_artifacts']}; lp={x['id']:x for x in lineage['patches']}
    sets={'tool_ids_exact':set(ix['tools'])==set(lt),'source_ids_exact':set(ix['sources'])==set(ls),'patch_ids_exact':set(ix['patches'])==set(lp)}
    source_identity_checks={}
    for identity,item in {**ls,**lp}.items():
        reference=(ix['sources'] if identity in ls else ix['patches']).get(identity)
        observed=source_identity_observation(reference) if reference else {'digest_blake3':None}
        expected=item.get('digest',{})
        source_identity_checks[identity]={'reference':reference,'expected':expected,'observed':observed,
            'valid':expected.get('algorithm')=='blake3' and expected.get('hex_value')==observed['digest_blake3']}
    if not all(sets.values()):issues.append({'error':'StageX expansion ID sets mismatch','sets':sets})
    artifacts=dict(ix.get('sources',{})); artifacts.update(ix.get('patches',{})); artifacts['seed']={'kind':'source','role':'stagex-seed','path':''}; unique=True; outmap=[]
    for tid,t in lt.items():
        node=ix.get('tools',{}).get(tid)
        if not node: continue
        exp=set(t.get('output_artifact_ids',[])); got=set(node.get('outputs',{}))
        if exp!=got: issues.append({'error':'StageX original output inventory mismatch','tool':tid,'expected':sorted(exp)[:20],'actual':sorted(got)[:20]})
        for aid,ref in node.get('outputs',{}).items():
            if aid in artifacts: unique=False
            artifacts[aid]=ref; outmap.append((tid,aid,ref))
    gen_missing=[]; gen_checks=0
    for g in lineage['generated_artifacts']:
        gen_checks+=1; node=ix.get('tools',{}).get(g['producing_tool_id'])
        if not node or g['id'] not in node.get('outputs',{}):gen_missing.append(g['id'])
    input_edges=0; disposed=[]; missing_input=[]
    for tid,t in lt.items():
        node=ix.get('tools',{}).get(tid)
        if not node:continue
        actual_node={r for sid in node['step_ids'] for r in actual.get(sid,set())}
        for raw in list(t.get('source_artifact_ids',[]))+list(t.get('input_artifact_ids',[])):
            canonical='tinycc-0.9.27-stagex-patch' if raw=='tinycc-0.9.27-patch-source' else raw
            ref=artifacts.get(canonical)
            if ref is None:missing_input.append({'tool':tid,'input':canonical,'reason':'unmapped'});continue
            input_edges+=1
            reason=node.get('input_dispositions',{}).get(canonical,'')
            if reason and reason.strip():disposed.append({'tool':tid,'input':canonical,'reason':reason})
            elif key(ref) not in actual_node:missing_input.append({'tool':tid,'input':canonical,'reason':'not actual used input'})
    def refs(nodeid):
        if nodeid in artifacts:return [artifacts[nodeid]]
        n=ix.get('tools',{}).get(nodeid)
        return list(n.get('outputs',{}).values()) if n else None
    graphchecks=0; graphfails=[]; skipped=0
    for e in lineage['stage_graph']:
        if e['to_node_id'] in ix.get('sources',{}) or e['to_node_id'] in ix.get('patches',{}):
            skipped+=1
            target=source_identity_checks.get(e['to_node_id'],{})
            if refs(e['from_node_id']) is None or not target.get('valid'):
                graphfails.append({'edge':e,'reason':'source-admission edge has unknown source endpoint or mismatched authenticated identity'})
            continue
        a=refs(e['from_node_id']);b=refs(e['to_node_id'])
        if a is None or b is None:graphfails.append({'edge':e,'reason':'unmapped endpoint'});continue
        for inp in a:
            graphchecks+=1
            if not any(depends(stage,steps,actual,out,inp) for out in b):graphfails.append({'edge':e,'input':inp,'reason':'no actual dependency path'})
    provider=tr.get('exports',{}).get('stagex-transition'); pchecks=0; pfails=[]
    for p in lineage.get('provider_outputs',[]):
        pchecks+=1; ref=artifacts.get(p['producing_artifact_id'])
        if not ref or not provider or not depends(stage,steps,actual,provider,ref):pfails.append(p['producing_artifact_id'])
    pub_steps={s['id']:s for s in pr['steps']}; pub_commands=pix.get('validation_commands',{}); pub_issues=[]
    for k,v in pub_commands.items():
        s=pub_steps.get(v)
        if not s:pub_issues.append({'launch':k,'error':'missing selected command'});continue
        op=s['operation']
        if op.get('kind')!='exec':pub_issues.append({'launch':k,'error':'not Exec'});continue
        ex=op.get('executable_input');r=s.get('inputs',{}).get(ex)
        # Exact export or local positive build output; consumer enforces this rule.
        if k in ('positive-execution','tcc-execution') and not (r and r.get('kind')=='produced' and r.get('action_id')==pr['stage_id']+'/'+pub_commands.get('ld-positive' if k=='positive-execution' else 'tcc-positive','')): pub_issues.append({'launch':k,'error':'positive smoke not exact produced build output','ref':r})
    return {'stagex_lineage_counts':{k:len(lineage.get(k,[])) for k in ('source_artifacts','patches','transition_tools','generated_artifacts','stage_graph','provider_outputs')},'stagex_index_counts':{'sources':len(ix.get('sources',{})),'patches':len(ix.get('patches',{})),'tools':len(ix.get('tools',{}))},'id_sets':sets,'artifact_identity_unique':unique,'generated_output_checks':gen_checks,'generated_output_missing_count':len(gen_missing),'generated_output_missing_examples':gen_missing[:12],'partition':part,'lineage_input_edge_checks':input_edges,'authored_input_dispositions':disposed,'missing_actual_source_or_bootstrap_edges':missing_input,'stage_graph_edge_count':len(lineage['stage_graph']),'source_identity_checks':source_identity_checks,'source_identity_failure_count':sum(not x['valid'] for x in source_identity_checks.values()),'authenticated_source_admission_edge_count':skipped,'edges_to_source_or_patch_not_product_checked':0,'source_admission_contract':'Frozen validate_expansion treats edges into pre-existing authenticated source/patch nodes as admission, not seed production. All target identities are checked without inventing execution edges.','actual_stage_graph_edge_checks':graphchecks,'stage_graph_dependency_failures':graphfails[:20],'stage_graph_dependency_failure_count':len(graphfails),'provider_output_checks':pchecks,'provider_output_failures':pfails,'publication':{'binutils_admitted':pix.get('binutils_admitted'),'validation_command_count':len(pub_commands),'validation_commands':pub_commands,'step_count':len(pr['steps']),'unique_command_count':len(set(pub_commands.values())),'command_issues':pub_issues,'recipe_blake3':b3(files['publication_recipe']),'index_blake3':b3(files['publication_index'])}}

def route_parse(text):
    src=text.split('  sources = [',1)[1].split('  stages = [',1)[0]
    reg=text.split('  stages = [',1)[1].split('  final_outputs = [',1)[0]
    fin=text.split('  final_outputs = [',1)[1].split('\n  ],',1)[0]
    source_pairs=re.findall(r'\{\s*id\s*=\s*"([^"]+)"\s*,\s*kind\s*=\s*"[^"]+"\s*,.*?sha256_hex\s*=\s*"([0-9a-f]{64})"',src,re.S)
    matches=re.findall(r'\{\s*id\s*=\s*"([^"]+)"\s*,\s*kind\s*=\s*"([^"]+)"\s*,\s*source_ids\s*=\s*\[([^\]]*)\]\s*,\s*bootstrap_stage_id\s*=\s*"([^"]*)"\s*,\s*rust_version\s*=\s*"([^"]+)"\s*,\s*outputs\s*=\s*\[([^\]]*)\]',reg,re.S)
    stages=[{'id':a,'kind':b,'source_ids':re.findall(r'"([^"]+)"',c),'bootstrap_stage_id':d,'rust_version':e,'outputs':re.findall(r'"([^"]+)"',f)} for a,b,c,d,e,f in matches]
    finals=re.findall(r'role\s*=\s*"([^"]+)"',fin)
    if not source_pairs or not stages or not finals:raise ValueError('cannot statically parse frozen Rust typed route')
    polreg=text.split('  policy = {',1)[1].split('  },',1)[0]; policy={k:v=='true' for k,v in re.findall(r'([a-z_]+)\s*=\s*(true|false)',polreg)}
    return source_pairs,stages,finals,policy

def rust_check(recipe,index,route_path,native_recipe):
    text=route_path.read_text(); sources,route_stages,finals,policy=route_parse(text); stage=recipe['stage_id']; steps,actual,part=partition(recipe,index['stages'],stage)
    sourceids={s[0] for s in sources}; routeby={s['id']:s for s in route_stages}; native_exports=native_recipe.get('exports',{})
    achecks=[]; source_edges=0; source_missing=[]; boot_edges=0; boot_missing=[]; boot_exec=0; boot_exec_missing=[]; output_checks=0; bad_outputs=[]; cargo_exec=[]
    for sid,digest in sources:
        ref=index.get('archives',{}).get(sid,{}); valid=ref.get('kind')=='source' and ref.get('role')=='rust-source-archive-set' and normal_rel(ref.get('path',''),False)
        p=COHORT/'inputs/rust-source-archives'/ref.get('path','') if valid else None
        observed=sha(p) if p and p.is_file() else None
        achecks.append({'source_id':sid,'path':ref.get('path'),'reference_valid':valid,'route_sha256':digest,'observed_sha256':observed,'sha256_matches':observed==digest})
    for sid,rs in routeby.items():
        node=index.get('stages',{}).get(sid)
        if not node:continue
        if set(node.get('outputs',{}))!=set(rs['outputs']):bad_outputs.append({'stage':sid,'reason':'route output labels differ','expected':rs['outputs'],'actual':sorted(node.get('outputs',{}))})
        for src in rs['source_ids']:
            source_edges+=1;ref=index.get('archives',{}).get(src)
            if not ref or key(ref) not in {r for step in node.get('step_ids',[]) for r in actual.get(step,set())}: source_missing.append({'stage':sid,'source':src})
        pred=rs['bootstrap_stage_id']
        if pred:
            pn=index.get('stages',{}).get(pred)
            if not pn:boot_missing.append({'stage':sid,'predecessor':pred,'reason':'missing index node'});continue
            for oid,r in pn.get('outputs',{}).items():
                boot_edges+=1
                if key(r) not in {x for step in node.get('step_ids',[]) for x in actual.get(step,set())}:boot_missing.append({'stage':sid,'predecessor':pred,'output':oid,'ref':r})
            comp=pn.get('outputs',{}).get('rustc');boot_exec+=1
            if not any(steps.get(i,{}).get('operation',{}).get('kind')=='exec' and steps[i]['inputs'].get(steps[i]['operation'].get('executable_input'))==comp for i in node.get('step_ids',[])):boot_exec_missing.append({'stage':sid,'predecessor':pred})
        for role,r in node.get('outputs',{}).items():
            output_checks+=1
            aid=r.get('action_id','');m=re.fullmatch(re.escape(stage)+r'/([A-Za-z0-9_.:-]+)',aid) if r.get('kind')=='produced' else None
            local=m.group(1) if m else None
            if local not in node.get('step_ids',[]) or r.get('output_id') not in steps.get(local,{}).get('outputs',{}):bad_outputs.append({'stage':sid,'role':role,'ref':r})
    cargo=index.get('cargo_executable_reference') or recipe.get('exports',{}).get('cargo')
    for s in recipe['steps']:
        op=s['operation']
        if op['kind']=='exec' and cargo and s['inputs'].get(op.get('executable_input'))==cargo:cargo_exec.append(s['id'])
    native_roles=['native.cc1','native.cc1plus','native.as','native.ld','native.ar','native.nm','native.ranlib','native.objcopy','native.strip','native.readelf','native.loader','host.make','host.cmake','host.python','host.perl','host.busybox','native.provider','native.sysroot','linux.headers','native.libc','native.libgcc','native.libgcc-eh','native.libstdcxx']
    usage={role:any(key(r)==key(native_exports.get(role)) for refs in actual.values() for r in []) for role in []}
    usage={role:any(key(native_exports[role]) in refs for refs in actual.values()) for role in native_roles if role in native_exports}
    last=route_stages[-1]; final_stage=index['stages'].get(last['id'],{}).get('outputs',{})
    final_match={role:recipe.get('exports',{}).get(role)==final_stage.get(role) for role in finals}
    return {'route_path':str(route_path),'route_bytes':route_path.stat().st_size,'route_blake3':b3(route_path),'route_evaluated':False,'source_ids':sorted(sourceids),'stage_ids':sorted(routeby),'final_outputs':finals,'policy':policy,'archive_keyset_matches':set(index.get('archives',{}))==sourceids,'stage_keyset_matches':set(index.get('stages',{}))==set(routeby),'archive_checks':achecks,'source_edge_checks':source_edges,'missing_source_edges':source_missing,'bootstrap_output_edge_checks':boot_edges,'missing_bootstrap_output_edges':boot_missing[:30],'missing_bootstrap_output_edge_count':len(boot_missing),'bootstrap_compiler_exec_checks':boot_exec,'missing_bootstrap_compiler_execs':boot_exec_missing,'node_output_checks':output_checks,'invalid_output_refs':bad_outputs[:30],'invalid_output_ref_count':len(bad_outputs),'partition':part,'cargo_exec_count':len(cargo_exec),'cargo_exec_examples':cargo_exec[:20],'native_role_usage':usage,'native_role_missing':sorted(r for r,u in usage.items() if not u),'final_export_matches':final_match,'provider_export':recipe.get('exports',{}).get(stage),'rustc_export':recipe.get('exports',{}).get('rustc'),'toolchain_closure_export':recipe.get('exports',{}).get('toolchain-closure')}

def native_edge_check(recipe,index):
    stage=recipe['stage_id'];steps,actual,part=partition(recipe,index['derivations'],stage); nodes=index['derivations'];
    projections=installed_member_projections(recipe); refined_edges=[]
    edgecount=declared=used=0; missingdecl=[];missingused=[];outcheck=0;badout=[];sourcepaths=0;source_nodes=0;sourceactual=0
    for drv,node in nodes.items():
        node_actual={r for sid in node.get('step_ids',[]) for r in actual.get(sid,set())};node_declared={key(r) for sid in node.get('step_ids',[]) if sid in steps for r in steps[sid].get('inputs',{}).values()}
        if node.get('input_sources'):source_nodes+=1
        sourcepaths+=len(node.get('input_sources',[]))
        if any(json.loads(r).get('kind') in ('source','manifest-source') for r in node_actual):sourceactual+=1
        for out,r in node.get('outputs',{}).items():
            outcheck+=1; aid=r.get('action_id','');m=re.fullmatch(re.escape(stage)+r'/([A-Za-z0-9_.:-]+)',aid) if r.get('kind')=='produced' else None;local=m.group(1) if m else None
            if local not in node.get('step_ids',[]) or r.get('output_id') not in steps.get(local,{}).get('outputs',{}):badout.append({'drv':drv,'output':out,'ref':r})
        for dep,outs in node.get('input_derivations',{}).items():
            dn=nodes.get(dep)
            for out in outs:
                edgecount+=1
                if not dn or out not in dn.get('outputs',{}):missingdecl.append({'drv':drv,'dependency':dep,'output':out,'reason':'unmapped output'});continue
                r=dn['outputs'][out]; k=key(r)
                if k in node_declared:declared+=1
                else:missingdecl.append({'drv':drv,'dependency':dep,'output':out,'reason':'not declared'})
                if k in node_actual:used+=1
                else:
                    members=[p for member_key,p in projections.items() if p['tree']==r and member_key in node_actual]
                    if members:
                        used+=1; refined_edges.append({'drv':drv,'dependency':dep,'output':out,'selected_tree':r,'actual_consumed_members':members})
                    else:missingused.append({'drv':drv,'dependency':dep,'output':out,'reason':'neither selected Tree nor a proven exact installed member is actually consumed'})
    # Explicit Linux-header/make attribution. A declared tree edge is not command consumption.
    make_ref=recipe.get('exports',{}).get('host.make'); linux_headers_ref=recipe.get('exports',{}).get('linux.headers')
    linux_nodes=[(drv,node) for drv,node in nodes.items() if linux_headers_ref and linux_headers_ref in node.get('outputs',{}).values()]
    make_uses=[]; linux_make_edges=[]
    linux_make_members=[]
    for drv,node in linux_nodes:
        for sid in node.get('step_ids',[]):
            s=steps.get(sid)
            if not s or s['operation']['kind']!='exec': continue
            op=s['operation']; candidate_words=list(op.get('arguments',[]))+list(op.get('environment',{}).values())
            invname=op.get('invocation_input'); invref=s.get('inputs',{}).get(invname) if invname else None
            if invref and invref.get('kind')=='produced' and invref.get('action_id','').startswith(stage+'/'):
                producer=steps.get(invref['action_id'][len(stage)+1:])
                if producer and producer['operation']['kind']=='write-text':
                    try:
                        request=json.loads(producer['operation']['text'])
                        candidate_words.extend(request.get('arguments',[])); candidate_words.extend(request.get('environment',{}).values())
                    except (TypeError,ValueError): pass
            for word in candidate_words:
                for partword in word.get('parts',[]):
                    name=partword.get('name')
                    if partword.get('kind')=='input' and partword.get('path')=='bin/make' and make_ref and s.get('inputs',{}).get(name)==make_ref:
                        linux_make_members.append({'derivation':drv,'step_id':sid,'scope':'Input','input_name':name,'member':'bin/make'})
                    elif partword.get('kind')=='runtime' and partword.get('path')=='bin/make' and make_ref:
                        input_name=op.get('runtime_mounts',{}).get(partword.get('mount'))
                        if input_name and s.get('inputs',{}).get(input_name)==make_ref:
                            linux_make_members.append({'derivation':drv,'step_id':sid,'scope':'Runtime','mount':partword.get('mount'),'input_name':input_name,'member':'bin/make'})
    for drv,node in linux_nodes:
        for sid in node.get('step_ids',[]):
            s=steps.get(sid)
            if not s:continue
            input_names=[name for name,r in s.get('inputs',{}).items() if r==make_ref] if make_ref else []
            actual_make=bool(make_ref and key(make_ref) in actual.get(sid,set()))
            if input_names or actual_make:
                op=s['operation'];exe=op.get('executable_input') if op['kind']=='exec' else None; exref=s.get('inputs',{}).get(exe) if exe else None
                entry={'linux_headers_derivation':drv,'step_id':sid,'operation_kind':op['kind'],'make_input_names':input_names,'make_ref_is_actual_used_input':actual_make,'executable_input':exe,'executable_ref':exref,'static_arguments':op.get('arguments',[]) if op['kind']=='exec' else None,'invocation_input':op.get('invocation_input') if op['kind']=='exec' else None}
                linux_make_edges.append(entry)
    for s in recipe['steps']:
        op=s['operation']
        if op['kind']=='exec' and make_ref and s['inputs'].get(op.get('executable_input'))==make_ref:
            make_uses.append({'step_id':s['id'],'derivation_owners':[d for d,n in nodes.items() if s['id'] in n.get('step_ids',[])],'arguments':op.get('arguments',[]),'invocation_input':op.get('invocation_input'),'runtime_mounts':op.get('runtime_mounts',{})})
    projection=projections.get(key(make_ref)) if make_ref else None
    if projection and projection['member']=='bin/make':
        for use in make_uses:
            for drv,node in linux_nodes:
                if use['step_id'] in node.get('step_ids',[]):
                    linux_make_members.append({'derivation':drv,'step_id':use['step_id'],'scope':'exact Produced Executable refinement',**projection})
    report={'index_schema':index.get('schema'),'roots':index.get('roots',[]),'root_count':len(index.get('roots',[])),'derivation_count':len(nodes),'partition':part,'derivation_output_owner_checks':outcheck,'invalid_output_owner_count':len(badout),'invalid_output_owner_examples':badout[:8],'input_derivation_output_edge_count':edgecount,'edges_present_as_declared_inputs':declared,'edges_present_as_actual_operation_inputs':used,'missing_declared_edge_count':len(missingdecl),'missing_declared_examples':missingdecl[:10],'missing_actual_edge_count':len(missingused),'missing_actual_examples':missingused[:15],'source_anchor_path_count':sourcepaths,'nodes_with_source_anchors':source_nodes,'nodes_with_actual_typed_source_inputs':sourceactual,'linux_headers_derivations':len(linux_nodes),'host_make_export':make_ref,'linux_header_make_edge_details':linux_make_edges[:50],'linux_header_make_edge_detail_count':len(linux_make_edges),'actual_host_make_executable_steps':make_uses[:50],'actual_host_make_executable_step_count':len(make_uses),'linux_header_consumed_bin_make_member_count':len(linux_make_members),'linux_header_consumed_bin_make_member_examples':linux_make_members[:30],'linux_header_consumed_bin_make_member_complete':bool(linux_nodes and linux_make_members)}
    report['exact_installed_member_dependency_edges']=refined_edges
    report['linux_headers_export_ref']=linux_headers_ref
    report['linux_headers_export_owner_derivations']=[drv for drv,_ in linux_nodes]
    return report

def mantle_root_report(stage,recipe,prior):
    exports=recipe.get('exports',{}); sem=recipe.get('semantic_roots',{})
    steps={step['id']:step for step in recipe['steps']}
    binary=exports.get(stage); smoke=exports.get('mantle-smoke')
    unit_keys=sorted(name for name in sem if name.startswith('rust-unit:') and name.endswith(':path+native#mantle@0.1.0:mantle:bin:build'))
    unit=sem[unit_keys[0]] if len(unit_keys)==1 else []
    help_steps=[]
    for step in recipe['steps']:
        op=step['operation']
        if op['kind']=='exec' and step.get('inputs',{}).get(op.get('executable_input'))==binary and any(p.get('kind')=='literal' and p.get('text')=='--help' for word in op.get('arguments',[]) for p in word.get('parts',[])):
            help_steps.append(step['id'])
    expected_roots={key(ref) for refs in sem.values() for ref in refs}
    if binary: expected_roots.add(key(binary))
    if smoke: expected_roots.add(key(smoke))
    actual_roots={key(ref) for ref in recipe.get('roots',[])}
    smoke_action=smoke.get('action_id','') if isinstance(smoke,dict) else ''
    smoke_local=smoke_action[len(stage)+1:] if isinstance(smoke_action,str) and smoke_action.startswith(stage+'/') else None
    smoke_step=steps.get(smoke_local) if smoke_local else None
    report={
        'export_names':sorted(exports),'binary_export':binary,'smoke_export':smoke,
        'mantle_unit_key_candidates':unit_keys,'mantle_unit_key_unique':len(unit_keys)==1,'mantle_unit_root':unit,
        'build_script_roots':{name:refs for name,refs in sem.items() if name.startswith('build-script:')},
        'semantic_key_count':len(sem),'semantic_product_count':sum(map(len,sem.values())),
        'semantic_key_prefix_counts':dict(collections.Counter(name.split(':',1)[0] for name in sem)),
        'semantic_root_empty_count':sum(not refs for refs in sem.values()),
        'semantic_refs_nonproduced_count':sum(ref.get('kind')!='produced' for refs in sem.values() for ref in refs),
        'binary_is_exact_mantle_semantic_product':bool(binary and binary.get('kind')=='produced' and len(unit_keys)==1 and binary in unit),
        'binary_output_kind':kind_for(binary,prior[stage],prior,STAGES.index(stage)) if binary else None,
        'help_consumer_step_ids':help_steps,'help_consumer_count':len(help_steps),
        'smoke_export_is_produced':bool(smoke and smoke.get('kind')=='produced'),
        'smoke_export_actual_producer_valid':bool(smoke_step and smoke_step['id'] in help_steps and smoke.get('output_id')=='stdout' and out_kind(smoke_step,'stdout')=='text'),
        'recipe_roots_exact_semantic_binary_smoke_union':actual_roots==expected_roots and len(recipe.get('roots',[]))==len(actual_roots),
    }
    report['root_and_consumer_failure_count']=int(not report['binary_is_exact_mantle_semantic_product'] or report['binary_output_kind']!='executable' or report['help_consumer_count']!=1 or not report['smoke_export_actual_producer_valid'] or not report['recipe_roots_exact_semantic_binary_smoke_union'])
    return report

def main():
    SCRATCH.mkdir(parents=True,exist_ok=True)
    planfile=COHORT/'source-program-preparation-input-plan.json'; plan=load(planfile)
    route_path,route=selected_route()
    idrec={'cold_route':route_path,'plan':planfile}
    # Resolve selectors from current route; never assume e9 member names.
    sxid=route['stagex_transition']['record_identity']; sxpubid=route['stagex_publication']['record_identity']
    if sxid!=sxpubid: raise ValueError('StageX selector record identities differ')
    sxroot=COHORT/'program-sources'/sxid
    sxtr=sxroot/route['stagex_transition']['member']; sxpr=sxroot/route['stagex_publication']['member']
    nativeid=route['native']['record_identity']; nroot=COHORT/'program-sources'/nativeid
    nr=nroot/route['native']['recipe_member']; ni=nroot/route['native']['index_member']
    rustref=route['rust_provider']; rustroot=COHORT/'inputs/rust-source-archives'; rr=rustroot/rustref['path']; ri=rustroot/(rustref['path']+'.expansion.json')
    rustix=load(ri)
    rust_route_path=rustroot/rustix['route_path']
    lineage_dir=COHORT/'inputs/stagex-lineage'; lineage_candidates=sorted(lineage_dir.glob('*.json'))
    if len(lineage_candidates)!=1: raise ValueError(f'expected one current lineage JSON under {lineage_dir}, got {len(lineage_candidates)}')
    lineage=lineage_candidates[0]
    mantlepaths=[program_member_path(member) for member in MANTLE_MEMBERS]
    files={'cold_route':route_path,'lineage':lineage,'stagex_transition_recipe':sxtr,'stagex_transition_index':Path(str(sxtr)+'.expansion.json'),'publication_recipe':sxpr,'publication_index':Path(str(sxpr)+'.expansion.json'),'native_recipe':nr,'native_index':ni,'rust_recipe':rr,'rust_index':ri,'rust_route':rust_route_path,'mantle_stage1':mantlepaths[0],'mantle_stage2':mantlepaths[1]}
    # Preserve source selectors and identities for every selected file.
    for k,p in files.items():idrec[k]=p
    hashes={k:{'path':str(p),'size_bytes':p.stat().st_size,'blake3':b3(p)} for k,p in idrec.items()}
    def claim_path(value):
        if not isinstance(value,str) or not value: return None
        p=Path(value)
        candidates=[p] if p.is_absolute() else [COHORT/p,SOURCE/p,Path.cwd()/p]
        return next((candidate for candidate in candidates if candidate.exists()),candidates[0])
    plan_claim=claim_path(SUBJECT.get('input_plan'))
    report_claim=claim_path(SUBJECT.get('preparation_report'))
    runner_claim=claim_path(SUBJECT.get('runner'))
    source_claim=Path(SUBJECT.get('frozen_source','')).resolve()
    cohort_claim=Path(SUBJECT.get('cohort','')).resolve()
    subject_file_observations={'input_plan':{'claim':SUBJECT.get('input_plan'),'path':str(plan_claim) if plan_claim else None,'blake3':b3(plan_claim) if plan_claim and plan_claim.is_file() else None},'preparation_report':{'claim':SUBJECT.get('preparation_report'),'path':str(report_claim) if report_claim else None,'blake3':b3(report_claim) if report_claim and report_claim.is_file() else None},'runner':{'claim':SUBJECT.get('runner'),'path':str(runner_claim) if runner_claim else None,'blake3':b3(runner_claim) if runner_claim and runner_claim.is_file() else None}}
    plan_digest_input=dict(plan); plan_digest_input["plan_digest_blake3"]=""
    computed_plan_digest=subprocess.check_output([B3,"--derive-key","mantle-source-built-fixed-point-plan-v2","--no-names"],
        input=json.dumps(plan_digest_input,ensure_ascii=False,separators=(",",":")).encode()).decode().strip()
    subject_checks={'input_plan_blake3_matches_observed':SUBJECT.get('input_plan_blake3')==plan['plan_digest_blake3']==computed_plan_digest,'input_plan_path_matches_observed':plan_claim is not None and plan_claim.resolve()==planfile.resolve(),'expected_stagex_lineage_blake3_matches_observed':SUBJECT.get('expected_stagex_lineage_blake3')==hashes['lineage']['blake3'],'preparation_report_raw_blake3_matches_observed':bool(report_claim and report_claim.is_file() and SUBJECT.get('preparation_report_raw_blake3')==b3(report_claim)),'runner_blake3_matches_observed':bool(runner_claim and runner_claim.is_file() and SUBJECT.get('runner_blake3')==b3(runner_claim)),'frozen_source_path_matches_observed':source_claim==SOURCE.resolve(),'cohort_path_matches_observed':cohort_claim==COHORT.resolve()}
    subject_checks['source_inputs_match_observed_plan']=SUBJECT.get('source_inputs')==plan.get('source_inputs')
    expected_revision=os.environ.get('EXPECTED_FROZEN_PRODUCT_REVISION','')
    if not re.fullmatch('[0-9a-f]{40}',expected_revision):
        raise ValueError('expected product revision must be explicitly sealed as a 40-hex commit')
    subject_checks['sealed_product_revision_matches']=SUBJECT['frozen_product_revision']==expected_revision
    subject_checks['program_manifest_matches_admitted_bytes']=SUBJECT['prepared_native_source_manifest_raw_blake3']==SOURCE_MANIFEST_OBSERVATIONS[MANTLE_ROLE]['blake3']
    subject_checks['program_record_matches_admitted_identity']=source_record_inventories(MANTLE_ROLE)[MANTLE_RECORD]['content_blake3']==SUBJECT['program_records'][MANTLE_RECORD]
    if not all(subject_checks[name] for name in ('sealed_product_revision_matches',
            'program_manifest_matches_admitted_bytes','program_record_matches_admitted_identity')):
        raise ValueError('subject revision or program source authority differs from sealed expectation')
    subject_source_inputs=[{**x,'resolved_kind':'tree' if x['kind']=='directory' else 'text','resolution_basis':'frozen lower_step maps hydrated directories to Tree and regular files to Text; exact members resolved separately'} for x in SUBJECT['source_inputs']]
    consumer_hashes={p:{'path':p,'blake3':b3(SOURCE/p),'size_bytes':(SOURCE/p).stat().st_size} for p in CONSUMERS}
    prior={}; recipe_summaries={}
    # Recipe stage order comes from the current input plan, cross-checked against fixed cold-route protocol.
    if [s['stage_id'] for s in plan['stages']]!=STAGES: raise ValueError('plan stage order differs from frozen six-stage consumer order')
    stage_data={}
    for i,stage in enumerate(STAGES):
        if stage=='stagex-transition': path=sxtr
        elif stage=='stagex-provider-publication':path=sxpr
        elif stage=='full-source-native-provider':path=nr
        elif stage=='full-source-rust-provider':path=rr
        else:path=mantlepaths[0 if stage=='mantle-stage1' else 1]
        data,owner,summary,issues,words=recipe_report(path,stage,prior,i,plan['stages'][i])
        recipe_summaries[stage]=summary;stage_data[stage]=data
        prior[stage]=owner
    (SCRATCH/'source-binding-kind-inventory.json').write_text(json.dumps({'subject_json_blake3':b3(SUBJECT_PATH),'consumer_blake3':b3(SOURCE/'src/source_built_recipe.rs'),'source_bundle_consumer_blake3':b3(SOURCE/'src/source_bundle.rs'),'manifests':SOURCE_MANIFEST_OBSERVATIONS,'resolved_references':[{**json.loads(encoded),**resolution} for encoded,resolution in SOURCE_KIND_CACHE.items()],'non_claim':'Static source binding classes only; separate complete physical audit binds embedded payload bytes. No product execution or launch acceptance.'},indent=2,sort_keys=True)+'\n')
    del issues,words
    sx=stagex_check(route,files,prior)
    native_idx=load(ni); native_report=native_edge_check(stage_data['full-source-native-provider'],native_idx)
    rust_report=rust_check(stage_data['full-source-rust-provider'],rustix,rust_route_path,stage_data['full-source-native-provider'])
    # Validate the uniquely qualified Rust-unit product and its actual binary/smoke consumers together.
    mantle_sem={stage:mantle_root_report(stage,stage_data[stage],prior) for stage in ('mantle-stage1','mantle-stage2')}
    # Full current consumer/hash bindings, graph/typed observations and bounded exclusions.
    stage_coverage={stage:summary['coverage'] for stage,summary in recipe_summaries.items()}
    unresolved=sum(x['unknown_binding_kind_count'] for x in recipe_summaries.values())
    runtime_invocations=sum(x['dynamic_invocation_uninspectable_count'] for x in recipe_summaries.values())
    raw_seed_total=sum(x['output_seeds']['total'] for x in recipe_summaries.values())
    raw_seed_covered=sum(x['output_seeds']['covered'] for x in recipe_summaries.values())
    def partition_defects(p): return sum(p.get(k,0) for k in ('duplicate_assignment_count','unclassified_step_count','unknown_assigned_step_count','empty_node_count','unsupported_non_exec_node_count','invalid_node_output_count'))
    finding_counts={
        'recipe_reference_word_and_guard_errors':sum(x['reference_issue_count']+x['word_issue_count']+len(x['graph_guards']['issues']) for x in recipe_summaries.values()),
        'stagex_failures':sum(not ok for ok in sx['id_sets'].values())+int(not sx['artifact_identity_unique'])+sx['generated_output_missing_count']+len(sx['missing_actual_source_or_bootstrap_edges'])+sx['stage_graph_dependency_failure_count']+sx['source_identity_failure_count']+len(sx['provider_output_failures'])+len(sx['publication']['command_issues'])+partition_defects(sx['partition'])+sx['edges_to_source_or_patch_not_product_checked'],
        'native_failures':partition_defects(native_report['partition'])+native_report['invalid_output_owner_count']+native_report['missing_declared_edge_count']+native_report['missing_actual_edge_count']+int(not native_report['linux_header_consumed_bin_make_member_complete']),
        'rust_failures':partition_defects(rust_report['partition'])+rust_report['invalid_output_ref_count']+len(rust_report['missing_source_edges'])+rust_report['missing_bootstrap_output_edge_count']+len(rust_report['missing_bootstrap_compiler_execs'])+sum(not x['sha256_matches'] for x in rust_report['archive_checks'])+len(rust_report['native_role_missing'])+sum(not x for x in rust_report['final_export_matches'].values())+int(not rust_report['archive_keyset_matches'])+int(not rust_report['stage_keyset_matches'])+rust_report['cargo_exec_count'],
        'mantle_root_and_consumer_failures':sum(report['root_and_consumer_failure_count'] for report in mantle_sem.values()),
        'subject_claim_mismatches':sum(not ok for ok in subject_checks.values()),
        'raw_output_seed_failures':raw_seed_total-raw_seed_covered,
    }
    failures=sum(finding_counts.values())
    limitations=[]
    if runtime_invocations: limitations.append({'criterion':'runtime invocation decode/admission','unobserved_runtime_invocations':runtime_invocations,'reason':'payload bytes are produced only during protected controller execution'})
    if unresolved: limitations.append({'criterion':'prepared source binding kinds','unresolved_binding_kind_count':unresolved,'reason':'no actual admitted binding kind available; never inferred from identity or digest shape'})
    limitations.extend([{'criterion':'protected execution and runtime observations','reason':'static inspection does not execute products or observe protected launch results'},{'criterion':'Cargo-free semantic topology equality','reason':'not derivable solely from recipe/index; requires the actual plan-derived topology consumer'},{'criterion':'source/profile complete byte identity','reason':'the inspector hashes selected admitted inputs and route/plan; full-source archive identity is a separate physical qualification boundary'},{'criterion':'fixed-point equality, reproducibility, signed-row acceptance, release eligibility','reason':'not established by an inert graph report'}])
    interpreter=Path(sys.executable)
    result={'scope':'Inert static inspection only; never executes source programs, recipes, controllers, providers, NCL, builds, tests, or launch commands.','subject_binding':{'claims':SUBJECT,'subject_json_observed':{'path':str(SUBJECT_PATH),'size_bytes':SUBJECT_PATH.stat().st_size,'blake3':b3(SUBJECT_PATH)},'observed_claim_checks':subject_checks,'source_input_provenance':subject_source_inputs,'input_plan_observed':hashes['plan'],'route_observed':hashes['cold_route'],'selected_inputs_observed':hashes,'consumer_sources_observed':consumer_hashes},'selectors':route,'six_recipe_reports':recipe_summaries,'stagex_lineage_expansion_publication':sx,'native_expansion_edges_and_make_uses':native_report,'rust_route_expansion_and_canonical_handoffs':rust_report,'mantle_stage_semantic_root_and_smoke_inventory':mantle_sem,'consumer_blake3':consumer_hashes,'selected_input_blake3':hashes,'execution_identity':{'interpreter_path':str(interpreter),'interpreter_blake3':b3(interpreter),'hash_tool_path':B3,'hash_tool_blake3':b3(Path(B3)),'command_argv':sys.argv},'inspector_blake3':b3(Path(__file__)),'coverage':{'stages':stage_coverage,'recipe_count':{'total':6,'covered':len(recipe_summaries)},'recipe_steps':{'total':sum(x['step_count'] for x in recipe_summaries.values()),'covered':sum(x['coverage']['recipe_steps']['covered'] for x in recipe_summaries.values())},'static_words':{'total':sum(x['word_count'] for x in recipe_summaries.values()),'covered':sum(x['coverage']['static_words']['covered'] for x in recipe_summaries.values())},'typed_word_issues':sum(x['word_issue_count'] for x in recipe_summaries.values()),'source_producer_reference_issues':sum(x['reference_issue_count'] for x in recipe_summaries.values()),'runtime_symbolic_invocation_count':runtime_invocations,'unresolved_binding_kind_count':unresolved,'stagex_edge_failure_count':sx.get('stage_graph_dependency_failure_count',0),'native_edge_failure_count':native_report['missing_actual_edge_count'],'rust_source_edge_failure_count':len(rust_report['missing_source_edges']),'static_failures':failures},'limitations':limitations,'bounded_verdict':'HOLD' if failures or limitations else 'STATIC-ONLY; protected execution and release eligibility remain unproven'}
    result['subject_binding']['file_observations']=subject_file_observations
    coverage=result['coverage']; coverage['finding_counts']=finding_counts
    coverage['raw_output_seeds']={'total':raw_seed_total,'covered':raw_seed_covered,'invalid':raw_seed_total-raw_seed_covered,
        'failure_examples':[row for x in recipe_summaries.values() for row in x['output_seeds']['failure_examples']][:100]}
    coverage['stagex_lineage_edges']={'total':sx['stage_graph_edge_count'],'covered':sx['actual_stage_graph_edge_checks']+sx['authenticated_source_admission_edge_count'],'authenticated_source_admission':sx['authenticated_source_admission_edge_count'],'execution_dependency_checks':sx['actual_stage_graph_edge_checks'],'uncovered_source_or_patch_edges':sx['edges_to_source_or_patch_not_product_checked'],'failed':sx['stage_graph_dependency_failure_count']+sx['source_identity_failure_count']}
    coverage['native_derivation_edges']={'total':native_report['input_derivation_output_edge_count'],'declared':native_report['edges_present_as_declared_inputs'],'used':native_report['edges_present_as_actual_operation_inputs'],'missing_declared':native_report['missing_declared_edge_count'],'missing_actual':native_report['missing_actual_edge_count']}
    coverage['rust_source_edges']={'total':rust_report['source_edge_checks'],'covered':rust_report['source_edge_checks']-len(rust_report['missing_source_edges']),'missing':len(rust_report['missing_source_edges'])}
    coverage['rust_bootstrap_outputs']={'total':rust_report['bootstrap_output_edge_checks'],'covered':rust_report['bootstrap_output_edge_checks']-rust_report['missing_bootstrap_output_edge_count'],'missing':rust_report['missing_bootstrap_output_edge_count']}
    coverage['mantle_semantic_roots']={stage:{'total':x['semantic_product_count'],'covered':x['semantic_product_count'],'binary_root_exact':x['binary_is_exact_mantle_semantic_product'],'help_consumers':x['help_consumer_count'],'smoke_export_present':x['smoke_export_is_produced']} for stage,x in mantle_sem.items()}
    out=SCRATCH/'graph-audit.json';out.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps(result,indent=2,sort_keys=True))
if __name__=='__main__':main()
