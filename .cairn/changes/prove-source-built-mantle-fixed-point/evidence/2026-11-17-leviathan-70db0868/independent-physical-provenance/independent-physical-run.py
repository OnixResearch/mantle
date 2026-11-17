import hashlib,json,pathlib,subprocess,sys
src=pathlib.Path(__file__).with_name('independent-physical-remote.py').read_text()
src=src.replace("need(program[ident]==known['program'][ident],'output program differs from augmented native closure')", "need(program[ident]=={k:v for k,v in known['program'][ident].items() if k!='physical'},'output program differs from augmented native closure')")
cmd=['ssh','-T','leviathan.cymric-daggertooth.ts.net','/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 -']
print('inert_remote_analyzer_sha256='+hashlib.sha256(src.encode()).hexdigest(),flush=True)
p=subprocess.run(cmd,input=src.encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=3200)
if p.returncode:
 print('REMOTE ANALYSIS FAILED rc='+str(p.returncode)+' stdout='+p.stdout.decode('utf-8','replace')[-2000:]+' stderr='+p.stderr.decode('utf-8','replace')[-4000:],file=sys.stderr,flush=True)
 raise SystemExit(p.returncode)
try:
 value=json.loads(p.stdout)
except Exception:
 print('REMOTE ANALYSIS MALFORMED '+p.stdout.decode('utf-8','replace')[-4000:],file=sys.stderr)
 raise
out=pathlib.Path(__file__).with_name('independent-physical-remote-result.json')
with out.open('x') as f:json.dump(value,f,indent=2,sort_keys=True);f.write('\n')
print(json.dumps({'result_scratch':str(out),'remote_records':value['closures'],'program_members':{k:len(v['physical']['source_members']) for k,v in value['programs'].items()},'preauthor_role_leaves':{k:v['entry_count'] for k,v in value['preauthor']['roles'].items()},'selected':value['selected_sources']['stages'],'source_copy':value['source_copy'],'aws_git':value['aws_git'],'initial_absence':value['initial_absence']},sort_keys=True),flush=True)
