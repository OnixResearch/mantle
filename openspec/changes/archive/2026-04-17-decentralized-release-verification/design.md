# Design: Decentralized release verification

## Context

Today crunch has three strong ingredients for release integrity:

- release evidence bundles with canonical manifests and bundle-local verify
- self-hosting proof bundles that bind stage0/stage1/stage2 evidence together
- native attestations for artifact and closure provenance

Those pieces still live mostly inside one build universe. A malicious or buggy
single builder can still produce a self-consistent release bundle,
self-consistent proof bundle, and self-consistent linkage record. Stronger
claims need outside witnesses and a verifier that can distinguish technical
agreement from social-policy sufficiency.

## Goals / Non-Goals

**Goals:**

- define canonical technical formats for release and witness attestations
- define file-based discovery and revocation inputs for the first phase
- define how the verifier separates technical agreement from social-policy
  sufficiency
- define a small initial trust-tier model that does not bake local policy into
  artifact hashes

**Non-Goals:**

- implement a global transparency log in this change
- settle one permanent quorum policy for every crunch deployment
- replace the existing release evidence bundle format
- claim that decentralized verification removes all trust assumptions

## Decisions

### 1. Split release trust into technical and social planes

**Choice:** decentralized release verification uses two explicit planes:

- a **technical evidence plane** with canonical release attestations, witness
  attestations, signature checks, and digest comparisons
- a **social trust plane** with role bindings, quorum thresholds,
  independence rules, and revocation policy

**Rationale:** technical verification and social governance evolve at different
speeds. Keeping them separate prevents policy churn from invalidating artifact
identity.

**Implementation:** the verifier first establishes technical status, then
applies policy over the validated witness set.

### 2. Release attestations use canonical compact JSON and BLAKE3 identity

**Choice:** future release publication will define a canonical release
attestation above release evidence. The first phase uses canonical compact JSON
bytes and BLAKE3 over those canonical bytes as the release-attestation
identity.

**Rationale:** witnesses need one stable thing to sign. Signing a loose bundle
path or ad hoc file set is too ambiguous.

**Implementation:** the release attestation schema is:

```json
{
  "schema": "crunch-release-attestation-v1",
  "release_id": "<human-readable release name>",
  "release_evidence_manifest_digest_blake3": "<64-char hex>",
  "proof_bundle_digest_blake3": "<64-char hex>",
  "proof_mode": "<proof mode>",
  "workflow": {
    "command": "<workflow command>",
    "version": "<workflow version>"
  },
  "binary_digests": [
    { "name": "crunch", "algorithm": "blake3", "digest": "<64-char hex>" }
  ]
}
```

`binary_digests` is a deterministic list of per-output `(name, algorithm,
digest)` tuples sorted by `(name, algorithm, digest)` before serialization.
The release-attestation identity is the 64-character lowercase hex BLAKE3
encoding of the canonical bytes. Canonical key ordering is the key order shown
in this schema for every object at every nesting level. Nested objects follow
that same rule: `workflow` keys are ordered as `command`, `version`, and each
binary tuple is ordered as `name`, `algorithm`, `digest`. `proof_bundle_digest_blake3`
and `proof_mode` are always required in the first phase because this design is
for releases that already carry full proof material.

### 3. Witness attestations bind rebuilt outputs to one release attestation

**Choice:** each independent rebuilder publishes a signed witness attestation
that names one release-attestation digest and records a rebuilt output digest
set plus a bounded rebuild-environment summary.

**Rationale:** the verifier needs an explicit machine-checkable answer to
"which release did this witness rebuild, and what outputs did they get?"

**Implementation:** the witness attestation schema is:

```json
{
  "schema": "crunch-witness-attestation-v1",
  "release_attestation_digest_blake3": "<64-char hex>",
  "witness_identity": "<stable witness key name>",
  "signature_suite": "ed25519-detached-v1",
  "rebuilt_digests": [
    { "name": "crunch", "algorithm": "blake3", "digest": "<64-char hex>" }
  ],
  "rebuild_environment_summary": {
    "system": "<target system>",
    "toolchain": "<short toolchain label>",
    "host_class": "<bounded host class label>"
  }
}
```

`rebuilt_digests` follows the same deterministic tuple ordering as
`binary_digests`. `rebuild_environment_summary` is inside the signed payload,
contains only bounded string labels, and must stay under a small fixed field
set so canonical bytes stay stable and reviewable. Its nested key order is
`system`, `toolchain`, `host_class`.

### 4. The first signature suite is detached Ed25519

**Choice:** the first phase uses `ed25519-detached-v1` as the release and
witness signature suite.

**Rationale:** crunch already uses Ed25519-style trust material for store and
substitution signing, and attestation payloads are small enough that detached
signatures over canonical bytes are simple and sufficient.

**Implementation:** each release or witness attestation has a sibling `.sig`
file whose content is:

```text
<key-name>:<base64-ed25519-signature>
```

The signature is computed over the canonical attestation bytes. The release
attestation is signed by the release-publishing signer role; witness
attestations are signed by witness rebuilder roles. Verification fails before
digest comparison if the `.sig` file is missing, malformed, unknown, or
cryptographically invalid.

### 5. File-based discovery and revocation are first-phase only

