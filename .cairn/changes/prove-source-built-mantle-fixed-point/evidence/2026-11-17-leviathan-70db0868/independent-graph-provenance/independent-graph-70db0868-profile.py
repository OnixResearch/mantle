#!/usr/bin/env python3
"""Stream full native manifest and 17GB output profile as inert data, compare embedded plans."""
import json,re,subprocess
from pathlib import Path
B3='/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
BASE=Path('/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-70db0868-primary-sources')
PLANROOT=BASE/'program-sources/source-built-mantle-programs/bootstrap/source-built'
PATHS=[BASE/'inputs/native-source-authority/native-source-closure.json',BASE/'source-profile.json']
PLAN_KEYS={f'bootstrap/source-built/mantle-stage{n}.unit-plan.json':(PLANROOT/f'mantle-stage{n}.unit-plan.json') for n in (1,2)}
EXPECTED={'native-source-closure.json':'87480463278bf0d94cf540f92c0f20f9dc3a8a7963a74b982a41bd9ba9523233','source-profile.json':'11c2b350fc9944d8bf71f2dea4ceb48b20eea869bf09922b20efa7168a92c534'}
def bytehash(data):return subprocess.run([B3,'--no-names'],input=data,capture_output=True,check=True).stdout.decode().strip()
def selected(path):
 count=0;found=[];bytes_seen=0;whole_hash=subprocess.Popen([B3,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 selected_hash=None; selected_bytes=0;selected_members={};selected_meta={}
 in_records=False;in_record=False;record_prefix=[];is_selected=False;child=False;candidate=[];target=None
 with path.open('rb',buffering=1024*1024) as stream:
  for line in stream:
   bytes_seen+=len(line);whole_hash.stdin.write(line)
   if not in_records:
    if line.strip()==b'"records": [':in_records=True
    continue
   if not in_record:
    if line==b'    {\n':
     in_record=True;record_prefix=[line];is_selected=False
    elif line.lstrip().startswith(b']'):in_records=False
    continue
   if is_selected:
    selected_hash.stdin.write(line);selected_bytes+=len(line)
    if line==b'        {\n':child=True;candidate=[line];target=None
    elif child:
     if b'"path": "bootstrap/source-built/mantle-stage' in line and b'.unit-plan.json"' in line:
      m=re.search(rb'"path": "([^"]+)"',line)
      if m and m.group(1).decode() in PLAN_KEYS:target=m.group(1).decode()
     if target:candidate.append(line)
     if line in (b'        },\n',b'        }\n'):
      if target:
       obj=json.loads(b''.join(candidate).rstrip().removesuffix(b','))
       raw=bytes.fromhex(obj['content_hex']);actual=PLAN_KEYS[target].read_bytes()
       selected_members[target]={'embedded_bytes':len(raw),'plan_bytes':len(actual),'embedded_blake3':bytehash(raw),'plan_blake3':bytehash(actual),'declared_blake3':obj['blake3'],'all_equal':raw==actual and bytehash(raw)==obj['blake3']}
      child=False;candidate=[];target=None
   else:
    record_prefix.append(line)
    if line.startswith(b'      "identity": '):
     identity=json.loads(line.partition(b': ')[2].rstrip().removesuffix(b','))
     if identity=='source-built-mantle-programs':
      if selected_hash is not None:raise ValueError('duplicate selected record')
      selected_hash=subprocess.Popen([B3,'--no-names'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
      for prefix in record_prefix:selected_hash.stdin.write(prefix);selected_bytes+=len(prefix)
      is_selected=True;count+=1
    if len(record_prefix)>30 and not is_selected:raise ValueError('record identity not within initial bounded fields')
   if line in (b'    },\n',b'    }\n'):
    if is_selected:
     digest,error=selected_hash.communicate(timeout=120)
     if selected_hash.returncode:raise RuntimeError(error.decode())
     found.append(digest.decode().strip());selected_meta={'record_raw_blake3':found[-1],'record_bytes':selected_bytes,'plan_members':selected_members}
    in_record=False;record_prefix=[];is_selected=False;selected_members={};selected_bytes=0
 digest,err=whole_hash.communicate(timeout=120)
 if whole_hash.returncode:raise RuntimeError(err.decode())
 if in_records or in_record or count!=1 or len(found)!=1:raise ValueError('incomplete, ambiguous source manifest stream')
 return {'path':str(path),'bytes':bytes_seen,'raw_blake3':digest.decode().strip(),'raw_expected':EXPECTED[path.name],'raw_matches':digest.decode().strip()==EXPECTED[path.name],**selected_meta}
results=[selected(p) for p in PATHS]
print(json.dumps({'full_streams':results,'embedded_record_equal':results[0]['record_raw_blake3']==results[1]['record_raw_blake3'],'all_plan_members_equal':all(all(item['all_equal'] for item in r['plan_members'].values()) and set(r['plan_members'])==set(PLAN_KEYS) for r in results)},separators=(',',':')))
