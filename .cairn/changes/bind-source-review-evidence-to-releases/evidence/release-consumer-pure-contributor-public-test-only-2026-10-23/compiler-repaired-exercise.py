#!/usr/bin/env python3
"""PUBLIC TEST-ONLY: compiler-compatible continuation; preserve first compiler failure."""
import copy,json,subprocess,time
from pathlib import Path
RUN=Path(__file__).resolve().parent
PRIOR=Path('/home/brittonr/scratch/mantle-review-contributor-only-20261023-1792797197194973954')
INPUT=PRIOR/'TEST-ONLY-NOT-REAL-REVIEW'
ORIGINAL=Path('/home/brittonr/scratch/mantle-release-review-consumer-20261023-1792796950003385125')
DEPS=Path('/home/brittonr/.cargo-target/mantle-cairn-native-release-20261023/release/deps')
RLIB=DEPS/'libcrunch_release_core-f592d3d63163e0f3.rlib'
RUSTC=Path('/nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08/bin/rustc')
EVIDENCE=Path('/home/brittonr/git/OnixResearch/mantle/.cairn/changes/bind-source-review-evidence-to-releases/evidence')
PREFIX='release-consumer-pure-contributor-public-test-only-2026-10-23'
records=[]
def digest(data):return subprocess.check_output(['b3sum','--no-names'],input=data).decode().strip()
def save(path,data):path.write_text(json.dumps(data,indent=2)+'\n')
def capture(label,args):
    args=[str(a) for a in args];start=time.time()
    with (RUN/(label+'.stdout')).open('xb') as out,(RUN/(label+'.stderr')).open('xb') as err:r=subprocess.run(args,stdout=out,stderr=err,timeout=90)
    records.append({'label':label,'argv':args,'exit':r.returncode,'expected_exit':0,'cwd':str(RUN),'started_unix':start,'finished_unix':time.time(),'stdout':str(RUN/(label+'.stdout')),'stderr':str(RUN/(label+'.stderr'))});save(RUN/'commands.json',records)
    assert r.returncode==0,(label,r.returncode)
    return (RUN/(label+'.stdout')).read_bytes()
def pins():
    result={}
    for role,row in json.loads((ORIGINAL/'executable-pins-before.json').read_bytes()).items():
        actual=digest(Path(row['path']).read_bytes());assert actual==row['blake3'];result[role]={'path':row['path'],'blake3':actual}
    result['prebuilt-core-rlib']={'path':str(RLIB),'blake3':digest(RLIB.read_bytes())};result['throwaway-compiler']={'path':str(RUSTC),'blake3':digest(RUSTC.read_bytes())}
    return result
before=pins();save(RUN/'executable-pins-before.json',before)
capture('matching-rustc-version',[RUSTC,'--version'])
capture('compile-throwaway-contributor',[RUSTC,'--edition=2024',PRIOR/'contributor.rs','--extern','crunch_release_core='+str(RLIB),'-L','dependency='+str(DEPS),'-C','linker=/etc/profiles/per-user/brittonr/bin/cc','-o',RUN/'PUBLIC-TEST-ONLY-contributor'])
policy=json.loads((INPUT/'policy.json').read_bytes());archive=INPUT/'PUBLIC-TEST-ONLY-actual-source.tar'
raw=capture('exercise-pure-contributor',[RUN/'PUBLIC-TEST-ONLY-contributor',INPUT,digest(archive.read_bytes()),archive.stat().st_size])
after=pins();save(RUN/'executable-pins-after.json',after);assert before==after
expected=['selected-required-policy: SourceReviewVerification { status: Satisfied, valid: true','unselected-recorded-only: SourceReviewVerification { status: Recorded, valid: true','optional-absence: SourceReviewVerification { status: NotRequired, valid: true','required-missing-policy: SourceReviewVerification { status: Missing, valid: false','selected-policy-byte-mismatch: SourceReviewVerification { status: Stale, valid: false','tampered-signature: SourceReviewVerification { status: Invalid, valid: false']
assert all(text in raw.decode() for text in expected)
receipt=json.loads((INPUT/'acceptance.json').read_bytes())
allcommands=[]
for commands in [json.loads((PRIOR/'commands.json').read_bytes()),records]:
    for row in commands:
        row=copy.deepcopy(row)
        for stream in ['stdout','stderr']:
            path=Path(row[stream]);data=path.read_bytes();row[stream]={'path':str(path),'blake3':digest(data),'size_bytes':len(data),'text':data.decode(errors='replace')}
        allcommands.append(row)
