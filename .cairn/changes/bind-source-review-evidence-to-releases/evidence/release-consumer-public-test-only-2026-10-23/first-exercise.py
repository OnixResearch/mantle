#!/usr/bin/env python3
"""PUBLIC TEST-ONLY: actual source files, archive, current review producer/receiver.
No human review, release approval, self-hosting proof or release eligibility.
"""
import copy, hashlib, json, os, subprocess, time
from pathlib import Path
RUN=Path(__file__).resolve().parent
INPUT=RUN/'TEST-ONLY-NOT-REAL-REVIEW'; INPUT.mkdir(mode=0o700)
OUT=RUN/'receipts'; OUT.mkdir(mode=0o700)
REPO=RUN/'tiny-source'; REPO.mkdir(mode=0o700)
MANTLE=Path('/home/brittonr/git/OnixResearch/mantle/target/cairn-finish-20261023/wait-request-repaired-immutable-release-cohort/mantle')
CAIRN=Path('/nix/store/3pcb3kqllcyy38ffwid4s7mw19hh7bcb-cairn-0.1.0/bin/cairn')
VALENCE=Path('/nix/store/ql5djfqm579pi0vzhxpj9yamqypnkg0k-valence-0.1.0/bin/valence')
OPENSSL=Path('/nix/store/gvglnxscmvx4mh9my0cy0xgi0l1z6l9s-openssl-3.5.9-bin/bin/openssl')
FIXTURE=Path('/home/brittonr/scratch/cairn-source-review/fixtures/source-review/TEST-ONLY-keys')
PINFILE=MANTLE.parent/'ready-executable-pins.json'
records=[]
def digest(data): return subprocess.check_output(['b3sum','--no-names'],input=data).decode().strip()
def save_at(path,value): path.write_text(json.dumps(value,indent=2)+'\n'); return path
def save(name,value): return save_at(INPUT/name,value)
def capture(label,args,expected=0,cwd=None):
    args=[str(a) for a in args]; start=time.time()
    with (RUN/(label+'.stdout')).open('xb') as out,(RUN/(label+'.stderr')).open('xb') as err:
        r=subprocess.run(args,stdout=out,stderr=err,cwd=cwd,timeout=90)
    records.append({'label':label,'argv':args,'cwd':str(cwd or RUN),'exit':r.returncode,'expected_exit':expected,'started_unix':start,'finished_unix':time.time(),'stdout':label+'.stdout','stderr':label+'.stderr'})
    save_at(RUN/'commands.json',records)
    if r.returncode != expected: raise RuntimeError('Unexpected exit: '+label)
    return (RUN/(label+'.stdout')).read_bytes()
def pins():
    allpins=json.loads(PINFILE.read_bytes())['roles']; result={}
    for role,row in allpins.items():
        actual=digest(Path(row['path']).read_bytes()); assert actual==row['blake3']
        result[role]={'path':row['path'],'blake3':actual}
    for role,path,expected in [('cairn',CAIRN,'8ba55fe02cdf478210a472fdc8232791b33773609b8a82803e0609b5eb4ca743'),('valence',VALENCE,'b5d95a18fabfb5a109543503829aa63c73e1e85f16e5cb556fc238628285917b')]:
        actual=digest(path.read_bytes()); assert actual==expected; result[role]={'path':str(path),'blake3':actual}
    result['openssl']={'path':str(OPENSSL),'blake3':digest(OPENSSL.read_bytes())}
    return result
save_at(RUN/'executable-pins-before.json',pins())
capture('release-create-help',[MANTLE,'release','create','--help'])
capture('release-verify-help',[MANTLE,'release','verify','--help'])
files={'.gitignore':'vendor-deps/\n','.cargo/vendor-config.toml':'directory = "vendor-deps"\n','Cargo.toml':'[package]\nname="PUBLIC-TEST-ONLY-demo"\nversion="0.1.0"\nedition="2024"\n','Cargo.lock':'[[package]]\nname = "dep"\nversion = "0.1.0"\nsource = "git+https://example.invalid/dep.git#0123456789abcdef"\n','src/main.rs':'fn main() { println!("PUBLIC TEST ONLY source input; not built or reviewed"); }\n','vendor-deps/dep/Cargo.toml':'[package]\nname="dep"\nversion="0.1.0"\n','vendor-deps/dep/lib.rs':'pub fn public_test_only_dep() {}\n'}
checksums={name.split('vendor-deps/dep/')[1]:hashlib.sha256(data.encode()).hexdigest() for name,data in files.items() if name.startswith('vendor-deps/dep/')}
files['vendor-deps/dep/.cargo-checksum.json']=json.dumps({'files':checksums,'package':None})+'\n'
for name,data in files.items():
    p=REPO/name; p.parent.mkdir(parents=True,exist_ok=True); p.write_text(data)
