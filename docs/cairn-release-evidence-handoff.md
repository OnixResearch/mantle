# Cairn release evidence handoff

Mantle can carry Cairn lifecycle and release-readiness evidence inside release
bundles as external evidence. Cairn owns lifecycle readiness, policy semantics,
change/archive consistency, and gate interpretation. Mantle owns only the
bundle-local handoff: artifact id, role, schema, BLAKE3 digest, Cairn policy
digest, release-readiness id, coverage ids, non-claims, and the exact reviewed
archive receipt for Cairn's authenticated-input prerequisite.

`cairn_release_handoff::validate_cairn_release_evidence_handoff` is the pure
validator for already-loaded handoff rows and the typed archive dependency. The
release shell remains responsible for bounded no-follow file reads, BLAKE3
measurement, and locating explicit Cairn exports.

## Supported roles

- `cairn-release-readiness-receipt`
- `cairn-change-validation-receipt`
- `cairn-archive-evidence-index`

Every row must include a matching supported schema and state that Mantle is not
proving release correctness. Every handoff must also bind the pinned archived
`authenticate-stack-provenance-inputs` receipt and exact Cairn revision. Passing
validation proves only that the dependency and handoff rows are measured,
complete, supported, linked to the same bundle, and non-overclaiming. For these
lifecycle handoff rows, Mantle does not independently re-run producer signatures
and does not prove Cairn, release, build, source, artifact, or deployment correctness.

## Reviewer-signed source evidence is a separate boundary

The current Cairn `source_review` producer and independent Valence `source_review`
consumer export a closed version-1 manual-review contract. This is not the
historical September missing-producer gate, lifecycle readiness, or proof of a
source property. Mantle independently verifies the actual Artifact Auth Ed25519
statements, full public-key identities, exclusive `source-review-approval` role,
generation/currentness/revocation facts, author exclusion, distinct-key threshold,
and the exact signed source-subject / current claim-root / review-policy tuple.
Existing Cairn claim-DAG root semantics are retained: `subject_root` is not
redefined as the source-review subject hash.

The source subject must name the measured release archive BLAKE3, immutable
revision when available, `mantle-release-source` snapshot profile version 1, and
`source` projection. An attached source observation is recomputed through
`crunch-source-core` from the release acquisition facts; arbitrary observation
digests and different archive/profile bindings are not accepted. Locators are
discovery hints, not review authority. Optional source observations and optional
review absence preserve generic-release compatibility. Source-built receipt
association remains dependent on the genuine source-built I4 producer; ordinary
source observations are not substituted for it.

Bundle the following three exact files through `release create`'s existing
`--external-evidence` / `--external-evidence-role` /
`--external-evidence-schema` options, all with claim scope
`source-review-identity-linkage-only` and the seven source-review non-claims:

| File | Role | Schema |
| --- | --- | --- |
| Cairn attachment | `cairn-source-review-attachment` | `cairn.source-review-attachment.v1` |
| Explicit verification policy | `cairn-source-review-policy` | `cairn.source-review-verification-policy.v1` |
| Valence acceptance | `valence-source-review-acceptance` | `valence.source-review-acceptance.v1` |

`mantle release verify BUNDLE --source-review-policy POLICY.json
--require-source-review --json` requires the exact externally selected policy
bytes to equal the bundled policy, then remeasures source and evidence bytes and
independently authenticates the review. A bundled policy alone cannot select
operator authority. Its unselected valid evidence is reported as `recorded`, not
as a selected required-policy discharge. A malformed, stale, under-threshold,
unsupported, or partially missing attachment is not silently ignored. JSON and
human verification checks include the separate `source-review` contributor;
reviewer counts never become release-signer, build-witness, StageX-no-quorum, or
verified-property counts.

The pure Rust API, `crunch_release_core::source_review::verify_release_source_review`,
takes an owned `SourceReviewVerificationRequest` and returns
`ReleaseEvidenceEvaluation<SourceReviewVerification>`, like the other release
evaluators. The manifest and artifact payloads move into the request without
copying, and the manifest returns with the verification. The trusted policy
remains an explicit operator input; this boundary form does not change review
acceptance or signature checks.

### Nickel policy authoring

`config/source-review/contracts.ncl` validates supplied policy facts;
`config/source-review/default.ncl` defines optional-by-default authoring and two
required presets. `reviewed` requires one approval; `stagex_inspired` requires
two and excludes authors. Neither preset selects operator trust or creates
review approval.

