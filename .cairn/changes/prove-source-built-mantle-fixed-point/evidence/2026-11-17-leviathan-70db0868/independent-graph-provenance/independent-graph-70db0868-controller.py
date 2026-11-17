#!/usr/bin/env python3
"""Inert independent ControllerSpec declaration and complete template-role census."""
import collections,json,re
from pathlib import Path
BASE=Path('/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-70db0868-primary-sources')
RPT=Path('/home/brittonr/sbfp-20261113-berdKH/inspection-70db0868-primary-sources-graph-aws-db-fixed/controller-template-audit.json')
SOURCES={
'full-source-native-provider':'program-sources/fixed-point-native-source-program/source-built/native/program.json',
'full-source-rust-provider':'inputs/rust-source-archives/source-built/rust/program.json',
'mantle-stage1':'program-sources/source-built-mantle-programs/bootstrap/source-built/mantle-stage1.json',
'mantle-stage2':'program-sources/source-built-mantle-programs/bootstrap/source-built/mantle-stage2.json'}
report=json.load(RPT.open('rb'));results={};issues=[]
def mark(stage,sid,problem):
 if len(issues)<30:issues.append({'stage':stage,'controller':sid,'problem':problem})
for stage,path in SOURCES.items():
 raw=json.load((BASE/path).open('rb'))
 specs=[step for step in raw['steps'] if step['operation']['kind']=='controller']
 catalogs=report['per_stage'][stage]['controller_template_catalogs']
 byid={c['controller_spec']:c for c in catalogs};counters=collections.Counter()
 if len(catalogs)!=len(byid) or len(specs)!=len(catalogs):mark(stage,'all','catalog duplicate or missing controller declaration')
 for step in specs:
  s=step['operation']['specification'];sid=s['id'];c=byid.get(sid);counters['specs']+=1
  if not c:mark(stage,sid,'missing exact catalog');continue
  if step['id']!=sid or c['controller_step']!=sid:mark(stage,sid,'outer declaration/inner catalog ID differs')
  t=s['tree_bytes_max'];f=s['file_bytes_max']
  expected_outputs={n:{'kind':k,'bytes_max':b} for n,k,b in [('checkpoint','tree',t),('workspace','tree',t),('times','text',f),('generated','tree',t),('stdout','object',f),('stderr','text',f),('exit-status','text',4)]}
  if step['outputs']!=expected_outputs:mark(stage,sid,'controller output bounds/kinds differ from constructor')
  if not 0<s['commands_max'] or not s['slots'] or not (0<=min(s['restart_indices']) and max(s['restart_indices'])<len(s['start_arguments'])) or len(set(s['restart_indices']))!=3:mark(stage,sid,'invalid catalog/restart bounds')
  refs=[s['controller'],*s['inputs'].values()];slot_ids=set();expected_validation=[sid+'-start'];expected_runtime=['@controller-'+sid+'-start','@controller-'+sid+'-000000-resume','@controller-'+sid+'-finish'];generated=[]
  for slot in s['slots']:
   id=slot['id'];v=slot['executable']['kind'];vdata=slot['executable']['value'];counters['slots']+=1;counters['variant:'+v]+=1
   if id in slot_ids:mark(stage,sid,'duplicate command slot')
   slot_ids.add(id);refs.extend(slot['inputs'].values())
   if v=='fixed':refs.append(vdata)
   elif v=='adapted':refs.extend([vdata['executable'],vdata['original_image']])
   elif v=='inventory':refs.extend(ref for name,ref in vdata)
   elif v=='request-output':
    producers=vdata['producer_slots']
    if not producers or any(not prior['generated_outputs'] for prior in s['slots'] if prior['id'] in producers) or any(p not in [prior['id'] for prior in s['slots']] for p in producers):mark(stage,sid,'invalid generated executable producer')
   else:mark(stage,sid,'unknown executable selector variant')
   base=sid+'-000000-'+id
   names=[base+'-prepare']+([base+'-executable'] if v in ('inventory','request-output') else [])+[base+'-command',base+'-observation']
   expected_validation.extend(names)
   expected_runtime.extend(['@controller-'+base+'-prepare','@controller-'+base+'-command','@controller-'+base+'-observation']+(['@controller-'+base+'-executable'] if v in ('inventory','request-output') else []))
   counters['validation_templates']+=len(names);counters['runtime_payload_prepare']+=1;counters['runtime_payload_consume']+=1;counters['observations']+=1
   counters['same_slot_seed_pairs']+=2
   if not all(n in slot for n in ('id','executable','inputs','runtime_mounts','exit_policy','merge_stderr','generated_outputs')):mark(stage,sid,'incomplete original slot declaration')
  expected_validation.extend([sid+'-000000-resume',sid+'-finish']);counters['validation_templates']+=3
  if expected_validation!=c['validation_template_step_ids']:mark(stage,sid,'generated validation role IDs/order differ')
  if expected_runtime!=c['runtime_step_template_ids']:mark(stage,sid,'resolved runtime role IDs/order differ')
  if len(set(expected_validation))!=len(expected_validation):mark(stage,sid,'generated validation IDs collide')
  actual=[v for k,v in sorted(step['inputs'].items()) if k.startswith('controller-input-')]
  if len(actual)!=len(set(json.dumps(r,sort_keys=True) for r in refs)) or set(json.dumps(r,sort_keys=True) for r in actual)!=set(json.dumps(r,sort_keys=True) for r in refs):mark(stage,sid,'controller inputs not exact expected reference set')
  if len(actual)!=len(step['inputs']):
   guard={condition['status_input'] for condition in (step['operation'].get('guard') or {}).get('conditions',[])}
   if set(step['inputs'])-set(k for k in step['inputs'] if k.startswith('controller-input-'))!=guard:mark(stage,sid,'unknown outer controller input')
  if c['slot_count']!=len(s['slots']) or c['file_bytes_max']!=f or c['tree_bytes_max']!=t:mark(stage,sid,'catalog bounds or slot count differs')
 results[stage]={'counters':dict(counters),'catalogs_equal':len(catalogs)==len(specs),'expected_report_templates':report['per_stage'][stage]['generated_template_step_count'],'expected_report_seeds':report['per_stage'][stage]['generated_output_seeds']['total']}
print(json.dumps({'per_stage':results,'bounded_findings':issues,'all_catalogs_valid':not issues},separators=(',',':')))
