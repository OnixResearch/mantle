#!/usr/bin/env python3
"""PUBLIC TEST-ONLY contributor-only observation, never a release-create result."""
import copy,json,subprocess,time
from pathlib import Path
RUN=Path(__file__).resolve().parent
OLD=Path('/home/brittonr/scratch/mantle-release-review-consumer-20261023-1792796950003385125')
INPUT=RUN/'TEST-ONLY-NOT-REAL-REVIEW';INPUT.mkdir(mode=0o700)
OUT=RUN/'receipts';OUT.mkdir()
COHORT=Path('/home/brittonr/git/OnixResearch/mantle/target/cairn-finish-20261023/wait-request-repaired-immutable-release-cohort')
DEPS=Path('/home/brittonr/.cargo-target/mantle-cairn-native-release-20261023/release/deps')
RLIB=DEPS/'libcrunch_release_core-f592d3d63163e0f3.rlib'
RUSTC=Path('/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc')
CAIRN=Path('/nix/store/3pcb3kqllcyy38ffwid4s7mw19hh7bcb-cairn-0.1.0/bin/cairn')
VALENCE=Path('/nix/store/ql5djfqm579pi0vzhxpj9yamqypnkg0k-valence-0.1.0/bin/valence')
OPENSSL=Path('/nix/store/gvglnxscmvx4mh9my0cy0xgi0l1z6l9s-openssl-3.5.9-bin/bin/openssl')
records=[]
def digest(data):return subprocess.check_output(['b3sum','--no-names'],input=data).decode().strip()
def save(path,data):path.write_text(json.dumps(data,indent=2)+'\n')
def pins():
    result={}
    for role,row in json.loads((OLD/'executable-pins-before.json').read_bytes()).items():
        actual=digest(Path(row['path']).read_bytes());assert actual==row['blake3'];result[role]={'path':row['path'],'blake3':actual}
    result['prebuilt-core-rlib']={'path':str(RLIB),'blake3':digest(RLIB.read_bytes())}
    result['throwaway-compiler']={'path':str(RUSTC),'blake3':digest(RUSTC.read_bytes())}
    return result
def capture(label,args,expected=0):
    args=[str(a) for a in args];start=time.time()
    with (RUN/(label+'.stdout')).open('xb') as out,(RUN/(label+'.stderr')).open('xb') as err:r=subprocess.run(args,stdout=out,stderr=err,timeout=90)
    records.append({'label':label,'argv':args,'exit':r.returncode,'expected_exit':expected,'cwd':str(RUN),'started_unix':start,'finished_unix':time.time(),'stdout':str(RUN/(label+'.stdout')),'stderr':str(RUN/(label+'.stderr'))});save(RUN/'commands.json',records)
    assert r.returncode==expected,(label,r.returncode)
    return (RUN/(label+'.stdout')).read_bytes()
before=pins();save(RUN/'executable-pins-before.json',before)
old_input=OLD/'TEST-ONLY-NOT-REAL-REVIEW'
policy=json.loads((old_input/'policy.json').read_bytes());policy['subject']['source_revision']=None
claim=json.loads((old_input/'claim.json').read_bytes());claim['subject']['source_revision']=None
save(INPUT/'policy.json',policy);save(INPUT/'claim.json',claim)
archive=INPUT/'PUBLIC-TEST-ONLY-actual-source.tar';archive.write_bytes((old_input/archive.name).read_bytes());assert digest(archive.read_bytes())==policy['subject']['source_archive_blake3']
for who in ['reviewer-a','reviewer-b']:
    public=(OLD/(who+'-derive-public.stdout')).read_bytes()[12:]
    args=[CAIRN,'source-review','signing-request','--root',INPUT,'--claim','claim.json','--policy','policy.json','--producer',who+'-PUBLIC-TEST-ONLY','--key-id','PUBLIC-TEST-ONLY-NOT-REAL-APPROVAL','--generation','1','--public-key-hex',public.hex()]
    req=capture(who+'-request',args);(INPUT/(who+'-request.json')).write_bytes(req)
    raw=capture(who+'-raw',args+['--raw']);rawfile=INPUT/(who+'-statement.bin');rawfile.write_bytes(raw);request=json.loads(req);assert raw==bytes.fromhex(request['canonical_statement_hex']) and digest(raw)==request['statement_blake3']
    sig=INPUT/(who+'-signature.bin');capture(who+'-sign',[OPENSSL,'pkeyutl','-sign','-inkey',old_input/(who+'.PUBLIC-TEST-ONLY.der'),'-keyform','DER','-rawin','-in',rawfile,'-out',sig]);assert len(sig.read_bytes())==64
    cmd=[CAIRN,'source-review','assemble','--root',INPUT,'--request',who+'-request.json','--policy','policy.json','--signature-hex',sig.read_bytes().hex()]
    if who=='reviewer-b':cmd+=['--include','row-a.json']
    raw_attachment=capture(who+'-assemble',cmd)
    if who=='reviewer-a':save(INPUT/'row-a.json',json.loads(raw_attachment)['reviews'][0])
    else:(INPUT/'attachment.json').write_bytes(raw_attachment)