The defaults declare 64-row review, trusted-key and author-key limits. The
policy contract enforces the trusted-key and author-key list bounds and
requires the approval count to match the authentication threshold.
Reviewer purposes are exclusively `source-review-approval`; duplicate full-key
identities and currentness references from a different verifier context are
rejected. Current, overlap, superseded, revoked and unknown currentness values
are supplied observations, not permission for an ineligible key to count.

Apply `contracts.admit` with the selected preset to the operator's frozen
subject, claim-root, key and currentness facts, then export that authored input
with `nickel export --format json POLICY.ncl`. Use the resulting exact
`POLICY.json` bytes for the external producer workflow and explicit Mantle
selection below. The paired fixtures in `config/source-review/tests/` are
synthetic contract controls, never actual release policies or approvals. The
Rust `SourceReviewVerificationPolicy` and related closed DTOs in
`crunch-release-core::source_review` remain the runtime admission contract;
successful Nickel export does not replace signature or Valence verification.

### Real evidence preparation

Prerequisites are actual reviewed source bytes, the current explicit Cairn claim
root and review-policy digest, a verification policy containing the exact trusted
full public keys and currentness observation, and genuine reviewer private keys
held outside Cairn/Mantle/Valence. The tools do not invent approval, discover trust,
or sign for a reviewer. `CLAIM1.json` and `CLAIM2.json` must be reviewer-authorized
claims over that same source/root/policy; disposition must be `approved` for a
key to count. Different full keys may have identical display labels. A
`needs_revision` statement is attributable but never an approval.

For each reviewer, use the current public commands (paths are relative to the
explicit operator evidence root; output files must be fresh):

```sh
cairn source-review signing-request --root "$INPUT_ROOT" \
  --claim CLAIM1.json --policy POLICY.json --producer "$REVIEWER1" \
  --key-id "$KEY1_LABEL" --generation "$KEY1_GENERATION" \
  --public-key-hex "$KEY1_PUBLIC_HEX" > "$INPUT_ROOT/REQUEST1.json"
cairn source-review signing-request --root "$INPUT_ROOT" \
  --claim CLAIM1.json --policy POLICY.json --producer "$REVIEWER1" \
  --key-id "$KEY1_LABEL" --generation "$KEY1_GENERATION" \
  --public-key-hex "$KEY1_PUBLIC_HEX" --raw > "$INPUT_ROOT/STATEMENT1.bin"
```

The reviewer signs exactly `STATEMENT1.bin` with the corresponding Ed25519 key
using their external signer, producing a raw 64-byte signature. They must not sign
pretty JSON, a digest string, or an unrelated claim. After obtaining the actual
lowercase signature hex, assemble the first attachment, extract its signed row
for inclusion, then assemble the second reviewer in the same way:

```sh
cairn source-review assemble --root "$INPUT_ROOT" --request REQUEST1.json \
  --policy POLICY.json --signature-hex "$KEY1_SIGNATURE_HEX" > "$INPUT_ROOT/ONE.json"
jq '.reviews[0]' "$INPUT_ROOT/ONE.json" > "$INPUT_ROOT/ROW1.json"
cairn source-review assemble --root "$INPUT_ROOT" --request REQUEST2.json \
  --policy POLICY.json --signature-hex "$KEY2_SIGNATURE_HEX" \
  --include ROW1.json > "$INPUT_ROOT/ATTACHMENT.json"
cairn source-review verify --root "$INPUT_ROOT" \
  --attachment ATTACHMENT.json --policy POLICY.json
valence source-review verify --artifact-root "$INPUT_ROOT" \
  --attachment ATTACHMENT.json --policy POLICY.json \
  --output-root "$FRESH_RECEIPT_ROOT" --receipt ACCEPTANCE.json
```

Repeat the signing-request commands with reviewer 2's claim/key facts to produce
`REQUEST2.json` and its actual external signature before the second assemble.
The output-root directory must exist; Valence uses create-new output semantics.
Do not import a fabricated `accepted_manual_review` string: Mantle compares the
entire receipt identity/linkage and independently redoes authentication anyway.
Deterministic test keys/receipts are only test evidence and must never be used as
actual release approval. A selected two-reviewer policy still needs two genuine
authorized approvals before any real release can claim that policy satisfied.

