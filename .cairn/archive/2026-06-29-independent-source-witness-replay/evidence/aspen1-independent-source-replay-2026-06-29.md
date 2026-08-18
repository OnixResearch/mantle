# Aspen1 independent-source witness replay

Date: 2026-06-29

## Question

Can `aspen1.local` run `mantle release witness-rebuild --require-independent-source`, fetch the release-declared external source archive, verify its BLAKE3 digest before extraction, rebuild the published artifact, and produce a counted witness?

## Decision

Yes. Aspen1 completed the witness rebuild for release `independent-source-witness-replay-2026-06-29`; the witness audit records `source_acquisition.status = "verified"`, the fetched source archive digest matches the release manifest source digest, the rebuilt output digest matches the published release binary digest, and final release verification reached `final_class = quorum-satisfied` with Aspen1 counted.

## Important bounds

- This proves the external archive source-acquisition path (`file://.../external-source/source.tar`) and digest verification before witness rebuild. It does not prove raw Git tag/source reconstruction or upstream tag signature verification.
- To avoid rerunning the publisher-side self-hosting proof, the new release evidence bundle reused the previously successful stabilized self-hosting source/proof/binary artifact set from commit `4d6ed459`, while the current Mantle CLI created the `source_acquisition` manifest and enforced `--require-independent-source` on the witness.
- The request bundle source path was hardlinked to the transferred external source archive on Aspen because release bundle verification still requires the manifest-listed `source_archive` file to exist before the independent-source preparation gate. The audit shows the witness fetched and verified the release-declared external URL into `request.work/source-acquisition.tar` before launching the workflow.

## Release evidence creation with source acquisition

Command summary (pueue task `103`):

~~~text
release_id=independent-source-witness-replay-2026-06-29
source_url=file:///home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/external-source/source.tar
/home/brittonr/git/mantle/.tmp-independent-source/target/debug/mantle release create \
  --release-id "$release_id" \
  --bundle-dir /home/brittonr/git/mantle/target/release-evidence/$release_id \
  --binary /home/brittonr/git/mantle/target/release-evidence/stabilized-self-hosting-witness-replay-2026-06-29/binaries/01-stage2-mantle \
  --proof-bundle /home/brittonr/git/mantle/target/release-evidence/stabilized-self-hosting-witness-replay-2026-06-29/proof/self-hosting \
  --source-acquisition-url "$source_url"
/home/brittonr/git/mantle/.tmp-independent-source/target/debug/mantle release verify \
  /home/brittonr/git/mantle/target/release-evidence/$release_id
~~~

Output excerpt:

~~~text
source acquisition: file:///home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/external-source/source.tar
manifest: manifest.json
release evidence verified: /home/brittonr/git/mantle/target/release-evidence/independent-source-witness-replay-2026-06-29
release id: independent-source-witness-replay-2026-06-29
binaries: 1
source digest: 6a3068af1d14b04c45d306fa119a8fcf257a55491c94884ca464a8b2e61f1e21
stage2 digest: 99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328
proof mode: fixed-point
~~~

Manifest excerpt:

~~~json
"source_acquisition":{"kind":"external-archive","url":"file:///home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/external-source/source.tar","digest_blake3":"6a3068af1d14b04c45d306fa119a8fcf257a55491c94884ca464a8b2e61f1e21"}
~~~

## Signing, policy, and witness request export

Command summary (pueue task `112`):

~~~text
CRUNCH_CONFIG_DIR=target/release-signing/independent-source-witness-replay-2026-06-29/publisher \
  .tmp-independent-source/target/debug/mantle release attest \
  target/release-evidence/independent-source-witness-replay-2026-06-29 \
  --verification-dir target/release-verification/independent-source-witness-replay-2026-06-29

.tmp-independent-source/target/debug/mantle attest policy-init \
  target/release-verification/independent-source-witness-replay-2026-06-29 \
  --profile single-witness \
  --trusted-release-signer crunch-britton-desktop-1 \
  --trusted-witness-identity aspen1 \
  --force

.tmp-independent-source/target/debug/mantle release witness-export \
  target/release-evidence/independent-source-witness-replay-2026-06-29 \
  --verification-dir target/release-verification/independent-source-witness-replay-2026-06-29 \
  --request-dir target/release-witness-requests/independent-source-witness-replay-2026-06-29
~~~

Trusted publisher key:

~~~text
crunch-britton-desktop-1:9dR60KU9eWZyc2UbZ+LbIf3iHv4ZuitikW/DqH4ccXo=
~~~

Signed release attestation excerpt:

~~~json
{
  "release_id": "independent-source-witness-replay-2026-06-29",
  "binary_digests": [
    {
      "name": "binaries/01-01-stage2-mantle",
      "algorithm": "blake3",
      "digest": "99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328"
    }
  ]
}
~~~

## Aspen transfer and preflight

Transfer evidence (pueue tasks `123`, `128`, `164`):

