# Aspen1 Git-source witness replay

Date: 2026-06-29
Change: `git-source-witness-replay`

## Result

Aspen1 strict Git-source witness replay succeeded and final local release verification is `quorum-satisfied`.

Summary:

- source remote: `file:///home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/git-origin/mantle.git`
- source commit: `7ace5e14c39bf56f4ab7a2e8b354a0de61c00c44`
- source ref policy: `refs/heads/main`
- generated source archive digest: `6d1e366bbb94f1809c814fba9db29efbe3c2b33c02e2862ed2950a86d000c08a`
- published binary digest: `37768afa1d949dc27523954884abb82056e7bda06386a7f15d3b893d7e71a3b4`
- publisher trusted key: `crunch-britton-desktop-1:yG3NVpAYrBe965g8GsxwV0V+C6xnzFJgSgqXoWeZnrI=`
- Aspen1 witness trusted key: `crunch-aspen1-1:yCUo8ySXwGDjGoa6qQaOG0SeuaxJRj5r3CP5WGvu+QI=`
- policy status: `satisfied`
- independent agreement status: `satisfied`
- counted witnesses: `1`

## Local release/request preparation

Local artifacts:

~~~text
release bundle: /home/brittonr/git/mantle/target/release-evidence/git-source-witness-replay-2026-06-29
verification dir: /home/brittonr/git/mantle/target/release-verification/git-source-witness-replay-2026-06-29
witness request: /home/brittonr/git/mantle/target/release-witness-requests/git-source-witness-replay-2026-06-29
transfer bundle: /home/brittonr/git/mantle/target/aspen-transfer/git-source-witness-replay-2026-06-29
bare Git origin: /home/brittonr/git/mantle/target/aspen-transfer/git-source-witness-replay-2026-06-29/git-origin/mantle.git
release source clone used for archive trust root: /home/brittonr/git/mantle/target/git-source-witness-replay/git-source-witness-replay-2026-06-29/release-work
~~~

Manifest source acquisition:

~~~json
{"kind":"git","url":"file:///home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/git-origin/mantle.git","digest_blake3":"6d1e366bbb94f1809c814fba9db29efbe3c2b33c02e2862ed2950a86d000c08a","commit":"7ace5e14c39bf56f4ab7a2e8b354a0de61c00c44","reference":"refs/heads/main","archive_profile":"mantle-release-source","archive_version":"v1"}
~~~

Command summary:

~~~text
mantle release create --git-source-url file:///home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/git-origin/mantle.git --git-source-commit 7ace5e14c39bf56f4ab7a2e8b354a0de61c00c44 --git-source-ref refs/heads/main
mantle release attest target/release-evidence/git-source-witness-replay-2026-06-29 --verification-dir target/release-verification/git-source-witness-replay-2026-06-29
mantle attest policy-init target/release-verification/git-source-witness-replay-2026-06-29 --profile single-witness --trusted-release-signer crunch-britton-desktop-1 --trusted-witness-identity aspen1 --force
mantle release witness-export target/release-evidence/git-source-witness-replay-2026-06-29 --verification-dir target/release-verification/git-source-witness-replay-2026-06-29 --request-dir target/release-witness-requests/git-source-witness-replay-2026-06-29
scp -r target/aspen-transfer/git-source-witness-replay-2026-06-29 aspen1.local:/home/brittonr/mantle-witness/
~~~

## First strict-replay negative check

A first bundle prepared from the original live filesystem failed strict Git-source replay because host vendored file permissions (for example `vendor-deps/fnv/*` as `0640`) did not match Git checkout-normalized permissions (`0644`). The final bundle above was recreated from a fresh clone of the same bare Git origin, so the source archive trust-root digest is derived from the same checkout semantics used by Aspen1 replay.

Observed failing strict-replay error:

~~~text
Git source archive digest mismatch: expected e4c66beadaa3abc26aac96907c383da6fa7ebf9439885d9bdddafcc73953375f, generated 6d1e366bbb94f1809c814fba9db29efbe3c2b33c02e2862ed2950a86d000c08a from file:///home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/git-origin/mantle.git at commit 7ace5e14c39bf56f4ab7a2e8b354a0de61c00c44
~~~

## Aspen1 preflight

Command evidence: pueue task 165.

