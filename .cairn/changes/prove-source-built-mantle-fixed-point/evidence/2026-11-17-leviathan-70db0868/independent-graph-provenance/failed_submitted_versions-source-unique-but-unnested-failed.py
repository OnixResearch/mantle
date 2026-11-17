#!/usr/bin/env python3
"""Independent inert BLAKE3 source framing and two complete Mantle plan joins."""
import collections,json,os,subprocess,sys
from pathlib import Path
BASE=Path('/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-70db0868-primary-sources')
B3='/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
ROOT=BASE/'inputs/mantle-source'
RECORD=BASE/'program-sources/source-built-mantle-programs/bootstrap/source-built'
STAGES=('mantle-stage1','mantle-stage2')
RPT=Path('/home/brittonr/sbfp-20261113-berdKH/inspection-70db0868-primary-sources-graph-aws-db-fixed/mantle-selector-audit.json')
ROOT_DIGEST='e6a8d28aa4268856875f0f40561347d9e67c2181b33e14e8b2d7100c3d48437e'
def checksum(p):
 r=subprocess.run([B3,'--no-names',str(p)],text=True,capture_output=True,check=True);return r.stdout.strip()
def source_hash():
 files=[];pending=[ROOT];visited=0
 while pending:
  directory=pending.pop()
  with os.scandir(directory) as entries:
   for entry in entries:
    visited+=1
    if entry.name in ('.agent','.git','.jj','.pi','target') or directory==ROOT and entry.name=='cairn':continue
    if entry.is_dir(follow_symlinks=False):pending.append(Path(entry.path))
    elif entry.is_file(follow_symlinks=False):files.append(Path(entry.path))
 files.sort()
 process=subprocess.Popen([B3,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 for p in files:
  rel=p.relative_to(ROOT).as_posix().encode();length=p.stat().st_size
  process.stdin.write(b'file\0'+rel+b'\0'+str(length).encode()+b'\0')
  read=0
  with p.open('rb') as stream:
   while x:=stream.read(1024*1024):read+=len(x);process.stdin.write(x)
  if read!=length:raise ValueError('source file changed during read')
  process.stdin.write(b'\0')
 digest,err=process.communicate(timeout=3600)
 if process.returncode:raise RuntimeError(err.decode())
 return {'digest_blake3':digest.decode().strip(),'file_count':len(files),'visited_entries':visited,'matches_expected':digest.decode().strip()==ROOT_DIGEST}
def primary(args):
 return next((arg for arg in args if isinstance(arg,str) and not arg.startswith('-') and arg.endswith('.rs')),None)
def literal(word):
 parts=word.get('parts',[])
 return ''.join(p['text'] for p in parts) if parts and all(p.get('kind')=='literal' for p in parts) else None
def exact_plan_operand(derivation,step,args):
 raw=primary(derivation['derivation']['args'])
 if not raw:return False,'no planned source operand'
 staging='/home/brittonr/sbfp-20261113-berdKH/.prepared-cohort-70db0868-primary-sources.source-built-fixed-point-staging-2275366/inputs/'
 expected=raw if raw.startswith('/') else str(ROOT/raw)
 # Planned absolute source roots are the exact staging source roots, without normalization.
 if expected.startswith(staging+'mantle-source/'):
  role='mantle-source';member=expected[len(staging+'mantle-source/'):]
 elif expected.startswith(staging+'vendor-deps/'):
  role='vendor-inputs';member=expected[len(staging+'vendor-deps/'):]
 elif expected.startswith(str(ROOT)+'/'):
  role='mantle-source';member=expected[len(str(ROOT)+'/'):]
 elif expected.startswith(str(BASE/'inputs/vendor-deps')+'/'):
  role='vendor-inputs';member=expected[len(str(BASE/'inputs/vendor-deps')+'/'):]
 else:return False,'unknown source role for planned unnormalized operand '+repr(expected)
 matches=[]
 for word in args:
  if not isinstance(word,dict):continue
  parts=word.get('parts')
  if not isinstance(parts,list) or len(parts)!=1 or parts[0].get('kind')!='input':continue
  part=parts[0];ref=step['inputs'].get(part['name'])
  if not ref or ref.get('kind')!='source' or ref.get('role')!=role:continue
  source_member=ref['path'];path=part['path']
  joined=source_member+'/'+path if path and source_member else source_member or path
  # Rust Path::join purely lexical; no normalizing of .. aliases.
  if joined==member:matches.append(part['name'])
 return len(matches)==1,{'role':role,'relative':member,'exact_matches':len(matches),'planned':raw}
def main():
 report=json.load(RPT.open('rb'));checks={}
 measured=source_hash()
 for stage in STAGES:
  planfile=RECORD/(stage+'.unit-plan.json');recipefile=RECORD/(stage+'.json')
  plan=json.load(planfile.open('rb'));recipe=json.load(recipefile.open('rb'))
  derivations=plan['unit_graph']['derivations'];byid={d['unit_id']:d for d in derivations}
  steps={s['id']:s for s in recipe['steps']}
  evidence=report['per_stage'][stage]['unit_evidence']
  c=collections.Counter();errors=[]; seen_plan=set()
  if plan['unit_graph']['derivation_count']!=len(derivations):errors.append('plan derivation count disagrees with list')
  if plan['stage_id']!=stage or recipe['stage_id']!=stage:errors.append('stage mismatch')
  if len(steps)!=len(recipe['steps']):errors.append('duplicate recipe step')
  for e in evidence:
   c['unit_evidence_total']+=1;uid=e['fact_unit_id'];d=byid.get(uid)
   if not d:
    c['missing_plan_unit_ids']+=1
    if len(errors)<10:errors.append('fact unit absent plan: '+uid)
    continue
   seen_plan.add(uid)
   step=steps.get(e['compiler_selector_projection']['consumer_step'])
   if not step or step['operation']['kind']!='exec':
    c['missing_compiler']+=1;continue
   args=step['operation']['arguments']
   if e['compiler_selector_projection']['projection']=='static-template':
    template=steps.get(e['compiler_selector_projection']['template_step']);
    if template and template['operation']['kind']=='write-text':
     req=json.loads(template['operation']['text']);args=req['arguments'];c['static_templates']+=1
    else:c['unresolved_static_template']+=1
   else:c['direct_compiler']+=1
   c['planned_sources_total']+=1
   ok,detail=exact_plan_operand(d,step,args)
   if ok:c['planned_sources_exact']+=1
   else:
    c['planned_sources_invalid']+=1
    if len(errors)<10:errors.append({'unit_id':uid,'step':step['id'],'detail':detail})
  for d in derivations:
   c['plan_derivations_total']+=1
   if d['unit_id'] not in seen_plan:
    c['unclaimed_plan_derivations']+=1
    if len(errors)<10:errors.append('unclaimed plan derivation: '+d['unit_id'])
   if d['package_id']=='path+native#mantle@0.1.0':
    c['root_units_total']+=1
    if d['source_digest']=={'algorithm':'blake3-tree-v1','value':measured['digest_blake3']}:c['root_units_exact']+=1
    else:
     c['root_units_invalid']+=1
     if len(errors)<10:errors.append('root unit source digest mismatch: '+d['unit_id'])
  checks[stage]={'unit_plan':str(planfile),'unit_plan_blake3':checksum(planfile),'recipe':str(recipefile),'recipe_blake3':checksum(recipefile),'counters':dict(c),'diagnostics':errors[:10]}
 print(json.dumps({'source_digest':measured,'stages':checks},separators=(',',':')))
if __name__=='__main__':main()