capture('tiny-git-init',['git','init',REPO])
capture('tiny-git-stage',['git','add','.'],cwd=REPO)
archive=INPUT/'PUBLIC-TEST-ONLY-actual-source.tar'
capture('archive-actual-source',['tar','--format=gnu','--sort=name','--mtime=@0','--owner=0','--group=0','--numeric-owner','--exclude=.git','-cf',archive,'.'],cwd=REPO)
archive_bytes=archive.read_bytes()
capture('archive-list',['tar','-tvf',archive])
non_claims=sorted('source-review-does-not-prove-'+s for s in ['source-correctness','review-quality','build-correctness','reproducibility','witness-independence','deployment-safety','release-eligibility'])
keys=[]
for who in ['reviewer-a','reviewer-b']:
    seed_file=FIXTURE/(who+'.seed.hex'); seed=bytes.fromhex(seed_file.read_text().strip())
    key=INPUT/(who+'.PUBLIC-TEST-ONLY.der'); key.write_bytes(bytes.fromhex('302e020100300506032b657004220420')+seed); key.chmod(0o600)
    public_der=capture(who+'-derive-public',[OPENSSL,'pkey','-inform','DER','-in',key,'-pubout','-outform','DER'])
    assert len(public_der)==44 and public_der[:12].hex()=='302a300506032b6570032100'
    public=public_der[12:]; assert public.hex()==(FIXTURE/(who+'.public.hex')).read_text().strip()
    keys.append({'who':who,'key':key,'public':public,'identity':digest(public),'seed_source':str(seed_file),'seed_source_blake3':digest(seed_file.read_bytes())})
label_hash=lambda text:digest(('PUBLIC TEST ONLY '+text).encode())
subject={'schema':'cairn.source-review-subject.v1','source_archive_blake3':digest(archive_bytes),'source_revision':'PUBLIC-TEST-ONLY-LOCAL-INDEX-NOT-IMMUTABLE-REVISION','source_tree_blake3':None,'snapshot_profile':'mantle-release-source','snapshot_profile_version':1,'projection':'source'}
root={'claim_identity':label_hash('synthetic exact claim identity'),'subject_root':label_hash('synthetic exact current claim-DAG root'),'revision':1}
context={'profile':'test-only-currentness.v1','algorithm':'blake3','digest_hex':label_hash('synthetic exact currentness')}
review_policy=label_hash('synthetic exact reviewed policy')
policy={'schema':'cairn.source-review-verification-policy.v1','subject':subject,'claim_root':root,'review_policy_blake3':review_policy,'authentication_policy':{'schema':'artifact-auth.policy.v1','profile_id':'cairn.source-review.v1','threshold':2,'trusted_keys':[{'producer_id':k['who']+'-PUBLIC-TEST-ONLY','key_id':'PUBLIC-TEST-ONLY-NOT-REAL-APPROVAL','key_identity':{'profile':'ed25519-public-key.v1','algorithm':'blake3','digest_hex':k['identity']},'allowed_purposes':['source-review-approval'],'generation':1,'currentness':'current','currentness_ref':context} for k in keys]},'verifier_context':context,'author_key_blake3':[],'exclude_authors':True,'required_approvals':2,'non_claims':non_claims}
save('policy.json',policy)
save('claim.json',{'schema':'cairn.source-review-claim.v1','subject':subject,'claim_root':root,'review_policy_blake3':review_policy,'disposition':'approved','non_claims':non_claims})
for k in keys:
    who=k['who']; args=[CAIRN,'source-review','signing-request','--root',INPUT,'--claim','claim.json','--policy','policy.json','--producer',who+'-PUBLIC-TEST-ONLY','--key-id','PUBLIC-TEST-ONLY-NOT-REAL-APPROVAL','--generation','1','--public-key-hex',k['public'].hex()]
    req=capture(who+'-signing-request',args); (INPUT/(who+'-request.json')).write_bytes(req)
    raw=capture(who+'-raw',args+['--raw']); raw_path=INPUT/(who+'-statement.bin'); raw_path.write_bytes(raw)
    parsed=json.loads(req); assert raw==bytes.fromhex(parsed['canonical_statement_hex']) and digest(raw)==parsed['statement_blake3']
    sig=INPUT/(who+'-signature.bin'); capture(who+'-external-sign',[OPENSSL,'pkeyutl','-sign','-inkey',k['key'],'-keyform','DER','-rawin','-in',raw_path,'-out',sig]); assert len(sig.read_bytes())==64
    cmd=[CAIRN,'source-review','assemble','--root',INPUT,'--request',who+'-request.json','--policy','policy.json','--signature-hex',sig.read_bytes().hex()]
    if who=='reviewer-b': cmd+=['--include','row-a.json']
    attachment=capture(who+'-assemble',cmd)
    if who=='reviewer-a': save('row-a.json',json.loads(attachment)['reviews'][0])
    else: (INPUT/'attachment.json').write_bytes(attachment)