~~~text
witness rebuild preflight OK: /home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/request
release id: git-source-witness-replay-2026-06-29
scratch root: /home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/check-work
require Git source: true
source acquisition kind: git
source acquisition URL: file:///home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/git-origin/mantle.git
source acquisition commit: 7ace5e14c39bf56f4ab7a2e8b354a0de61c00c44
check only: no rebuild executed, no witness sidecars written
~~~

## Aspen1 full strict replay

Command evidence: pueue task 167.

~~~text
witness attestation: /home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/full-work/release-verification/git-source-witness-replay-2026-06-29/witnesses/aspen1.json
signature: /home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/full-work/release-verification/git-source-witness-replay-2026-06-29/witnesses/aspen1.json.sig
require Git source: true
source acquisition kind: git
source acquisition URL: file:///home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/git-origin/mantle.git
source acquisition commit: 7ace5e14c39bf56f4ab7a2e8b354a0de61c00c44
rebuild audit: /home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/full-work/witness-rebuild-audit/meta.json
crunch-aspen1-1:yCUo8ySXwGDjGoa6qQaOG0SeuaxJRj5r3CP5WGvu+QI=
~~~

Returned witness audit source acquisition:

~~~json
{"mode":"git-derived-source","url":"file:///home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/git-origin/mantle.git","digest_blake3":"6d1e366bbb94f1809c814fba9db29efbe3c2b33c02e2862ed2950a86d000c08a","fetched_path":"/home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/full-work/source-acquisition.tar","commit":"7ace5e14c39bf56f4ab7a2e8b354a0de61c00c44","reference":"refs/heads/main","tag":null,"archive_profile":"mantle-release-source","archive_version":"v1","status":"verified","error":null}
~~~

Returned rebuilt output:

~~~json
{"published_name":"binaries/01-mantle","path":"/home/brittonr/mantle-witness/git-source-witness-replay-2026-06-29/full-work/rebuilt-outputs/binaries/01-mantle","digest_blake3":"37768afa1d949dc27523954884abb82056e7bda06386a7f15d3b893d7e71a3b4"}
~~~

Witness attestation summary:

~~~json
{"release_attestation_digest_blake3":"767747b2f7f9251c689bb6fc414b5f4e359f7fa961e54f77ff3580d04d366649","witness_identity":"aspen1","rebuilt_digests":[{"name":"binaries/01-mantle","algorithm":"blake3","digest":"37768afa1d949dc27523954884abb82056e7bda06386a7f15d3b893d7e71a3b4"}],"rebuild_environment_summary":{"system":"x86_64-linux","toolchain":"replay-driver","host_class":"aspen1"}}
~~~

## Witness import and final verification

Command evidence:

~~~text
pueue task 169: mantle attest witness-import target/release-verification/git-source-witness-replay-2026-06-29 target/aspen-return/git-source-witness-replay-2026-06-29/git-source-witness-replay-2026-06-29
pueue task 172: mantle attest release-verify target/release-verification/git-source-witness-replay-2026-06-29 --trusted-public-key <publisher> --trusted-public-key <aspen1>
~~~

Final verification JSON:

~~~json
{
  "release_attestation_digest": "767747b2f7f9251c689bb6fc414b5f4e359f7fa961e54f77ff3580d04d366649",
  "release_signer_key_name": "crunch-britton-desktop-1",
  "discovered_witness_count": 1,
  "considered_witness_count": 1,
  "technical_class": "external-witness-match",
  "policy_status": "satisfied",
  "final_class": "quorum-satisfied",
  "matching_witness_count": 1,
  "independent_witness_identities": 1,
  "revoked_witness_count": 0,
  "independent_agreement_status": "satisfied",
  "independent_agreement_class": "independent-rebuild-agreement",
  "independent_agreement_report_digest": "1129e811ee7bc71ae04a5d5bab45e9a2e2c69d044669c0da78b7772ef9854424",
  "independent_agreement_counted_witness_count": 1,
  "independent_agreement_skipped_witness_count": 0,
  "independent_agreement_failed_witness_count": 0,
  "independent_agreement_witnesses": [
    {
      "witness_identity": "aspen1",
      "signer_key_name": "crunch-aspen1-1",
      "signature_valid": true,
      "digest_match": true,
      "independence_domain": "aspen1",
      "policy_counted": true,
      "classification_reason": "counted"
    }
  ]
}
~~~
