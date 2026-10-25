#!/usr/bin/env python3
"""Fresh continuation: preserve failed first harness; no unchanged failure retries."""
import copy,json,subprocess,time
from pathlib import Path
RUN=Path(__file__).resolve().parent
OLD=Path('/home/brittonr/scratch/mantle-release-review-consumer-20261023-1792796950003385125')
INPUT=OLD/'TEST-ONLY-NOT-REAL-REVIEW'
COHORT=Path('/home/brittonr/git/OnixResearch/mantle/target/cairn-finish-20261023/wait-request-repaired-immutable-release-cohort')
MANTLE=COHORT/'mantle'
VALENCE=Path('/nix/store/ql5djfqm579pi0vzhxpj9yamqypnkg0k-valence-0.1.0/bin/valence')
EVIDENCE=Path('/home/brittonr/git/OnixResearch/mantle/.cairn/changes/bind-source-review-evidence-to-releases/evidence')
PREFIX='release-consumer-public-test-only-2026-10-23'
EXPORT=EVIDENCE/PREFIX; EXPORT.mkdir(mode=0o755,exist_ok=False)
records=[]
def digest(data):return subprocess.check_output(['b3sum','--no-names'],input=data).decode().strip()
def save(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def capture(label,args,expected):
    args=[str(a) for a in args]; start=time.time()
    with (RUN/(label+'.stdout')).open('xb') as out,(RUN/(label+'.stderr')).open('xb') as err:
        r=subprocess.run(args,stdout=out,stderr=err,timeout=90)
    row={'label':label,'argv':args,'cwd':str(RUN),'exit':r.returncode,'expected_exit':expected,'started_unix':start,'finished_unix':time.time(),'stdout':label+'.stdout','stderr':label+'.stderr'}
    records.append(row); save(RUN/'commands.json',records)
    assert r.returncode==expected,(label,r.returncode)
    return (RUN/(label+'.stdout')).read_bytes()
def pins():
    result={}
    for role,row in json.loads((COHORT/'ready-executable-pins.json').read_bytes())['roles'].items():
        actual=digest(Path(row['path']).read_bytes()); assert actual==row['blake3']; result[role]={'path':row['path'],'blake3':actual}
    before=json.loads((OLD/'executable-pins-before.json').read_bytes())
    for role in ['cairn','valence','openssl']:
        row=before[role]; actual=digest(Path(row['path']).read_bytes());assert actual==row['blake3'];result[role]={'path':row['path'],'blake3':actual}
    return result
save(RUN/'executable-pins-before.json',pins())
# Explicitly incomplete ordinary bundle: actual input bytes, no successful release declaration,
# no invented proof, no fallback acceptance and no rewritten manifest after observation.
bundle=RUN/'PUBLIC-TEST-ONLY-INCOMPLETE-NOT-A-RELEASE';bundle.mkdir()
rows=[]
for source,role,schema in [(INPUT/'attachment.json','cairn-source-review-attachment','cairn.source-review-attachment.v1'),(INPUT/'policy.json','cairn-source-review-policy','cairn.source-review-verification-policy.v1'),(OLD/'receipts/acceptance.json','valence-source-review-acceptance','valence.source-review-acceptance.v1')]:
    dest=bundle/source.name;dest.write_bytes(source.read_bytes())
    rows.append({'role':role,'schema':schema,'relative_path':dest.name,'digest_blake3':digest(dest.read_bytes()),'claim_scope':'source-review-identity-linkage-only','non_claims':json.loads((INPUT/'policy.json').read_bytes())['non_claims']})
archive=INPUT/'PUBLIC-TEST-ONLY-actual-source.tar';(bundle/archive.name).write_bytes(archive.read_bytes())
save(bundle/'manifest.json',{'schema':'mantle-release-evidence-v1','release_id':'PUBLIC-TEST-ONLY-INCOMPLETE-NOT-A-RELEASE','source_archive':{'kind':'file','relative_path':archive.name,'size_bytes':archive.stat().st_size,'digest_blake3':digest(archive.read_bytes())},'external_evidence':rows})
capture('release-verify-incomplete-required-policy',[MANTLE,'--json','release','verify',bundle,'--source-review-policy',INPUT/'policy.json','--require-source-review'],3)
# Relevant independent consumer rejection for these exact source inputs; not a Mantle contributor claim.
negative=RUN/'TEST-ONLY-tampered-review';negative.mkdir();receipts=RUN/'negative-receipts';receipts.mkdir()
(negative/'policy.json').write_bytes((INPUT/'policy.json').read_bytes())
tampered=json.loads((INPUT/'attachment.json').read_bytes());sig=tampered['reviews'][0]['signature_hex'];tampered['reviews'][0]['signature_hex']=('0' if sig[0]!='0' else '1')+sig[1:];save(negative/'attachment.json',tampered)
raw=capture('valence-tampered-signature',[VALENCE,'source-review','verify','--artifact-root',negative,'--attachment','attachment.json','--policy','policy.json','--output-root',receipts,'--receipt','tampered-signature.json'],1)
negative_receipt=json.loads(raw);assert negative_receipt['outcome']=='invalid' and negative_receipt['approved_key_blake3']==[]
after=pins();save(RUN/'executable-pins-after.json',after)
before=json.loads((OLD/'executable-pins-before.json').read_bytes());assert before==after
oldrecords=json.loads((OLD/'commands.json').read_bytes())
assert oldrecords[-1]['exit']==3 and oldrecords[-1]['expected_exit']==1
# Retain harness expectation error exactly; it was not product behavior or a product bug.
allcommands=[]
for root,commands in [(OLD,oldrecords),(RUN,records)]:
    for row in commands:
        row=copy.deepcopy(row)
        for stream in ['stdout','stderr']:
            path=root/row[stream];data=path.read_bytes();row[stream]={'path':str(path),'blake3':digest(data),'size_bytes':len(data),'text':data.decode(errors='replace')}
        allcommands.append(row)
acceptance=json.loads((OLD/'receipts/acceptance.json').read_bytes())
non_claims=json.loads((INPUT/'policy.json').read_bytes())['non_claims']
summary={'schema':'mantle-release-source-review-public-test-only-consumer-v1','test_only':True,'real_human_approval':False,'actual_release_created':False,'mantle_source_review_contributor_reached':False,'scratch_roots':[str(OLD),str(RUN)],'archive':{'path':str(archive),'blake3':digest(archive.read_bytes()),'size_bytes':archive.stat().st_size,'construction':'GNU tar of actual tiny indexed local source and checksum-checked local vendor fixture; not the unavailable Mantle release archive'},'source_subject':json.loads((INPUT/'policy.json').read_bytes())['subject'],'independent_receipt':{'path':str(OLD/'receipts/acceptance.json'),'file_blake3':digest((OLD/'receipts/acceptance.json').read_bytes()),'receipt_blake3':acceptance['receipt_blake3'],'outcome':acceptance['outcome'],'approved_distinct_review_keys':len(acceptance['approved_key_blake3'])},'negative_independent_receipt':negative_receipt,'release_signer_count':0,'build_witness_count':0,'executable_pins_before':before,'executable_pins_after':after,'commands':allcommands,'non_claims':non_claims,'non_claims_additional':['No successful Mantle release create or verify.','No actual Mantle source-review contributor acceptance: ordinary manifest/proof validation precedes that contributor.','No full V5, source-built cold-v2 proof, reviewer authority, genuine human approval, or actual release eligibility.','Two PUBLIC TEST-ONLY review keys are separate from zero release signers and zero build witnesses.'],'blocked_acceptance_cases':['selected required-policy success','unselected recorded-only behavior','optional absence','Mantle required missing/invalid policy or tampered-byte contributor rejection'],'prerequisite':{'missing':'Genuine retained mantle-self-hosting-proof-v2 full proof bundle and matching stage2 binary','observed_release_create_exit':3,'observed_diagnostic':(OLD/'release-create-missing-genuine-proof.stderr').read_text().strip(),'retained_locations_checked':['docs/operator-proof-guide.md:75-169','target/self-hosting-proof (absent)','target root directory; no self-hosting proof root','named self/proof scratch roots and /tmp roots; no full Mantle proof found'],'code_barrier':['src/release_cmd.rs:1269 verifies ordinary bundle before source-review evaluation at :1272','src/release_evidence.rs:1075-1076 measures artifacts and verifies proof linkage first','crates/crunch-release-core/src/manifest.rs:2716-2727 requires all three fixed-point booleans true'],'incomplete_verify_diagnostic':(RUN/'release-verify-incomplete-required-policy.stderr').read_text().strip()},'harness_observation':{'preserved_failed_attempt':str(OLD),'error':'Expected exit 1 for missing proof but actual public CLI correctly emitted exit 3; no product bug and no unchanged-failure retry.','fresh_continuation':str(RUN)}}
retained=[archive,INPUT/'attachment.json',INPUT/'policy.json',OLD/'receipts/acceptance.json',receipts/'tampered-signature.json',OLD/'executable-pins-before.json',RUN/'executable-pins-after.json',OLD/'exercise.py',RUN/'finish.py',bundle/'manifest.json']
artifacts=[]
for p in retained:
    name=('first-exercise.py' if p==OLD/'exercise.py' else 'incomplete-bundle-manifest.json' if p==bundle/'manifest.json' else p.name)
    out=EXPORT/name;out.write_bytes(p.read_bytes());artifacts.append({'path':str(out.relative_to(EVIDENCE)),'source':str(p),'blake3':digest(out.read_bytes()),'size_bytes':out.stat().st_size})
save(EVIDENCE/(PREFIX+'.json'),summary)
save(EVIDENCE/(PREFIX+'-artifacts.json'),{'schema':'mantle-release-consumer-public-test-only-artifacts-v1','test_only':True,'artifacts':artifacts})
checks=[]
for p in sorted(EXPORT.iterdir()):checks.append({'path':str(p.relative_to(EVIDENCE)),'blake3':digest(p.read_bytes()),'size_bytes':p.stat().st_size})
for name in [PREFIX+'.json',PREFIX+'-artifacts.json']:
    p=EVIDENCE/name;checks.append({'path':name,'blake3':digest(p.read_bytes()),'size_bytes':p.stat().st_size})
save(EVIDENCE/(PREFIX+'-integrity.json'),{'schema':'mantle-release-consumer-public-test-only-integrity-v1','test_only':True,'checks':checks,'all_executable_pins_unchanged':before==after})
save(RUN/'report.json',summary)
print(json.dumps({'status':'reachable-cli-work-complete-with-explicit-prerequisite-blocker','mantle_contributor_reached':False,'evidence':str(EVIDENCE/(PREFIX+'.json')),'independent_reviews':len(acceptance['approved_key_blake3']),'tampered_signature_outcome':negative_receipt['outcome'],'pins_unchanged':before==after}))
