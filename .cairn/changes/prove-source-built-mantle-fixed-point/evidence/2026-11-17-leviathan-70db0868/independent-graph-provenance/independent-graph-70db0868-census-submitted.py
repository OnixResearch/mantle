#!/usr/bin/env python3
"""Inert independent six-recipe data census; run only trusted Python, never recipe code."""
import collections, gc, json, subprocess, sys
from pathlib import Path
HOST='leviathan.cymric-daggertooth.ts.net'
BASE=Path('/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-70db0868-primary-sources')
RECIPES=[
 ('stagex-transition','program-sources/source-built-stagex-programs/stagex-transition.json'),
 ('stagex-provider-publication','program-sources/source-built-stagex-programs/stagex-provider-publication.json'),
 ('full-source-native-provider','program-sources/fixed-point-native-source-program/source-built/native/program.json'),
 ('full-source-rust-provider','inputs/rust-source-archives/source-built/rust/program.json'),
 ('mantle-stage1','program-sources/source-built-mantle-programs/bootstrap/source-built/mantle-stage1.json'),
 ('mantle-stage2','program-sources/source-built-mantle-programs/bootstrap/source-built/mantle-stage2.json')]

INDEX={
 'full-source-native-provider':'program-sources/fixed-point-native-source-program/source-built/native/program.expansion.json',
 'full-source-rust-provider':'inputs/rust-source-archives/source-built/rust/program.json.expansion.json',
 'stagex-transition':'program-sources/source-built-stagex-programs/stagex-transition.json.expansion.json',
}
# NB Source-built Mantle route is authoritative for the exact recipe members.
def refs(v):
 if isinstance(v,dict):
  if v.get('kind') in ('produced','source','manifest-source') and ('action_id' in v or 'role' in v):
   yield v
  else:
   for child in v.values():yield from refs(child)
 elif isinstance(v,list):
  for child in v:yield from refs(child)
def issue(rows,stage,place,text):
 if len(rows)<30:rows.append({'stage':stage,'at':place,'error':text})