**Choice:** the first phase uses file-based discovery for release and witness
attestations and file-based revocation input, not an external service.

**Rationale:** this satisfies the no-transparency-log constraint while giving a
concrete workflow that is easy to test and easy to publish.

**Implementation:** the first-phase layout is:

```text
<verification-dir>/
  release-attestation.json
  release-attestation.json.sig
  witnesses/
    <witness-identity>.json
    <witness-identity>.json.sig
  policy.json
  revocations.json
```

`policy.json` is a file-based policy artifact with this first-phase shape:

```json
{
  "schema": "crunch-release-policy-v1",
  "min_matching_witnesses": 2,
  "independence_field": "witness_identity",
  "trusted_release_signers": ["release-signer-1"],
  "trusted_witness_signers": ["witness-a", "witness-b"]
}
```

`revocations.json` is a file-based policy input with this first-phase shape:

```json
{
  "schema": "crunch-release-revocations-v1",
  "revoked_witness_keys": ["witness-a"],
  "revoked_witness_attestation_digests_blake3": ["<64-char hex>"]
}
```

The verifier discovers `witnesses/*.json`, requires matching `.sig` sidecars,
applies revocations, then evaluates technical status and policy status.

### 6. Verifier outputs separate technical class, policy status, and final class

**Choice:** verifier output reports:

- **technical class**: `bundle-consistent`, `self-proof-valid`, or
  `external-witness-match`
- **policy status**: whether the witness set satisfies configured quorum,
  independence, and revocation rules
- **final class**: either the technical class or the promoted
  `quorum-satisfied` class when policy passes

**Rationale:** `quorum-satisfied` is not purely technical. It depends on social
policy. Keeping technical and final classes separate preserves the two-plane
boundary while still exposing one honest operator-facing result.

**Implementation:** machine-readable output must include all three fields.
Human-readable output may summarize them, but it must not collapse technical
failure and policy failure into one vague status.

### 7. CLI and stored outputs reuse the envelope JSON convention

**Choice:** release and witness attestation display should reuse the established
`crunch attest` envelope shape.

**Rationale:** the repo already has a working convention for machine-readable
attestation display. Reusing it avoids another output dialect.

**Implementation:** JSON output uses:

```json
{
  "kind": "crunch-release-attestation",
  "digest": "<64-char hex>",
  "stored_path": "<path>",
  "attestation": { ... }
}
```

and the analogous `crunch-witness-attestation` shape for witnesses. In the
first phase, `stored_path` is the deterministic filesystem path of the emitted
attestation sidecar file inside the verification directory, not a new abstract
locator scheme.

## Verification Strategy

Every spec scenario must map to at least one explicit test shape.

### Technical tests

- `release-verification-tech / Canonical release attestation digest is stable`:
  canonical release-attestation bytes are stable across repeated serialization
- `release-verification-tech / Release attestation binds release evidence`:
  real release evidence produces matching manifest and binary digest tuples
- `release-verification-tech / Witness with wrong release reference is rejected`:
  wrong `release_attestation_digest_blake3` is rejected
- `release-verification-tech / Witness with wrong rebuilt output digest is rejected`:
  wrong rebuilt digest tuple is rejected
- `release-verification-tech / Witness with invalid signature is rejected`:
  missing or invalid detached signature is rejected
- `release-verification-tech / Self-proof tier remains technically valid without witnesses`:
  zero witnesses still yields technical class `self-proof-valid`
- `release-verification-tech / Matching external witness raises technical class`:
  one matching witness raises technical class to `external-witness-match`

### Social-policy tests

- `release-verification-social / Policy update does not change technical artifact digests`:
  changing `policy.json` does not change attestation digests
- `release-verification-social / Insufficient quorum fails policy even after technical agreement`:
  insufficient quorum keeps technical success but fails policy status
- `release-verification-social / Satisfied quorum promotes final release class`:
  satisfied quorum promotes final class to `quorum-satisfied`
- `release-verification-social / Same actor cannot satisfy all required witness slots`:
  same witness domain cannot satisfy independence rules alone
- `release-verification-social / Revoked witness no longer satisfies release policy`:
  revoked witness keys or attestation digests degrade policy status without
  mutating the underlying technical result
- `release-verification-social / Revocation input comes from a file-based policy artifact`:
  revocation input is loaded from `revocations.json` before policy evaluation

### Discovery tests

- `release-verification-tech / Verifier discovers witness files from the verification directory`:
  verifier discovers witness files through the file layout under `witnesses/`
- discovery rejects missing `.sig` sidecars before digest comparison
- discovery applies `revocations.json` before quorum counting

## Risks / Trade-offs

**[Protocol without witnesses]**
A technical protocol can exist before any real witness network uses it.

**Mitigation:** require matching and mismatching witness fixtures in validation,
and keep social policy explicit about quorum expectations.

**[Policy sprawl]**
Too many local policy knobs could make results hard to compare across users.

**Mitigation:** ship a small set of named policy profiles in addition to raw
policy fields.

**[False confidence]**
Operators may read "technically valid" as "fully trusted."

**Mitigation:** keep output language explicit: technical class, policy status,
and final class are different things.

**[Schema churn]**
Canonical attestation formats are hard to change once published.

**Mitigation:** keep first-phase schemas narrow, versioned, and focused on
digest binding plus witness comparison.
