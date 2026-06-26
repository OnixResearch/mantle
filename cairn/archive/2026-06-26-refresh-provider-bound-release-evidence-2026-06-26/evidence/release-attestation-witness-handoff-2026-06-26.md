# Release attestation and witness handoff transcript — 2026-06-26

Task-ID: V2
Covers: r[verification_evidence.provider_bound_release_evidence_refresh_transcripts]

## Scope

This transcript records the publisher-side release attestation, witness-request
handoff, and final signed verification run for
`provider-bound-release-evidence-2026-06-26`.

Important bounded claim: the full checked-in `release witness-rebuild` helper was
executed and produced a fresh self-hosting proof bundle, but it failed closed at
witness sidecar creation because the current helper produces one rebuilt output
while this refreshed release request publishes two binaries. The final imported
witness sidecar below was created from the release bundle's portable replay
outputs, not from an independent external rebuild. This proves the
signature/import/policy verification path for the refreshed two-binary bundle; it
MUST NOT be cited as a successful independent external rebuild.

## Publisher attestation and request export

Command shape:

```sh
MANTLE=target/release-evidence/provider-bound-release-evidence-2026-06-26/binaries/01-mantle
BUNDLE=target/release-evidence/provider-bound-release-evidence-2026-06-26
VERIFY_ROOT=target/release-verification/provider-bound-release-evidence-2026-06-26
REQUEST_DIR=target/release-witness-requests/provider-bound-release-evidence-2026-06-26
STATE_DIR=target/release-verification-state/provider-bound-release-evidence-2026-06-26
PUBLISHER_CONFIG=target/release-signing/provider-bound-release-evidence-2026-06-26/publisher
WITNESS_ID=provider-bound-witness-2026-06-26

$MANTLE --state-dir "$STATE_DIR" --json release verify "$BUNDLE" \
  --require-deterministic-release \
  --require-provider-fixed-point-proof \
  > "$VERIFY_ROOT/bundle-verify.json"

CRUNCH_CONFIG_DIR="$PUBLISHER_CONFIG" \
  $MANTLE --state-dir "$STATE_DIR" --json release attest "$BUNDLE" \
  --verification-dir "$VERIFY_ROOT" \
  > "$VERIFY_ROOT/release-attest.json"

RELEASE_TRUSTED_KEY=$(CRUNCH_CONFIG_DIR="$PUBLISHER_CONFIG" \
  $MANTLE --state-dir "$STATE_DIR" attest key-show)
RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"

$MANTLE --state-dir "$STATE_DIR" --json attest policy-init "$VERIFY_ROOT" \
  --profile single-witness \
  --trusted-release-signer "$RELEASE_SIGNER_NAME" \
  --trusted-witness-identity "$WITNESS_ID" \
  > "$VERIFY_ROOT/policy-init.json"

$MANTLE --state-dir "$STATE_DIR" --json release witness-export "$BUNDLE" \
  --verification-dir "$VERIFY_ROOT" \
  --request-dir "$REQUEST_DIR" \
  > "$VERIFY_ROOT/witness-export.json"
```

Generated files:

```text
target/release-verification/provider-bound-release-evidence-2026-06-26/release-attestation.json
target/release-verification/provider-bound-release-evidence-2026-06-26/release-attestation.json.sig
target/release-verification/provider-bound-release-evidence-2026-06-26/policy.json
target/release-verification/provider-bound-release-evidence-2026-06-26/revocations.json
target/release-witness-requests/provider-bound-release-evidence-2026-06-26/request.json
```

Publisher public verifier token:

```text
crunch-britton-desktop-1:ZGucNN+ApSlD8YM/+qJmS/yhdkXGKgGksoH3O6LOqos=
```

Pre-witness `release-verify` was technically valid but policy-insufficient, as
expected for the single-witness policy before any witness sidecars were
imported:

```json
{
  "release_attestation_digest": "77b4a8b8bd76a2b3eac4f1e50a827508eb3d0b275d0d3a60293a9dffcadb254a",
  "release_signer_key_name": "crunch-britton-desktop-1",
  "discovered_witness_count": 0,
  "considered_witness_count": 0,
  "technical_class": "self-proof-valid",
  "policy_status": "insufficient",
  "final_class": "self-proof-valid",
  "matching_witness_count": 0,
  "independent_witness_identities": 0,
  "revoked_witness_count": 0,
  "policy_failure_reason": {
    "insufficient_quorum": {
      "required": 1,
      "matched": 0
    }
  }
}
```

## Full witness rebuild attempt

Command shape:

```sh
CRUNCH_CONFIG_DIR=target/release-signing/provider-bound-release-evidence-2026-06-26/witness \
CRUNCH_WITNESS_REBUILD_CLI_BIN="$PWD/target/release-evidence/provider-bound-release-evidence-2026-06-26/binaries/01-mantle" \
  ./scripts/rebuild-witness-request.sh \
  target/release-witness-requests/provider-bound-release-evidence-2026-06-26 \
  --identity provider-bound-witness-2026-06-26 \
  --system x86_64-linux \
  --toolchain rust-1.91.1 \
  --host-class nixos-25.05 \
  > target/release-verification/provider-bound-release-evidence-2026-06-26/witness-run/witness-rebuild.stdout \
  2> target/release-verification/provider-bound-release-evidence-2026-06-26/witness-run/witness-rebuild.stderr
```

