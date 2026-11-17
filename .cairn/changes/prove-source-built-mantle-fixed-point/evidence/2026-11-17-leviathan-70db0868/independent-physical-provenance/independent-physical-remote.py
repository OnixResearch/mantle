#!/usr/bin/env python3
"""Inert independent full-cohort physical inspection, never candidate execution."""
import collections, json, os, re, stat, subprocess, sys, tarfile
from pathlib import Path, PurePosixPath
R=Path('/home/brittonr/sbfp-20261113-berdKH')
C=R/'prepared-cohort-70db0868-primary-sources'
F=R/'runner-source-70db0868-primary-sources'
B='/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
G='/run/current-system/sw/bin/git'
N='/run/current-system/sw/bin/nix'
REV='70db08685c0c5fa5c326bfe32db78576d7e2e182'
PROGS={'fixed-point-native-source-program','fixed-point-rust-source-program','source-built-mantle-programs','source-built-stagex-programs'}
ROLE={'bootstrap-source':'native-source-bundle','mantle-source':'mantle-source','proof-input':'rust-source-archive-set','stagex-lineage':'stagex-lineage','stagex-seed':'stagex-seed','stagex-source-bundle':'stagex-source-bundle','vendored-cargo-inputs':'vendor-inputs'}
DIR={'mantle-source','proof-input','vendored-cargo-inputs'}
ROOTS={'mantle-source':C/'inputs/mantle-source','proof-input':C/'inputs/rust-source-archives','vendored-cargo-inputs':C/'inputs/vendor-deps'}
CHUNK=67108864
OUT={'revision':REV,'commands':[],'issues':[]}
def need(ok,why):
    if not ok: raise RuntimeError(why)
def run(cmd,input=None):
    p=subprocess.run(cmd,input=input,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=3600)
    need(p.returncode==0,f'command {cmd[:4]} rc={p.returncode}: {p.stderr[-800:]}')
    return p.stdout
