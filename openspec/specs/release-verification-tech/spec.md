# release-verification-tech Specification

## Purpose
Define the technical artifact formats, discovery model, trust classes, and CLI
surfaces used to verify release attestations and witness attestations.
## Requirements
### Requirement: Release verification MUST use canonical release attestations

Crunch MUST define a canonical release-attestation format that binds a release
identifier to the verified release-evidence manifest digest, the published
binary digest set, the proof identity, and the workflow identity. The first
phase MUST use canonical compact JSON bytes. The release-attestation identity
MUST be the digest of those canonical bytes. In the first phase, `release
identifier` means the human-readable release name carried by the release
evidence bundle, `proof identity` means the proof-bundle digest plus proof
mode, and `workflow identity` means the workflow command and workflow version
that produced the proof bundle.

#### Scenario: Canonical release attestation digest is stable

- GIVEN one release attestation with unchanged content
- WHEN it is serialized more than once
- THEN the canonical bytes are identical
- AND the canonical digest is identical

#### Scenario: Release attestation binds release evidence

- GIVEN a release evidence bundle that already verifies successfully
- WHEN a release attestation is created for it
- THEN the attestation records the release-evidence manifest digest
- AND it records the published binary digest set from that release candidate as
  deterministic per-output `(name, algorithm, digest)` tuples
- AND the first-phase tuples all use `algorithm = blake3`

### Requirement: Witness attestations MUST bind rebuilt outputs to one release attestation

Crunch MUST define a witness-attestation format whose signed payload names one
release-attestation digest and records the witness rebuilt output digest set.
The format MUST carry an explicit versioned signature-suite identifier and a
signature encoding that binds signer identity to the canonical witness
attestation digest.

#### Scenario: Witness with wrong release reference is rejected

- GIVEN a witness attestation that names a different release-attestation digest
- WHEN verification runs
- THEN the command exits non-zero
- AND it identifies the mismatched release reference

#### Scenario: Witness with wrong rebuilt output digest is rejected

- GIVEN a witness attestation whose rebuilt output digest set differs from the
  published release digest set
- WHEN verification runs
- THEN the command exits non-zero
- AND it identifies the mismatched rebuilt output digest

#### Scenario: Witness with invalid signature is rejected

- GIVEN a witness attestation whose detached signature is missing, unknown, or
  cryptographically invalid
- WHEN verification runs
- THEN the command exits non-zero
- AND it identifies the signature failure before digest comparison

### Requirement: First-phase attestation discovery MUST be file-based

Crunch MUST define a first-phase file-based discovery model for one release
attestation and its witness attestations.

#### Scenario: Verifier discovers witness files from the verification directory

- GIVEN a verification directory containing one release attestation,
  corresponding detached signatures, and zero or more witness attestation files
- WHEN verification runs
- THEN the command finds witness material through the configured directory
  layout alone
- AND it does not require an external witness-discovery service

### Requirement: Verifier MUST separate technical validity from policy sufficiency

Crunch MUST report technical validity independently from social-policy
sufficiency when verifying decentralized release material.

#### Scenario: Technically valid but policy-insufficient witness set

- GIVEN a release attestation and witness attestations whose signatures and
  rebuilt output digests are all technically valid
- AND the witness set does not satisfy the configured quorum or independence
  policy
- WHEN verification runs
- THEN the command reports technical success
- AND it separately reports policy insufficiency

### Requirement: Technical release verification MUST expose technical classes

Crunch MUST expose a technical verification class derived from release
evidence, self-proof status, and independent witness agreement. The initial
normative technical class order MUST include `bundle-consistent`,
`self-proof-valid`, and `external-witness-match`.

#### Scenario: Self-proof tier remains technically valid without witnesses

- GIVEN a release whose bundled release evidence and self-proof already verify
- AND no witness attestations exist
- WHEN verification runs
- THEN the reported technical tier is `self-proof-valid`
- AND the command does not report a technical verification failure

#### Scenario: Matching external witness raises technical class

- GIVEN a release whose bundled release evidence and self-proof already verify
- AND at least one external witness attestation matches the release digests
- WHEN verification runs
- THEN the reported technical class is `external-witness-match`
- AND the output names the satisfied technical class

### Requirement: CLI MUST export verifier-ready trusted public keys from signing keys

Crunch MUST provide `crunch attest key-show` to print the verifier-ready
trusted public key string for an existing signing keypair file.
ID: release.verification.tech.trustedkey.export.cli

The command MUST:
- accept an explicit signing-key path,
- support the default config signing-key location when no explicit path is
  given,
- fail clearly when no signing key exists,
- avoid generating or mutating key material as a side effect, and
- print the exact `name:base64` token accepted by `--trusted-public-key`.

#### Scenario: Explicit signing-key path prints verifier token

- GIVEN an existing signing keypair file passed with `--signing-key`
- WHEN the operator runs `crunch attest key-show`
- THEN the command exits successfully
- AND it prints the exact `name:base64` trusted public key token for that key

#### Scenario: Default config signing key prints verifier token

- GIVEN no explicit signing-key path
- AND the default config signing-key file exists
- WHEN the operator runs `crunch attest key-show`
- THEN the command exits successfully
- AND it prints the exact `name:base64` trusted public key token for that key

#### Scenario: Missing signing key fails without key generation

- GIVEN no explicit signing-key path and no default config signing-key file
- WHEN the operator runs `crunch attest key-show`
- THEN the command exits non-zero
- AND the diagnostic says that no signing key was found
- AND the command does not generate a new key file