~~~text
6a3068af1d14b04c45d306fa119a8fcf257a55491c94884ca464a8b2e61f1e21  /home/brittonr/git/mantle/target/aspen-transfer/independent-source-witness-replay-2026-06-29/external-source/source.tar
3833935c14e280da6f5d3bf458481479e02cad0630af2c68342758f9fad556e7  /home/brittonr/git/mantle/target/aspen-transfer/independent-source-witness-replay-2026-06-29/bin/mantle-current
...
/home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/bin/mantle-current
/home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/external-source/source.tar
/home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request/request.json
/home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/transfer-digests.blake3
...
206075810 -rw------- 2 brittonr brittonr 972889088 Jun 29 18:24 /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/external-source/source.tar
206075810 -rw------- 2 brittonr brittonr 972889088 Jun 29 18:24 /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request/release-evidence/independent-source-witness-replay-2026-06-29/source/.tmp4nw6s8
~~~

Check-only preflight (pueue task `165`):

~~~text
witness rebuild preflight OK: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request
release id: independent-source-witness-replay-2026-06-29
scratch root: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work
verification output: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work/release-verification/independent-source-witness-replay-2026-06-29
proof bundle output: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work/proof-bundle
require independent source: true
source acquisition URL: file:///home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/external-source/source.tar
audit metadata: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work/witness-rebuild-audit/meta.json
check only: no rebuild executed, no witness sidecars written
~~~

## Full Aspen1 witness rebuild

Command summary (pueue task `169`):

~~~text
ssh aspen1.local 'bash -lc "/home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/replay.sh"'
~~~

Output excerpt:

~~~text
witness rebuild completed: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request
release id: independent-source-witness-replay-2026-06-29
scratch root: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work
verification output: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work/release-verification/independent-source-witness-replay-2026-06-29
witness attestation: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work/release-verification/independent-source-witness-replay-2026-06-29/witnesses/aspen1.json
signature: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work/release-verification/independent-source-witness-replay-2026-06-29/witnesses/aspen1.json.sig
require independent source: true
source acquisition URL: file:///home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/external-source/source.tar
rebuild audit: /home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work/witness-rebuild-audit/meta.json
crunch-aspen1-1:AJF+lUlLhxRIGs5TnXKyJhdzlhzy9Gi2H5I5mtSqW3Q=
~~~

## Witness audit metadata

Source: `target/aspen1-witness-independent-source-witness-replay-2026-06-29/witness-rebuild-audit/meta.json`.

~~~json
{
  "schema": "mantle-witness-rebuild-audit-v1",
  "release_id": "independent-source-witness-replay-2026-06-29",
  "status": "success",
  "rebuilt_outputs": [
    {
      "published_name": "binaries/01-01-stage2-mantle",
      "digest_blake3": "99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328"
    }
  ],
  "source_acquisition": {
    "mode": "independent-source",
    "url": "file:///home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/external-source/source.tar",
    "digest_blake3": "6a3068af1d14b04c45d306fa119a8fcf257a55491c94884ca464a8b2e61f1e21",
    "fetched_path": "/home/brittonr/mantle-witness/independent-source-witness-replay-2026-06-29/request.work/source-acquisition.tar",
    "status": "verified",
    "error": null
  }
}
~~~

## Self-hosting proof summary

Source: `target/aspen1-witness-independent-source-witness-replay-2026-06-29/proof-bundle/summary.txt`.

~~~text
proof_mode: FixedPoint
selected_provider_kind: legacy-fetch
protected_exec_result: success
stage1_binary: 99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328 binaries/stage1-mantle
stage2_binary: 99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328 binaries/stage2-mantle
stage0_bwrap: 93cacbf9a523c439d160745228073492be1121b62074889345e676858fd1adf2 .../pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap
stage0_busybox: ce321b321d0f06650ebbb24b834f4d2d600f5765d9f5828fb09c41a3ccd31ed5 .../wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
stage2_bwrap: 93cacbf9a523c439d160745228073492be1121b62074889345e676858fd1adf2 .../pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap
stage2_busybox: ce321b321d0f06650ebbb24b834f4d2d600f5765d9f5828fb09c41a3ccd31ed5 .../wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
stage1_equals_stage2: true
stage0_bwrap_equals_stage2_bwrap: true
stage0_busybox_equals_stage2_busybox: true
stage2_hermeticity_mode: strict
stage2_fallback_events: []
~~~

## Final release verification

Command summary (pueue task `415`):

~~~text
.tmp-independent-source/target/debug/mantle attest witness-import \
  target/release-verification/independent-source-witness-replay-2026-06-29 \
  target/aspen1-witness-independent-source-witness-replay-2026-06-29/release-verification/independent-source-witness-replay-2026-06-29

.tmp-independent-source/target/debug/mantle --json attest release-verify \
  target/release-verification/independent-source-witness-replay-2026-06-29 \
  --trusted-public-key 'crunch-britton-desktop-1:9dR60KU9eWZyc2UbZ+LbIf3iHv4ZuitikW/DqH4ccXo=' \
  --trusted-public-key 'crunch-aspen1-1:AJF+lUlLhxRIGs5TnXKyJhdzlhzy9Gi2H5I5mtSqW3Q='
~~~

Saved JSON: `target/aspen1-witness-independent-source-witness-replay-2026-06-29/final-release-verify.json`.

~~~json
{
  "release_attestation_digest": "f286aeb40f466f449fbd4f23d39653db1d1beb9386327a3066c66feff14fea65",
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
  "independent_agreement_report_digest": "b7aa93e2e7fad4cdb85b0c79f9b0112d8baf7db9543a9be79c54a642364bdf0c",
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