def b3(data): return run([B,'--no-names'],data).decode().strip()
def filehash(p): return run([B,'--no-names',str(p)]).decode().strip()
def fingerprint(obj):
    p=subprocess.Popen([B,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    for s in json.JSONEncoder(ensure_ascii=False,sort_keys=True,separators=(',',':')).iterencode(obj): p.stdin.write(s.encode())
    out,err=p.communicate();need(p.returncode==0,err[-400:]);return out.decode().strip()
def canonical_content(rec):
    p=subprocess.Popen([B,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    md=rec.get('metadata',{})
    if md:
        need(bool(rec['files']),'unexpected metadata-only record')
        p.stdin.write(b'kind\0');p.stdin.write(json.dumps(rec['kind'],separators=(',',':')).encode()+b'\nmetadata\0')
        p.stdin.write(json.dumps(md,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()+b'\nfiles\0')
    for e in rec['files']:
        p.stdin.write(json.dumps(e,ensure_ascii=False,separators=(',',':')).encode()+b'\n')
    out,err=p.communicate();need(p.returncode==0,err[-400:]);return out.decode().strip()
def entries(path):
    # One record per top-level pretty JSON object; prevents whole-profile allocation.
    start=False;within=False;buf=bytearray();count=0
    with path.open('rb',buffering=1024*1024) as stream:
        for line in stream:
            if not start:
                if line.strip()==b'"records": [':start=True
                continue
            if not within:
                if line in (b'    {\n',b'    {\r\n'):within=True;buf=bytearray(line)
                elif line.strip().startswith(b']'): break
                continue
            buf.extend(line)
            need(len(buf)<=20*1024**3,'profile record exceeds maximum')
            if line in (b'    },\n',b'    }\n',b'    },\r\n',b'    }\r\n'):
                row=json.loads(buf.rstrip().removesuffix(b','));count+=1;yield row
                within=False;buf=bytearray()
    need(start and not within and count>0,f'profile incomplete {path}')
def rel(v):
    need(isinstance(v,str) and v and '\\' not in v and '\n' not in v and '\0' not in v,'unsafe source name')
    p=PurePosixPath(v);need(not p.is_absolute() and str(p)==v and '..' not in p.parts,'unsafe relative path');return v
def walk(root):
    result={};stack=[(root,'')]
    need(root.is_dir() and not root.is_symlink(),f'non-directory {root}')
    while stack:
        p,prefix=stack.pop()
        with os.scandir(p) as it: children=list(it)
        for e in children:
            name=rel(prefix+'/'+e.name if prefix else e.name);s=e.stat(follow_symlinks=False)
            need(stat.S_ISDIR(s.st_mode) or stat.S_ISREG(s.st_mode) or stat.S_ISLNK(s.st_mode),f'special node {e.path}')
            result[name]=(Path(e.path),s)
            if stat.S_ISDIR(s.st_mode):stack.append((Path(e.path),name))
    return result
def hex_bytes(e):
    text=e.get('content_hex');size=e['size']
    need(isinstance(text,str) and len(text)==size*2 and text==text.lower(),'invalid embedded payload')
    for off in range(0,len(text),2*1024*1024):yield bytes.fromhex(text[off:off+2*1024*1024])
def role_tree(rec,plan):
    # Recompute tree from every embedded preauthor leaf, including all required parents.
    groups=collections.defaultdict(list)
    for e in rec['files']:
        need(e['file_type']=='regular','unexpected symlink in source directory')
        groups[rel(e['path'])].append(e)
    parents=set()
    for p in groups:
        for parent in PurePosixPath(p).parents:
            if str(parent)!='.':parents.add(str(parent))
    need(not parents.intersection(groups),'parent/leaf collision')
    paths=sorted(groups.keys()|parents,key=lambda x:x.encode())
    physical=walk(ROOTS[rec['metadata']['bootstrap_profile_class']])
    if rec['metadata']['bootstrap_profile_class']!='proof-input':need(set(paths)==set(physical),'original dir differs from hydrated inventory')
    else:need(set(groups).issubset(physical),'original rust archives absent after authoring')
    p=subprocess.Popen([B,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    check=[];payload=0;chunked=0;leaf_hashes=[]
    for name in paths:
        raw=name.encode();p.stdin.write(len(raw).to_bytes(8,'little')+raw)
        if name in parents:
            p.stdin.write((0o040755).to_bytes(4,'little')+b'dir\0')
            if name in physical:need(stat.S_ISDIR(physical[name][1].st_mode),'bad directory')
            continue
        cells=sorted(groups[name],key=lambda e:e.get('chunk_index',-1));size=sum(e['size'] for e in cells)
        need(len(cells)==1 and cells[0].get('chunk_index') is None or
             len(cells)>1 and [e.get('chunk_index') for e in cells]==list(range(len(cells))) and
             all(e.get('chunk_count')==len(cells) for e in cells) and
             all(e['size']==CHUNK for e in cells[:-1]),'noncanonical chunks')
        mode=0o100755 if cells[0]['executable'] else 0o100644
        need(all(e['executable']==cells[0]['executable'] for e in cells),'mixed mode')
        p.stdin.write(mode.to_bytes(4,'little')+b'file\0'+size.to_bytes(8,'little'))
        file,st=physical[name];need(stat.S_ISREG(st.st_mode) and stat.S_IMODE(st.st_mode)==stat.S_IMODE(mode) and st.st_size==size,'preauthor physical file mode/size differs')
        with file.open('rb') as f:
            for e in cells:
                chunked+=int(e.get('chunk_index') is not None)
                for block in hex_bytes(e):
                    p.stdin.write(block);need(f.read(len(block))==block,f'preauthor embedded source bytes differ {name}')
                if e.get('chunk_index') is not None:
                    # independently hash each 64MiB chunk from declared hex, not inferred from whole file
                    need(b3(bytes.fromhex(e['content_hex']))==e['blake3'],'embedded chunk digest differs')
            need(not f.read(1),f'trailing embedded source file bytes {name}')
        if len(cells)==1:check.append(f'{cells[0]["blake3"]}  {file}\n')
        payload+=size;leaf_hashes.append((name,[e['blake3'] for e in cells]))
    digest,err=p.communicate();need(p.returncode==0,err[-400:]);digest=digest.decode().strip()
    need(digest==plan['digest_blake3'] and payload==plan['size_bytes'],'preauthor reconstructed tree not actual plan identity')
    if check:run([B,'--check','--quiet'],''.join(check).encode())
    return {'digest_blake3':digest,'embedded_leaves':len(rec['files']),'unique_files':len(groups),'parents':len(parents),'payload_bytes':payload,'chunk_entries':chunked,'leaf_inventory_blake3':b3(json.dumps(leaf_hashes,ensure_ascii=False,separators=(',',':')).encode())}
def role_file(rec,plan):
    files=rec['files'];need(len(set(e['path'] for e in files))==1,'file role has multiple members')
    files=sorted(files,key=lambda e:e.get('chunk_index',-1));p=subprocess.Popen([B,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    n=0
    for e in files:
        for chunk in hex_bytes(e):p.stdin.write(chunk);n+=len(chunk)
        if e.get('chunk_index') is not None:need(b3(bytes.fromhex(e['content_hex']))==e['blake3'],'file role chunk mismatch')
    d,err=p.communicate();need(p.returncode==0,err[-400:]);d=d.decode().strip()
    if rec['metadata']['bootstrap_profile_class']!='bootstrap-source':need((n,d)==(plan['size_bytes'],plan['digest_blake3']),'preauthor file role plan mismatch')
    return {'identity':d,'bytes':n,'embedded_leaves':len(files),'path':files[0]['path']}
def scan_profile(path,phase,plan,pre_roots,known):
    found={};role_rows={};original={};program={};counts=collections.Counter()
    for rec in entries(path):
        ident=rec['identity'];need(ident not in found,f'duplicate profile record {ident}');found[ident]=1
        kind=rec.get('metadata',{}).get('bootstrap_profile_class')
        if kind is not None:
            need(kind in ROLE and kind not in role_rows,'duplicate or unknown role class')
            row={'profile_class':kind,'role':ROLE[kind],'record_identity':ident,'record_content_blake3':rec['content_blake3'],'record_body_blake3':fingerprint(rec),'entry_count':len(rec['files']),'payload_bytes':rec['payload_bytes']}
            if phase=='input':
                need(canonical_content(rec)==rec['content_blake3'],'preauthor record content invalid')
                if kind in DIR: row['reconstruction']=role_tree(rec,plan[ROLE[kind]])
                else: row['reconstruction']=role_file(rec,plan[ROLE[kind]])
            else:
                if kind=='mantle-source':need(row['record_body_blake3']==pre_roots[kind]['record_body_blake3'],'authored compiler source body changed')
                if kind=='vendored-cargo-inputs':need(row['record_body_blake3']==pre_roots[kind]['record_body_blake3'],'vendor body changed')
                if kind in ('stagex-lineage','stagex-seed','stagex-source-bundle'):
                    need(row['record_body_blake3']==pre_roots[kind]['record_body_blake3'],'original stagex role body changed')
            role_rows[kind]=row
            counts['role_embedded']+=len(rec['files'])
        elif ident in PROGS:
            need(phase=='output','preauthor input has program authority');program[ident]={'body':fingerprint(rec),'content':rec['content_blake3'],'entries':len(rec['files']),'payload':rec['payload_bytes']}
            need(program[ident]==known['program'][ident],'output program differs from augmented native closure')
        else:
            original[ident]=fingerprint(rec)
            if phase=='output':need(ident in known['original'] and original[ident]==known['original'][ident],f'output native body differs {ident}')
        del rec
    need(len(role_rows)==7 and len(original)==59 and (phase=='input' or len(program)==4),'profile records incomplete')
    return {'roles':role_rows,'original':original,'program':program,'record_count':len(found),'counts':dict(counts)}
def scan_closure(path,programs=False):
    seen={};original={};prog={}
    for rec in entries(path):
        ident=rec['identity'];need(ident not in seen,f'closure duplicate {ident}');seen[ident]=1
        body=fingerprint(rec)
        if ident in PROGS:
            need(programs,'program already in preauthor plan')
            need(canonical_content(rec)==rec['content_blake3'],'authored source record digest invalid')
            prog[ident]={'body':body,'content':rec['content_blake3'],'entries':len(rec['files']),'payload':rec['payload_bytes']}
            # complete record files/archives/physical tree and recursive NAR
            prog[ident]['physical']=verify_program(rec)
        else: original[ident]=body
        del rec
    need(len(original)==59 and len(prog)==(4 if programs else 0),'native record inventory incomplete')
    return {'original':original,'program':prog,'record_count':len(seen),'raw_blake3':filehash(path)}
def verify_program(rec):
    ident=rec['identity'];root=C/'program-sources'/ident;archive=C/'program-sources'/f'{ident}.tar'
    need(root.is_dir() and archive.is_file(),'missing authored program physical source/archive')
    paths=walk(root);files={k:(v,s) for k,(v,s) in paths.items() if stat.S_ISREG(s.st_mode)}
    grouped=collections.defaultdict(list)
    for e in rec['files']:
        need(e['file_type']=='regular' and e['executable'] is False,'nonregular/executable program member');grouped[rel(e['path'])].append(e)
    need(set(grouped)==set(files),'authored member inventory differs')
    parents=set()
    for relname in grouped:
        for p in PurePosixPath(relname).parents:
            if str(p)!='.':parents.add(str(p))
    need(set(paths)==set(grouped)|parents,'authored program has extra/missing directories')
    checks=[];total=0;member_hashes={}
    for name,cells in grouped.items():
        cells.sort(key=lambda e:e.get('chunk_index',-1));f,st=files[name]
        need(stat.S_IMODE(st.st_mode)==0o644 and st.st_size==sum(e['size'] for e in cells),'member mode/size differs')
        with f.open('rb') as stream:
            for e in cells:
                for chunk in hex_bytes(e):need(stream.read(len(chunk))==chunk,f'embedded program bytes differ {ident}/{name}')
                if e.get('chunk_index') is not None:need(b3(bytes.fromhex(e['content_hex']))==e['blake3'],'authored chunk mismatch')
            need(not stream.read(1),'program trailing bytes')
        if len(cells)==1:checks.append(f'{cells[0]["blake3"]}  {f}\n')
        total+=st.st_size;member_hashes[name]=filehash(f)
    need(total==rec['payload_bytes'],'program payload total differs')
    if checks:run([B,'--check','--quiet'],''.join(checks).encode())
    with tarfile.open(archive) as tf:
        names=set()
        for m in tf:
            need(m.isfile() and m.name.startswith('source/') and m.mode==0o644 and m.uid==m.gid==m.mtime==0,'bad tar member metadata')
            name=rel(m.name[7:]);need(name not in names and name in files,'tar inventory mismatch');names.add(name)
            need(m.size==files[name][1].st_size,'tar member size differs')
            with tf.extractfile(m) as source, files[name][0].open('rb') as physical:
                while True:
                    a=source.read(1024*1024);need(a==physical.read(len(a)),'tar member bytes differ')
                    if not a:break
        need(names==set(grouped),'tar missing member')
    nar=run([N,'--extra-experimental-features','blake3-hashes','hash','path','--type','blake3','--base16',str(root)]).decode().strip()
    need(nar==rec['metadata']['hash'] and rec['metadata']['url']==archive.as_uri() and rec['metadata']['hash_mode']=='recursive','NAR identity/ownership differs')
    return {'tree_nar':nar,'archive_blake3':filehash(archive),'source_members':member_hashes,'archive_member_count':len(grouped),'record_entry_count':len(rec['files']),'payload_bytes':total}
def git_copies(subject):
    db=Path(subject['original_aws']['capture']);need(db.name=='aws-lc.git' and db.is_dir(),'wrong original Git pointer')
    commit=subject['original_aws']['commit'];tree=run([G,f'--git-dir={db}','rev-parse',commit+'^{tree}']).decode().strip()
    need(tree==subject['original_aws']['git_tree_sha1'],'original Git tree mismatch')
    raw=run([G,f'--git-dir={db}','ls-tree','-rz','--full-tree',commit]);expected={};modes=collections.Counter()
    for row in raw.split(b'\0'):
        if not row:continue
        prefix,name=row.split(b'\t',1);mode,typ,blob=prefix.split(b' ');name=name.decode()
        need(typ==b'blob' and mode in (b'100644',b'100755') and name not in expected,'unsupported Git tree member')
        expected[name]=(blob.decode(),int(mode,8));modes[mode.decode()]+=1
    rows=[]
    for label,root in [('frozen',F/'bootstrap/source-inputs'/f'aws-lc-original-{commit}'),('prepared',C/'inputs/mantle-source/bootstrap/source-inputs'/f'aws-lc-original-{commit}')]:
        treefiles=walk(root);actual={k:(p,s) for k,(p,s) in treefiles.items() if not stat.S_ISDIR(s.st_mode)}
        need(set(actual)==set(expected),'AWS Git/copy inventory differs')
        check=[]
        for name,(blob,mode) in expected.items():
            f,s=actual[name];need(stat.S_ISREG(s.st_mode) and stat.S_IMODE(s.st_mode)==stat.S_IMODE(mode),'AWS Git/copy mode differs')
            check.append(str(f)+'\n')
        observed=run([G,f'--git-dir={db}','hash-object','--no-filters','--stdin-paths'],''.join(check).encode()).decode().splitlines()
        need(observed==[expected[name][0] for name in expected],'AWS original blob bytes differ')
        rows.append({'label':label,'blobs':len(expected),'modes':dict(modes),'tree':tree,'copied_git_blobs_verified':len(observed)})
    return rows
def frozen_copy():
    text=(F/'src/self_build.rs').read_text();m=re.search(r'STAGED_SOURCE_TOP_LEVEL_ENTRIES: &\[&str\] = &\[(.*?)\];',text,re.S)
    need(m is not None,'frozen capture allowlist absent')
    names=set(re.findall(r'"([^"]+)"',m.group(1)))-{'vendor-deps'}
    orig=walk(F);actual=walk(C/'inputs/mantle-source')
    wanted={name:(p,s) for name,(p,s) in orig.items() if name.split('/')[0] in names and '.git' not in name.split('/')}
    need(set(actual)==set(wanted),'authored compiler tree additions or missing original parents')
    files=0;bytes_count=0
    for name,(p,s) in wanted.items():
        q,t=actual[name]
        need(stat.S_IFMT(t.st_mode)==stat.S_IFMT(s.st_mode) and stat.S_IMODE(t.st_mode)==stat.S_IMODE(s.st_mode),'compiler source leaf/parent mode differs')
        if stat.S_ISREG(s.st_mode):
            need(t.st_size==s.st_size,'source leaf size differs')
            with p.open('rb') as a,q.open('rb') as b:
                while True:
                    x=a.read(1024*1024);need(x==b.read(len(x)),f'frozen Mantle source bytes changed {name}')
                    if not x:break
            files+=1;bytes_count+=s.st_size
        elif stat.S_ISLNK(s.st_mode):need(os.readlink(p)==os.readlink(q),'frozen source link changed')
    return {'capture_roots':sorted(names),'original_nodes_compared':len(wanted),'original_leaves_compared':files,'payload_bytes':bytes_count,'authored_compiler_tree_additions':len(set(actual)-set(wanted))}
def routes_and_helpers(identity):
    maps=identity['source_copy_bindings']['frozen_helper_translator_copies']
    for row in maps:
        p=Path(row['frozen_path']);q=Path(row['program_member']);need(p.is_file() and q.is_file() and p.read_bytes()==q.read_bytes() and filehash(q)==row['blake3'],'helper/translator byte mismatch')
    native=F/'src/source_built_mantle_programs_native.rs';t=native.read_text();m=re.search(r'const PIPE_SOURCE: &str = r###"(.*?)"###;',t,re.S);need(m is not None,'pipe source absent');pipe=m.group(1)
    result=[];parity=[]
    for stage in ('mantle-stage1','mantle-stage2'):
        p=C/'program-sources/source-built-mantle-programs/bootstrap/source-built'/f'{stage}.json';recipe=json.loads(p.read_bytes());steps={s['id']:s for s in recipe['steps']};need(len(steps)==len(recipe['steps']),'duplicate recipe step')
        selected=[];pipes=[]
        for s in recipe['steps']:
            name=s['id'];aws=re.fullmatch(r'unit-00037-script\.perlasm-(\d+)\.generate',name);ring=re.fullmatch(r'unit-00031-script\.sources\.perl-(\d+)\.authenticate',name)
            if aws:
                args=s['operation']['arguments'];one=lambda word:word['parts'][0];source=one(args[2]);original=one(args[1]);path=source['path']
                need(source['kind']=='input' and source['name']=='source' and original['path']==path and original['name']=='original-script','AWS source selection differs')
                need(s['inputs']['original-script']==s['inputs']['source'] and s['inputs']['source']['action_id']==stage+'/unit-00037-script.perlasm-source','AWS generator unauthenticated source')
                i=aws.group(1);pipeid=f'unit-00037-script.perlasm-{i}.pipe-source';tr=steps[f'unit-00037-script.perlasm-{i}.translate']
                need(steps[pipeid]['operation']['text']==pipe and s['inputs']['pipe-source']=={'kind':'produced','action_id':stage+'/'+pipeid,'output_id':'source'},'AWS pipe not frozen')
                need(tr['inputs']['raw']=={'kind':'produced','action_id':stage+'/'+name,'output_id':'raw'},'AWS translator missing raw product')
                if '/rsaz-' in path:need(s['inputs']['aws-as-probe']=={'kind':'produced','action_id':stage+'/unit-00037-script.assembler-version','output_id':'stderr'},'AWS probe missing')
                selected.append(('aws',name,path,one(args[4])['text']));pipes.append(pipeid)
            if ring:
                args=s['operation']['arguments'];one=lambda word:word['parts'][0];path=one(args[3])['path'];prefix=name[:-len('.authenticate')];gen=steps[prefix+'.generate'];tr=steps[prefix+'.translate'];pipeid=prefix+'.pipe-source';out=steps[prefix+'.result']
                need(s['inputs']['package']=={'kind':'source','role':'vendor-inputs','path':'ring'},'ring package source not original')
                need(gen['inputs']['original-script']=={'kind':'produced','action_id':stage+'/'+name,'output_id':'authenticated'},'ring authenticated text dropped')
                need(steps[pipeid]['operation']['text']==pipe and gen['inputs']['pipe-source']=={'kind':'produced','action_id':stage+'/'+pipeid,'output_id':'source'},'ring pipe not frozen')
                need(tr['inputs']['raw']=={'kind':'produced','action_id':stage+'/'+prefix+'.generate','output_id':'raw'} and out['inputs']['assembly']=={'kind':'produced','action_id':stage+'/'+prefix+'.translate','output_id':'assembly'},'ring generator/translator result mismatch')
                selected.append(('ring',name,path,one(args[4])['text']));pipes.append(pipeid)
        need(len(selected)==len(pipes) and all(steps[p]['outputs']['source']['kind']=='text' for p in pipes),'pipe output kind differs')
        for typ,name,path,out in selected:
            root=C/('inputs/mantle-source/bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9' if typ=='aws' else 'inputs/vendor-deps/ring')
            script=root/path;need(script.is_file(),'selected physical script missing')
            source=script.read_text();need(len(re.findall(r'open OUT,\s*"\|.*?\$xlate.*?\$output',source))==1,f'unsupported script pipe boundary {script}')
            if typ=='aws' and path.endswith(('rsaz-2k-avx512.pl','rsaz-3k-avx512.pl','rsaz-4k-avx512.pl')):need(source.count('`$ENV{CC} -Wa,-v -c -o /dev/null -x assembler /dev/null 2>&1`')==1,'original assembler probe differs')
        result.append({'stage':stage,'recipe_blake3':filehash(p),'selected_count':len(selected),'aws':sum(x[0]=='aws' for x in selected),'ring':sum(x[0]=='ring' for x in selected),'pipe_sources':len(pipes),'selected_inventory_blake3':b3(json.dumps(sorted(selected),separators=(',',':')).encode())});parity.append(sorted(selected))
        del steps,recipe
    need(parity[0]==parity[1],'stage script selection differs')
    return {'stages':result,'stage_parity':True,'frozen_pipe_blake3':b3(pipe.encode()),'helper_copies':len(maps),'native_llvm_owned_helper_members':[str(Path(r['program_member']).relative_to(C/'program-sources')) for r in maps if 'llvm-cmake-' in r['program_member']]}
def main():
    subj=json.loads((R/'review-subject-70db0868-primary-sources-aws-db-fixed.json').read_bytes());need(subj['frozen_product_revision']==REV,'subject revision differs')
    ident=json.loads((R/'inspection-70db0868-primary-sources-identity-aws-db-fixed/identity-review.json').read_bytes())
    plan=json.loads((C/'source-program-preparation-input-plan.json').read_bytes());want={r['role']:r for r in plan['source_inputs']}
    need(len(want)==len(ROLE)==7,'source plan roles differ')
    OUT['input_hashes']={'remote_subject':filehash(R/'review-subject-70db0868-primary-sources-aws-db-fixed.json'),'preauthor_profile':filehash(R/'source-profile-70db0868-primary-sources.json'),'prepared_profile':filehash(C/'source-profile.json'),'preauthor_plan':filehash(C/'source-program-preparation-input-plan.json'),'preparation':filehash(C/'source-program-preparation.json')}
    before=scan_profile(R/'source-profile-70db0868-primary-sources.json','input',want,{},{});OUT['preauthor']={'roles':before['roles'],'record_count':before['record_count'],'original_native_record_bodies_blake3':before['original']}
    plan_native=scan_closure(C/'native-source-closure.json');need(plan_native['original']==before['original'],'59 native bodies changed between bootstrap role and plan')
    augmented=scan_closure(C/'inputs/native-source-authority/native-source-closure.json',True);need(augmented['original']==plan_native['original'],'original native closure bodies changed after authoring')
    after=scan_profile(C/'source-profile.json','output',want,before['roles'],augmented)
    need(after['original']==before['original'],'original native bodies changed in launch profile')
    OUT['closures']={'bootstrap_source_raw_blake3':before['roles']['bootstrap-source']['reconstruction']['identity'],'preauthor_plan_raw_blake3':plan_native['raw_blake3'],'augmented_raw_blake3':augmented['raw_blake3'],'original_native_bodies_match_three_boundaries':59,'preauthor_native_records':plan_native['record_count'],'augmented_records':augmented['record_count'],'output_profile_records':after['record_count']}
    OUT['postauthor_roles']=after['roles'];OUT['programs']=augmented['program'];OUT['source_copy']=frozen_copy();OUT['aws_git']=git_copies(subj);OUT['selected_sources']=routes_and_helpers(ident)
    guards=ident['initial_authority_absence'];paths=guards['absent_first_launch_outputs']+guards['absent_provider_store_roots'];held=R/'cold-proof-70db0868-primary-sources-gpt6sol'
    need(len(paths)==27 and all(not os.path.lexists(p) for p in paths) and not os.path.lexists(held),'initial authority or actual held cold proof root exists')
    OUT['initial_absence']={'checked_legacy_and_provider_paths':len(paths),'actual_held_cold_root':str(held),'held_cold_root_absent':True}
    OUT['frozen_consumer_hashes']={path:filehash(F/path) for path in ident['frozen_identity_consumers']}
    OUT['inspector_hashes']={n:filehash(R/'review-tools-70db0868-primary-sources/identity-review'/n) for n in ('identity-audit.py','preauthor-role-physical-audit.py')}
    OUT['commands']=['b3sum --no-names <entire-preauthor-and-output-profiles-and-closures>','b3sum --no-names (stdin; embedded leaf/parent framing, canonical record bodies)','b3sum --check --quiet (all unchunked preauthor leaves and program members)','git --git-dir=<original-aws.git> rev-parse <commit>^{tree}','git --git-dir=<original-aws.git> ls-tree -rz --full-tree <commit>','git --git-dir=<original-aws.git> hash-object --no-filters --stdin-paths (both full copies)','nix --extra-experimental-features blake3-hashes hash path --type blake3 --base16 <all four authored program directories>','python3 inert stdin analyzer: complete all seven roles, 59 native bodies across input/plan/closure/output, four program records, all source leaves, archive members, AWS inventory, source selection and initial path absence']
    print(json.dumps(OUT,sort_keys=True))
if __name__=='__main__':main()