capture('cairn-verify',[CAIRN,'source-review','verify','--root',INPUT,'--attachment','attachment.json','--policy','policy.json'])
raw_receipt=capture('valence-verify',[VALENCE,'source-review','verify','--artifact-root',INPUT,'--attachment','attachment.json','--policy','policy.json','--output-root',OUT,'--receipt','acceptance.json'])
receipt=json.loads(raw_receipt);assert receipt['outcome']=='accepted_manual_review' and len(receipt['approved_key_blake3'])==2
(INPUT/'acceptance.json').write_bytes((OUT/'acceptance.json').read_bytes())
tampered=json.loads((INPUT/'attachment.json').read_bytes());sig=tampered['reviews'][0]['signature_hex'];tampered['reviews'][0]['signature_hex']=('0' if sig[0]!='0' else '1')+sig[1:];save(INPUT/'tampered-attachment.json',tampered)
capture('compile-throwaway-contributor',[RUSTC,'--edition=2024',RUN/'contributor.rs','--extern','crunch_release_core='+str(RLIB),'-L','dependency='+str(DEPS),'-C','linker=/etc/profiles/per-user/brittonr/bin/cc','-o',RUN/'PUBLIC-TEST-ONLY-contributor'])
capture('exercise-pure-contributor',[RUN/'PUBLIC-TEST-ONLY-contributor',INPUT,policy['subject']['source_archive_blake3'],archive.stat().st_size])
after=pins();save(RUN/'executable-pins-after.json',after);assert before==after
summary={'schema':'mantle-source-review-pure-contributor-public-test-only-v1','test_only':True,'actual_release_create_success':False,'actual_release_verify_success':False,'pure_contributor_exercised':True,'ordinary_manifest_admissible':False,'execution_boundary':'Throwaway one-file executable linked to already built release-core rlib; zero Cargo/build-target writes; this is not Mantle public CLI acceptance','subject':policy['subject'],'archive_blake3':digest(archive.read_bytes()),'independent_receipt_blake3':receipt['receipt_blake3'],'independent_receipt_file_blake3':digest((OUT/'acceptance.json').read_bytes()),'approved_distinct_public_test_review_keys':len(receipt['approved_key_blake3']),'release_signer_count':0,'build_witness_count':0,'cases':['selected-required-policy: Satisfied valid=true approvals=2','unselected-recorded-only: Recorded valid=true approvals=2','optional-absence: NotRequired valid=true approvals=0','required-missing-policy: Missing valid=false approvals=0','selected-policy-byte-mismatch: Stale valid=false approvals=0','tampered-signature: Invalid valid=false approvals=0'],'commands':records,'executable_pins_before':before,'executable_pins_after':after,'non_claims':policy['non_claims'],'non_claims_additional':['No genuine human review, reviewer authority, actual release eligibility or full V5.','The ordinary release manifest is deliberately inadmissible and asserted rejected by canonical_release_evidence_manifest before contributor observation.','Pure contributor results do not bypass or satisfy the missing genuine self-hosting CLI prerequisite.']}
save(RUN/'report.json',summary)
print(json.dumps({'pure_contributor_exercised':True,'cases':6,'pins_unchanged':before==after,'report':str(RUN/'report.json')}))