summary={'schema':'mantle-source-review-pure-contributor-public-test-only-v1','test_only':True,'actual_release_create_success':False,'actual_release_verify_success':False,'pure_contributor_exercised':True,'ordinary_manifest_admissible':False,'execution_boundary':'Throwaway one-file executable linked to already built release-core rlib; zero Cargo/build-target writes; not Mantle public CLI acceptance','subject':policy['subject'],'archive_blake3':digest(archive.read_bytes()),'independent_receipt_blake3':receipt['receipt_blake3'],'independent_receipt_file_blake3':digest((INPUT/'acceptance.json').read_bytes()),'approved_distinct_public_test_review_keys':len(receipt['approved_key_blake3']),'release_signer_count':0,'build_witness_count':0,'cases':expected,'commands':allcommands,'executable_pins_before':before,'executable_pins_after':after,'non_claims':policy['non_claims'],'non_claims_additional':['No genuine human review, reviewer authority, actual release eligibility or full V5.','Ordinary release manifest is deliberately inadmissible and asserted rejected by canonical_release_evidence_manifest before contributor observation.','Pure contributor results do not bypass or satisfy missing genuine self-hosting CLI prerequisite.'],'prior_failed_attempt':{'root':str(PRIOR),'diagnostic':'E0514: existing core rlib compiled by rustc 1.96.0-nightly c75612477; initial driver used rustc 1.99.0-nightly dc3f85158','resolution':'Changed compiler to exact qualified development-shell compiler in fresh one-shot output root; no rebuilding dependencies or immutable executables.'},'scratch_roots':[str(PRIOR),str(RUN)]}
EXPORT=EVIDENCE/PREFIX;EXPORT.mkdir(exist_ok=False)
artifacts=[]
for source,name in [(INPUT/'policy.json','policy.json'),(INPUT/'claim.json','claim.json'),(INPUT/'attachment.json','attachment.json'),(INPUT/'acceptance.json','acceptance.json'),(INPUT/'tampered-attachment.json','tampered-attachment.json'),(PRIOR/'contributor.rs','contributor.rs'),(PRIOR/'exercise.py','producer-exercise.py'),(Path(__file__),'compiler-repaired-exercise.py'),(RUN/'exercise-pure-contributor.stdout','contributor.stdout'),(RUN/'exercise-pure-contributor.stderr','contributor.stderr'),(RUN/'PUBLIC-TEST-ONLY-contributor','PUBLIC-TEST-ONLY-contributor'),(RUN/'executable-pins-before.json','executable-pins-before.json'),(RUN/'executable-pins-after.json','executable-pins-after.json')]:
    dest=EXPORT/name;dest.write_bytes(source.read_bytes());artifacts.append({'path':str(dest.relative_to(EVIDENCE)),'source':str(source),'blake3':digest(dest.read_bytes()),'size_bytes':dest.stat().st_size})
save(EVIDENCE/(PREFIX+'.json'),summary);save(EVIDENCE/(PREFIX+'-artifacts.json'),{'schema':'mantle-source-review-pure-contributor-artifacts-v1','test_only':True,'artifacts':artifacts})
checks=artifacts.copy()
for name in [PREFIX+'.json',PREFIX+'-artifacts.json']:
    p=EVIDENCE/name;checks.append({'path':name,'blake3':digest(p.read_bytes()),'size_bytes':p.stat().st_size})
save(EVIDENCE/(PREFIX+'-integrity.json'),{'schema':'mantle-source-review-pure-contributor-integrity-v1','test_only':True,'checks':checks,'all_executable_pins_unchanged':before==after})
save(RUN/'report.json',summary)
print(json.dumps({'pure_contributor_exercised':True,'cases':6,'pins_unchanged':before==after,'evidence':str(EVIDENCE/(PREFIX+'.json'))}))