capture('cairn-verify',[CAIRN,'source-review','verify','--root',INPUT,'--attachment','attachment.json','--policy','policy.json'])
receipt=capture('valence-verify',[VALENCE,'source-review','verify','--artifact-root',INPUT,'--attachment','attachment.json','--policy','policy.json','--output-root',OUT,'--receipt','acceptance.json'])
r=json.loads(receipt); assert r['outcome']=='accepted_manual_review' and len(r['approved_key_blake3'])==2
assert r['attachment_artifact_blake3']==digest((INPUT/'attachment.json').read_bytes()) and r['policy_artifact_blake3']==digest((INPUT/'policy.json').read_bytes())
# Actual CLI, truthful missing-proof precondition; do not forge self-hosting success.
binary=RUN/'PUBLIC-TEST-ONLY-output.sh'; binary.write_text('#!/bin/sh\nprintf "%s\\n" "PUBLIC TEST ONLY actual tiny output; no build correctness claim"\n'); binary.chmod(0o755)
capture('tiny-output-run',[binary])
cmd=[MANTLE,'--json','release','create','--release-id','PUBLIC-TEST-ONLY-source-review-consumer','--bundle-dir',RUN/'bundle-with-review','--binary',binary,'--proof-bundle',RUN/'UNAVAILABLE-GENUINE-SELF-HOSTING-PROOF']
for path,role,schema in [(INPUT/'attachment.json','cairn-source-review-attachment','cairn.source-review-attachment.v1'),(INPUT/'policy.json','cairn-source-review-policy','cairn.source-review-verification-policy.v1'),(OUT/'acceptance.json','valence-source-review-acceptance','valence.source-review-acceptance.v1')]:
    cmd+=['--external-evidence',path,'--external-evidence-role',role,'--external-evidence-schema',schema,'--external-evidence-claim-scope','source-review-identity-linkage-only']
for claim in non_claims: cmd+=['--external-evidence-non-claim',claim]
capture('release-create-missing-genuine-proof',cmd,1,cwd=REPO)
assert not (RUN/'bundle-with-review').exists()
save_at(RUN/'executable-pins-after.json',pins())
save_at(RUN/'reachable-report.json',{'schema':'mantle-release-source-review-public-test-only-consumer-v1','test_only':True,'real_human_approval':False,'release_created':False,'archive':{'path':str(archive),'blake3':digest(archive_bytes),'size_bytes':len(archive_bytes)},'subject':subject,'claim_root':root,'review_policy_blake3':review_policy,'independent_receipt':{'path':str(OUT/'acceptance.json'),'file_blake3':digest((OUT/'acceptance.json').read_bytes()),'receipt_blake3':r['receipt_blake3'],'outcome':r['outcome'],'approved_distinct_review_keys':len(r['approved_key_blake3'])},'release_signer_count':0,'build_witness_count':0,'commands':records,'non_claims':non_claims,'blocked_cli_cases':['required-policy success','unselected recorded-only','optional absence','required missing/invalid policy or tampered-byte rejection'],'blocker':'Actual release create requires a genuine full self-hosting proof bundle and matching stage2 binary. No fabricated fixed_point success or immutable executable mutation permitted.'})
print(json.dumps({'status':'reachable-work-complete-genuine-proof-missing','test_only':True,'commands':len(records)}))