Result: failed closed after producing a fresh proof bundle under
`target/release-witness-requests/provider-bound-release-evidence-2026-06-26.work/proof-bundle/`.

Failure line:

```text
error: build failed
rebuilt output count mismatch: supported workflow produces 1 output, request expects 2
```

The produced proof bundle itself reported a fixed point:

```text
stage1_equals_stage2: true
stage0_bwrap_equals_stage2_bwrap: true
stage0_busybox_equals_stage2_busybox: true
stage1_binary: cadfc8d78320bba63ff47a511df86b76966b3eb6f878a1d384c6201184d0b54b binaries/stage1-mantle
stage2_binary: cadfc8d78320bba63ff47a511df86b76966b3eb6f878a1d384c6201184d0b54b binaries/stage2-mantle
```

That digest does not match this release's published two-binary set, so it was
not imported as release witness evidence.

## Portable replay witness sidecar

The bundle-local portable replay script was run through `/bin/sh` to avoid the
expected `Exec format error` from directly executing a script without a shebang.

Command shape:

```sh
$MANTLE --state-dir "$STATE_DIR" --json release reproduce "$BUNDLE" \
  --rebuild-output-dir target/release-witness-requests/provider-bound-release-evidence-2026-06-26.work/portable-replay-output \
  --rebuild-command /bin/sh \
  --rebuild-arg "$PWD/target/release-evidence/provider-bound-release-evidence-2026-06-26/rebuild-both-binaries.sh" \
  --report-path target/release-witness-requests/provider-bound-release-evidence-2026-06-26.work/portable-replay-report.json
```

Replay result:

```json
{
  "matched_count": 2,
  "mismatched_count": 0,
  "missing_count": 0,
  "release_id": "provider-bound-release-evidence-2026-06-26",
  "report_digest_blake3": "b1a13f38dc19f6e383e771c300a2e7bb83faf63fc52573e34065f35f11fbe9c8"
}
```

The witness sidecar was then signed from those two replayed outputs:

```sh
CRUNCH_CONFIG_DIR=target/release-signing/provider-bound-release-evidence-2026-06-26/witness \
  $MANTLE --state-dir "$STATE_DIR" --json attest witness-create \
  target/release-witness-requests/provider-bound-release-evidence-2026-06-26.work/release-verification/provider-bound-release-evidence-2026-06-26 \
  --rebuilt-binary target/release-witness-requests/provider-bound-release-evidence-2026-06-26.work/portable-replay-output/binaries/01-mantle \
  --rebuilt-binary target/release-witness-requests/provider-bound-release-evidence-2026-06-26.work/portable-replay-output/binaries/02-stage2-mantle \
  --identity provider-bound-witness-2026-06-26 \
  --system x86_64-linux \
  --toolchain portable-release-replay-v1 \
  --host-class local-replay
```

Witness attestation digest and rebuilt outputs:

```json
{
  "digest": "2a30878eabfb14eb922b2a357261faaf361ea9fc6c30cd3d83ea2b91310080ab",
  "signer": "crunch-britton-desktop-1",
  "rebuilt_digests": [
    {
      "name": "binaries/01-mantle",
      "algorithm": "blake3",
      "digest": "b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3"
    },
    {
      "name": "binaries/02-stage2-mantle",
      "algorithm": "blake3",
      "digest": "84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd"
    }
  ]
}
```

Witness public verifier token:

```text
crunch-britton-desktop-1:6BeaeBqI/tUIQ6rq1ISUm4kkX1c1aYWtt3X5cw0B0PY=
```

The release and witness keys share the default generated key name on this host,
but the verifier tokens contain distinct Ed25519 public key material.

## Import and final release verification

Command shape:

```sh
$MANTLE --state-dir "$STATE_DIR" --json attest witness-import "$VERIFY_ROOT" \
  target/release-witness-requests/provider-bound-release-evidence-2026-06-26.work/release-verification/provider-bound-release-evidence-2026-06-26

$MANTLE --state-dir "$STATE_DIR" --json attest release-verify "$VERIFY_ROOT" \
  --trusted-public-key "crunch-britton-desktop-1:ZGucNN+ApSlD8YM/+qJmS/yhdkXGKgGksoH3O6LOqos=" \
  --trusted-public-key "crunch-britton-desktop-1:6BeaeBqI/tUIQ6rq1ISUm4kkX1c1aYWtt3X5cw0B0PY="
```

Final verifier output:

```json
{
  "release_attestation_digest": "77b4a8b8bd76a2b3eac4f1e50a827508eb3d0b275d0d3a60293a9dffcadb254a",
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
  "independent_agreement_report_digest": "8af4773aa1abe9f685d43d74341b2c9eeed47f797996439c8a86591f44532c78",
  "independent_agreement_counted_witness_count": 1,
  "independent_agreement_skipped_witness_count": 0,
  "independent_agreement_failed_witness_count": 0
}
```

## Bounded non-claims

- This transcript does not prove a successful independent external rebuild.
- The full witness helper currently needs a two-output handoff repair before it
  can sign the refreshed release request directly.
- The final satisfied policy status is a local configured-policy result over the
  portable replay witness sidecar.
- Private signing keys were generated under ignored `target/release-signing/`
  paths and are intentionally not tracked.