def main():
 results=[]; allowners={}; problems=[]; grand=collections.Counter()
 for stage,relative in RECIPES:
  p=BASE/relative
  with p.open('rb') as stream:data=json.load(stream)
  if data.get('stage_id')!=stage:issue(problems,stage,str(p),'wrong stage id')
  steps=data['steps']; seen={}; owner={stage+'/'+s['id']:(i,s['outputs'],s['operation']['kind']) for i,s in enumerate(steps)}
  if len(owner)!=len(steps):issue(problems,stage,'steps','duplicate action IDs')
  count=collections.Counter(); local_issues=[]; dependencies={}; export_refs=[]
  for i,s in enumerate(steps):
   sid=s['id']; op=s['operation']; operation=op['kind']; input_map=s['inputs']; actual=set()
   if sid in seen:issue(local_issues,stage,sid,'duplicate step')
   count['steps']+=1;count['operation:'+operation]+=1
   for name,r in input_map.items():
    count['input_refs']+=1
    if r['kind']=='produced':
     count['produced_inputs']+=1; aid=r['action_id']; oid=r['output_id']; ns=aid.split('/',1)[0]
     target=owner.get(aid) if ns==stage else allowners.get(ns,{}).get(aid)
     if target is None:issue(local_issues,stage,sid+'.'+name,'Produced input missing exact local/prior producer')
     else:
      src_idx,outputs,producer_op=target
      if oid not in outputs:issue(local_issues,stage,sid+'.'+name,'Produced input output_id absent')
      if ns==stage:
       if src_idx>=i:issue(local_issues,stage,sid+'.'+name,'Produced input not prior in recipe order')
       actual.add(aid)
    elif r['kind'] not in ('source','manifest-source'):issue(local_issues,stage,sid+'.'+name,'unknown input reference kind')
   if operation=='observe-phase':actual.add(stage+'/'+op['phase'])
   count['dependency_edges']+=len(actual);dependencies[stage+'/'+sid]=actual
   for output,name in op.get('output_seeds',{}).items():
    count['raw_seeds']+=1
    r=input_map.get(name);dest=s['outputs'].get(output)
    if not dest or output in ('stdout','stderr','exit-status') or not r or r['kind']!='produced':
     issue(local_issues,stage,sid+'.seed.'+output,'seed destination or nonguard Produced source absent');continue
    aid=r['action_id'];src=(owner if aid.startswith(stage+'/') else allowners.get(aid.split('/',1)[0],{})).get(aid)
    if src is None:issue(local_issues,stage,sid+'.seed.'+output,'seed producer absent');continue
    src_idx,src_outputs,_=src;item=src_outputs.get(r['output_id']);
    if not item or item['kind']!=dest['kind'] or item['bytes_max']>dest['bytes_max'] or aid.startswith(stage+'/') and src_idx>=i:
     issue(local_issues,stage,sid+'.seed.'+output,'seed wrong kind, bound or producer order')
    else:count['raw_seeds_valid']+=1
   if operation=='observe-phase':
    count['phase_observations']+=1
    phase=stage+'/'+op['phase'];src=owner.get(phase)
    if not src or src[0]>=i:issue(local_issues,stage,sid,'ObservePhase producer absent or later')
    if op.get('state'):
     count['phase_alternatives']+=1;state=op['state'];r=input_map.get(state['skipped_input']);
     if not r or r['kind']!='produced' or src and state['output'] not in src[1]:issue(local_issues,stage,sid,'ObservePhase executed/skipped alternative broken')
   if operation=='exec':
    g=op.get('guard')
    if g:
     count['guarded_execs']+=1
     for cond in g['conditions']:
      count['guard_conditions']+=1;r=input_map.get(cond['status_input'])
      if not r or r.get('output_id')!='exit-status':issue(local_issues,stage,sid,'guard does not consume exact Produced exit-status')
      elif r['action_id'].startswith(stage+'/') and r['action_id'] not in seen:issue(local_issues,stage,sid,'guard status not prior in source order')
    if op.get('invocation_input'):
     count['invocation_requests']+=1;r=input_map.get(op['invocation_input'])
     if not r or r['kind']!='produced' or r['output_id'] in ('exit-status',):issue(local_issues,stage,sid,'invocation request not ordinary Produced')
     else:
      action=r['action_id'];producer=seen.get(action)
      if producer and producer['operation']['kind']=='write-text':
       count['static_invocations']+=1
       try:
        req=json.loads(producer['operation']['text']);
        if not isinstance(req,dict) or not {'arguments','environment'}<=req.keys():raise ValueError('missing required invocation fields')
        if set(req)-{'arguments','environment','cwd','argv0','executable_path','workspace_mounts','readonly_input_roots','process_attributes','descriptors'}:raise ValueError('unknown invocation fields')
        for field in ('cwd','argv0','executable_path'):
         if req.get(field):count['static_invocation_typed_words']+=1
        count['static_invocation_typed_words']+=len(req['arguments'])+len(req['environment'])
       except (ValueError,TypeError,KeyError) as e:issue(local_issues,stage,sid,'invalid static invocation JSON '+str(e))
      else: count['runtime_invocations_unobserved']+=1
   if operation=='controller':
    count['controllers']+=1;spec=op['specification'];count['controller_slots']+=len(spec['slots'])
   seen[stage+'/'+sid]=s
  # Reference occurrences beyond inputs (exports, roots, semantic roots, controller specs).
  for family,value in [('roots',data['roots']),('exports',data['exports']),('semantic_roots',data['semantic_roots'])]:
   for r in refs(value):
    count[family+'_refs']+=1
    if r['kind']=='produced':
     aid=r['action_id'];ns=aid.split('/',1)[0];src=(owner if ns==stage else allowners.get(ns,{})).get(aid)
     if not src or r['output_id'] not in src[1]:issue(local_issues,stage,family,'missing exact root/export producer')
     if family=='roots' and ns!=stage:issue(local_issues,stage,'roots','nonlocal genuine root')
     if ns==stage:export_refs.append(aid)
  stack=export_refs.copy();reached=set()
  while stack:
   aid=stack.pop()
   if aid in reached:continue
   reached.add(aid);stack.extend(dependencies.get(aid,()))
  count['root_dependency_reachable']=len(reached)
  count['root_dependency_unreachable']=len(owner)-len(reached)
  color={};cycles=0
  def visit(start):
   nonlocal cycles
   stack=[(start,False)]
   while stack:
    node,returning=stack.pop()
    if returning:color[node]=2;continue
    if color.get(node)==2:continue
    if color.get(node)==1:cycles+=1;continue
    color[node]=1;stack.append((node,True));stack.extend((x,False) for x in dependencies.get(node,()))
  for node in dependencies:visit(node)
  count['cycles']=cycles
  if stage in INDEX:
   idx=json.load((BASE/INDEX[stage]).open())
   part=idx.get('derivations',idx.get('stages',idx.get('tools',{})))
   if stage=='full-source-native-provider':
    if isinstance(part,dict):partition={k:v.get('step_ids',[]) for k,v in part.items()}
    else:partition={str(i):d.get('step_ids',[]) for i,d in enumerate(part)}
   else:partition={k:v['step_ids'] for k,v in part.items()}
   ids=[x for group in partition.values() for x in group]
   count['partition_nodes']=len(partition);count['partition_assignments']=len(ids)
   count['partition_unassigned']=len(set(s['id'] for s in steps)-set(ids));count['partition_unknown']=len(set(ids)-set(s['id'] for s in steps));count['partition_duplicates']=len(ids)-len(set(ids))
   if any((count[key] for key in ('partition_unassigned','partition_unknown','partition_duplicates'))):issue(local_issues,stage,'index','not exact partition')
  allowners[stage]=owner
  results.append({'stage':stage,'path':str(p),'size_bytes':p.stat().st_size,'counters':dict(count),'diagnostics':local_issues[:15]})
  grand.update(count);problems.extend(local_issues[:15]);del steps,data,seen;gc.collect()
 print(json.dumps({'stage_results':results,'grand_counters':dict(grand),'bounded_findings':problems[:25]},separators=(',',':')))
if __name__=='__main__':main()
