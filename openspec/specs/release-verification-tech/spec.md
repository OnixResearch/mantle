# release-verification-tech Specification

## Purpose
Define the technical artifact formats, discovery model, trust classes, and CLI
surfaces used to verify release attestations and witness attestations.
## Requirements
### Requirement: Release verification MUST use canonical release attestations

Mantle MUST define a canonical release-attestation format that binds a release
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

Mantle MUST define a witness-attestation format whose signed payload names one release-attestation digest and records the witness rebuilt output digest set. The format MUST carry an explicit versioned signature-suite identifier and a signature encoding that binds signer identity to the canonical witness attestation digest. Release reference and rebuilt-output mismatches remain fatal for a witness that is otherwise signature-valid, while missing, unknown, or cryptographically invalid witness signatures are classified and skipped so other trusted witnesses can still satisfy policy.

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

#### Scenario: Witness with invalid signature is classified and skipped

- GIVEN a witness attestation whose detached signature is missing, unknown, or
  cryptographically invalid
- AND at least one other trusted witness can still be evaluated
- WHEN verification runs
- THEN the command keeps evaluating trusted witness material
- AND the invalid-signature witness appears in independent agreement output as
  skipped or failed with a signature reason

### Requirement: First-phase attestation discovery MUST be file-based

Mantle MUST define a first-phase file-based discovery model for one release
attestation and its witness attestations.

#### Scenario: Verifier discovers witness files from the verification directory

- GIVEN a verification directory containing one release attestation,
  corresponding detached signatures, and zero or more witness attestation files
- WHEN verification runs
- THEN the command finds witness material through the configured directory
  layout alone
- AND it does not require an external witness-discovery service

### Requirement: Verifier MUST separate technical validity from policy sufficiency

Mantle MUST report technical validity independently from social-policy
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

Mantle MUST expose a technical verification class derived from release
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

Mantle MUST provide `mantle attest key-show` to print the verifier-ready
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
- WHEN the operator runs `mantle attest key-show`
- THEN the command exits successfully
- AND it prints the exact `name:base64` trusted public key token for that key

#### Scenario: Default config signing key prints verifier token

- GIVEN no explicit signing-key path
- AND the default config signing-key file exists
- WHEN the operator runs `mantle attest key-show`
- THEN the command exits successfully
- AND it prints the exact `name:base64` trusted public key token for that key

#### Scenario: Missing signing key fails without key generation

- GIVEN no explicit signing-key path and no default config signing-key file
- WHEN the operator runs `mantle attest key-show`
- THEN the command exits non-zero
- AND the diagnostic says that no signing key was found
- AND the command does not generate a new key file

### Requirement: CLI MUST import returned witness sidecars safely

Mantle MUST provide `mantle attest witness-import <verification-dir> <source>`
to validate and copy returned witness-attestation sidecars into a publisher
verification directory.
ID: release.verification.tech.witness.import.cli

The command MUST:
- accept either a directory containing `witnesses/*.json` plus matching `.sig`
  sidecars or a single witness-attestation `.json` file,
- reject missing signature sidecars,
- reject witness attestations whose referenced release-attestation digest does
  not match the destination verification directory,
- reject conflicting existing witness identities when the bytes differ, and
- write imported sidecars under `<verification-dir>/witnesses/`.

#### Scenario: Import accepts matching witness sidecars

- GIVEN a publisher verification directory with a signed release attestation
- AND a returned witness-attestation file plus matching `.sig` sidecar whose
  release-attestation digest matches the destination verification directory
- WHEN the operator runs `mantle attest witness-import`
- THEN the command exits successfully
- AND it writes the witness files under `<verification-dir>/witnesses/`

#### Scenario: Import rejects missing witness signature sidecar

- GIVEN a returned witness-attestation `.json` file without the matching `.sig`
  sidecar
- WHEN the operator runs `mantle attest witness-import`
- THEN the command exits non-zero
- AND it names the missing witness signature sidecar

#### Scenario: Import rejects wrong release-attestation digest

- GIVEN a returned witness attestation referencing a different
  release-attestation digest than the destination verification directory
- WHEN the operator runs `mantle attest witness-import`
- THEN the command exits non-zero
- AND it names the release-attestation digest mismatch

#### Scenario: Import accepts exact duplicate witness material idempotently

- GIVEN the destination verification directory already contains
  `witnesses/alice.json` and `witnesses/alice.json.sig`
- AND a second import for `alice` has byte-identical attestation and signature
  contents
- WHEN the operator runs `mantle attest witness-import`
- THEN the command exits successfully
- AND it leaves the existing files unchanged

#### Scenario: Import rejects conflicting duplicate witness identity

- GIVEN the destination verification directory already contains
  `witnesses/alice.json`
- AND a second import for `alice` has different attestation bytes or different
  signature bytes
- WHEN the operator runs `mantle attest witness-import`
- THEN the command exits non-zero
- AND it names the conflicting witness identity instead of overwriting it

### Requirement: CLI MUST rebuild witness requests into witness material

Mantle MUST provide `mantle release witness-rebuild <request-dir>` to drive the
witness-side rebuild from an exported request directory and create witness
material without hand-wiring the rebuild recipe. The repo MUST also provide
`./scripts/rebuild-witness-request.sh` as the checked-in preflight wrapper for
that command.
ID: release.verification.tech.witness.rebuild.cli

The command MUST:
- verify the request-directory schema, layout, and bundled release-evidence
  bundle before rebuild,
- match the recorded workflow identity as the exact pair
  `(workflow_command="./scripts/prove-self-hosting.sh",
  workflow_version="crunch-self-hosting-proof-v2")` for the first supported
  rebuild path,
- accept the witness metadata already required by `mantle attest
  witness-create`, including signing-key selection, witness identity, system,
  toolchain, and host class,
- reject unsupported request layouts or unsupported recorded workflow
  identities before rebuild starts,
- let `./scripts/rebuild-witness-request.sh` discover `bwrap`, resolve and
  absolutize a real static `SNIX_BUILD_SANDBOX_SHELL`, derive a controlled
  scratch root, and rewrite `TMPDIR` and `CARGO_TARGET_DIR` under that root
  before invoking the command,
- rebuild the published outputs using the request's recorded workflow identity
  instead of an ad hoc operator-chosen recipe,
- reject rebuilt output count or output-name mismatches before writing witness
  sidecars, and
- write witness sidecars plus a rebuild audit directory to deterministic paths
  that `mantle attest witness-import` can consume, and
- keep any helper `--check` mode strictly preflight-only so successful preflight
  does not count as proof of a successful rebuild.

#### Scenario: Valid request rebuild creates witness sidecars and audit bundle

- GIVEN an exported witness request directory with a supported layout and a
  verified bundled release-evidence bundle
- WHEN the witness operator runs `mantle release witness-rebuild` with valid
  witness metadata
- THEN the command rebuilds the published outputs through the recorded
  workflow identity
- AND it writes witness sidecars for the rebuilt outputs
- AND it writes a deterministic rebuild audit directory alongside those
  sidecars

#### Scenario: Unsupported request workflow fails before rebuild

- GIVEN an exported witness request directory whose recorded workflow identity
  or request layout is unsupported by the installed mantle version
- WHEN the witness operator runs `mantle release witness-rebuild`
- THEN the command exits non-zero before the rebuild starts
- AND it names the unsupported workflow identity or request layout

#### Scenario: Helper preflight does not count as rebuild proof

- GIVEN the witness host runs `./scripts/rebuild-witness-request.sh --check`
- WHEN preflight succeeds
- THEN the helper reports only prerequisite status
- AND it does not claim to have produced publishable witness sidecars

#### Scenario: Rebuilt output mismatch blocks witness publication

- GIVEN an exported witness request directory whose published output list does
  not match the locally rebuilt output count or output names
- WHEN `mantle release witness-rebuild` finishes the local rebuild
- THEN the command exits non-zero
- AND it names the output mismatch
- AND it does not write publishable witness sidecars

### Requirement: Replayable witness scratch validation MUST stay helper-compatible

Replayable witness scratch validation MUST accept only real helper-owned
`tmp/` and `cargo-target/` directories at scratch-root top level and MUST
reject symlinked or non-directory helper-owned entries before the rebuild
workflow starts.
ID: release.verification.tech.witness.rebuild.scratch

#### Scenario: Helper-owned scratch directories are accepted

- GIVEN a witness scratch root that already contains top-level `tmp/` and
  `cargo-target/` directories created by `./scripts/rebuild-witness-request.sh`
- WHEN `mantle release witness-rebuild` validates that scratch root
- THEN validation succeeds
- AND the rebuild workflow may continue

#### Scenario: Symlinked helper-owned scratch entry is rejected

- GIVEN a witness scratch root whose top-level `tmp/` or `cargo-target/` entry
  is a symlink
- WHEN `mantle release witness-rebuild` validates that scratch root
- THEN the command exits non-zero before the workflow driver starts
- AND the diagnostic names the symlinked helper-owned entry rejection

#### Scenario: Non-directory helper-owned scratch entry is rejected

- GIVEN a witness scratch root whose top-level `tmp/` or `cargo-target/` entry
  is a regular file or another non-directory node
- WHEN `mantle release witness-rebuild` validates that scratch root
- THEN the command exits non-zero before the workflow driver starts
- AND the diagnostic names the helper-owned entry type mismatch

### Requirement: Replayable witness no-launch assertions MUST be observable

Replayable witness regression tests MUST back any "workflow driver never
launched" claim with an observable launch seam that the fake witness-rebuild
 driver implementation actually consumes.
ID: release.verification.tech.witness.rebuild.testing

#### Scenario: Fake driver writes the consumed launch signal

- GIVEN a replayable witness test configures the fake witness-rebuild driver
  with a launch-signal path
- WHEN the fake driver process starts
- THEN the fake driver writes that launch signal before it reports success or
  failure

#### Scenario: Preflight rejection proves launch never happened

- GIVEN a replayable witness negative test that expects preflight rejection
  before the workflow driver starts
- WHEN the command exits non-zero
- THEN the test asserts the rejection diagnostic
- AND the test asserts the consumed launch signal is absent

### Requirement: Technical verification surfaces reproducibility separately

Mantle MUST report reproducibility status separately from signature trust, witness agreement, and basic bundle integrity.
ID: release.verification.tech.reproducibility.status

The JSON output MUST make it possible for callers to require reproducibility evidence without confusing it with independent rebuild agreement or social quorum. Reproducibility status MUST include at least `absent`, `matched`, and `mismatched` states.

#### Scenario: Caller requires reproducibility

- GIVEN a release with valid bundle integrity and self-proof
- BUT no reproducibility report
- WHEN verification runs with a require-reproducible option
- THEN verification exits non-zero
- AND JSON output reports reproducibility status `absent`

#### Scenario: Reproducibility mismatch stays distinct

- GIVEN a release with valid signatures and matching witness policy
- BUT a reproducibility report records byte drift
- WHEN verification runs
- THEN the output reports the signature and witness statuses separately
- AND reproducibility status is `mismatched`

### Requirement: Witness independence evidence is structured

Mantle MUST derive witness independence evidence from signed witness attestations and detached signature verification results before any witness can count toward independent rebuild agreement.
ID: release.verification.tech.witness.independence.evidence

A countable witness MUST have a trusted signer key name, a witness identity, a rebuild environment summary with host class, rebuilt output digests, and a selected independence-domain value for the active policy selector. Missing, malformed, untrusted, or selector-empty evidence MUST keep the witness out of the counted set and MUST appear in verifier output as skipped or failed with a reason.

#### Scenario: Missing independence field is skipped

- GIVEN a witness whose signature is trusted and rebuilt digests match
- BUT the active independence selector resolves to an empty field
- WHEN agreement verification runs
- THEN the witness does not count toward independent agreement
- AND the report lists it as skipped with a missing-independence-evidence reason

#### Scenario: Malformed environment evidence is rejected

- GIVEN a witness sidecar with malformed rebuild environment summary data
- WHEN agreement verification loads witness material
- THEN verification does not count that witness
- AND the diagnostic names the malformed environment evidence

### Requirement: Canonical independent rebuild agreement report

Mantle MUST define a canonical independent rebuild agreement report that binds one release attestation to the accepted witness attestations, rebuilt output digest sets, witness identities, signer key names, and environment summaries used for agreement.
ID: release.verification.tech.independent.agreement.report

The report identity MUST be the BLAKE3 digest of canonical compact JSON bytes. Witness entries MUST be sorted deterministically and MUST include enough data for a verifier to explain why each witness counted, was skipped, or failed.

#### Scenario: Agreement report digest is stable

- GIVEN the same release attestation and same accepted witness set in different
  filesystem discovery orders
- WHEN the agreement report is serialized
- THEN the canonical bytes are identical
- AND the BLAKE3 report digest is identical

#### Scenario: Skipped witness is explainable

- GIVEN one valid matching witness and one witness with an unknown signing key
- WHEN the agreement report is generated
- THEN the matching witness is counted
- AND the unknown-key witness is listed as skipped with a signature-trust reason

### Requirement: Verifier exposes independent agreement class

Mantle MUST expose JSON field `independent_agreement_status` and class value `independent-rebuild-agreement` when the configured independent rebuild agreement threshold is satisfied.
ID: release.verification.tech.independent.agreement.class

The verifier MUST keep digest matching, signature validity, independence, and policy sufficiency separately visible in JSON output. `independent_agreement_status` MUST be `satisfied` only when the configured policy threshold is met across distinct selected independence domains; otherwise it MUST be `unsatisfied` with diagnostics. The JSON output MUST also include `independent_agreement_class`, `independent_agreement_report_digest`, `independent_agreement_counted_witness_count`, `independent_agreement_skipped_witness_count`, `independent_agreement_failed_witness_count`, and per-witness classification reasons. The class value `independent-rebuild-agreement` MUST NOT replace or obscure existing technical/final classes such as `external-witness-match`; it is an additional agreement class reported beside them. A release MUST NOT be reported as independently agreed when matching witnesses all come from the same configured independence domain.

#### Scenario: Independent witnesses satisfy agreement

- GIVEN a release with valid evidence
- AND two matching witnesses from distinct configured independence domains
- AND policy requiring two independent matching witnesses
- WHEN release verification runs
- THEN the output reports independent rebuild agreement satisfied
- AND it names the agreement report digest

#### Scenario: Same-domain witnesses do not satisfy agreement

- GIVEN a release with two matching witness attestations from the same
  configured independence domain
- WHEN policy requires two independent domains
- THEN verification keeps technical digest matching visible
- AND it reports independent rebuild agreement unsatisfied

### Requirement: Operator docs bound independent agreement claims

Mantle MUST document independent rebuild agreement as policy-scoped evidence from accepted witness files, not as proof of full-source bootstrap, global reproducibility, or public witness discovery.
ID: release.verification.tech.independent.agreement.docs

The docs MUST name the JSON fields `independent_agreement_status`, `independent_agreement_class`, agreement report digest, counted/skipped/failed counts, and witness classification reasons. The docs MUST state that independence metadata is evaluated from signed witness sidecars and verifier-local policy, and stronger identity vetting is outside first-phase file verification.

#### Scenario: Docs avoid overclaiming agreement

- GIVEN operator documentation describes independent rebuild agreement
- WHEN it explains a satisfied agreement
- THEN it ties the claim to configured policy and accepted witness files
- AND it does not claim full-source bootstrap or global reproducibility

### Requirement: Reproducibility proof report

Mantle MUST define a canonical reproducibility proof report that records enough evidence to justify a bounded reproducibility claim for release artifacts.
ID: release.verification.tech.reproducibility.proof

The report MUST record a closed proof class, release identifier, source digest, recipe identity, environment assumptions, clean rebuild store identities when a rebuild is claimed, produced output digest set, comparison verdict, and the evidence artifact digests that were checked before the verdict was emitted. Mantle-owned output comparisons MUST use BLAKE3 unless the report also records an explicit interoperability reason for a non-BLAKE3 digest. The first supported proof classes MUST extend the existing technical release classes and distinguish at least:

- `bundle-consistent`: the release-evidence bundle is internally valid, but no self-proof or rebuild agreement is claimed.
- `self-proof-valid`: the bundled self-proof verifies for the published digest set, but no additional clean rebuild agreement is claimed.
- `self-rebuild-match`: Mantle rebuilt the release artifact from the recorded recipe in a clean local store and matched the published BLAKE3 output digest set.
- `external-witness-match`: at least one accepted witness rebuilt from the exported request and matched the published BLAKE3 output digest set.
- `policy-satisfied`: the accepted witness set satisfies the configured social verification policy.

A report MUST NOT use a stronger proof class than the evidence supports. A mismatch, unsupported recipe, dirty input, missing required digest, reused non-clean rebuild store where a clean store is required, or incomplete witness material MUST produce a failed report or a weaker class rather than silently updating the published digest set. Existing `self-proof-valid` release verification behavior MUST remain valid and MUST NOT be renamed away by the reproducibility report work.

#### Scenario: Local clean rebuild proves self rebuild match

- GIVEN a release-evidence bundle that verifies successfully
- AND the bundle records a supported rebuild recipe identity
- AND Mantle rebuilds the release artifact in a clean local store
- WHEN the rebuilt output BLAKE3 digest set equals the published digest set
- THEN the reproducibility report records proof class `self-rebuild-match`
- AND it records the recipe identity, clean store identity, source digest, published digest set, rebuilt digest set, and comparison verdict

#### Scenario: Independent witness promotes the proof class

- GIVEN a release-evidence bundle that verifies successfully
- AND an accepted witness attestation references the release attestation
- AND the witness rebuilt output BLAKE3 digest set equals the published digest set
- WHEN release verification evaluates the witness material
- THEN the reproducibility report may record proof class `external-witness-match`
- AND it MUST NOT record `policy-satisfied` unless the configured witness policy is also satisfied

#### Scenario: Mismatch fails closed

- GIVEN a clean rebuild or witness rebuild produces a different output BLAKE3 digest set
- WHEN Mantle creates or verifies the reproducibility report
- THEN the report records a failed comparison verdict naming the mismatched output
- AND it does not replace the published digest set with the rebuilt digest set
- AND it does not claim `self-rebuild-match`, `external-witness-match`, or `policy-satisfied`

#### Scenario: Bundle consistency does not over-claim reproducibility

- GIVEN a release-evidence bundle verifies internally
- BUT no clean rebuild or accepted witness rebuild evidence is present
- WHEN Mantle reports the technical proof class
- THEN it may report `bundle-consistent`
- AND it does not claim any rebuild-based reproducibility class

### Requirement: Reproducibility proof recipe is replayable

Mantle MUST make the rebuild recipe used for a reproducibility proof replayable by another environment without inventing ad hoc commands.
ID: release.verification.tech.reproducibility.recipe

The recipe identity MUST bind the command, workflow version, source/bundle inputs, output selection, sandbox expectations, and required environment assumptions. A reproducibility report MUST reject unknown recipe identities unless a compatible implementation explicitly supports them. The first supported recipe MAY be the existing self-hosting proof workflow and exported witness request workflow, but any accepted recipe MUST be versioned and documented.

#### Scenario: Supported recipe can be replayed from evidence

- GIVEN a reproducibility request exported from a verified release-evidence bundle
- AND the request names a supported recipe identity
- WHEN another environment runs the documented witness rebuild entry point
- THEN the rebuilt outputs are selected by the recipe rather than by operator guesswork
- AND the resulting witness material records the replayed recipe identity

#### Scenario: Unknown recipe is rejected

- GIVEN a release-evidence bundle or witness request names an unsupported recipe identity
- WHEN Mantle attempts to create or verify reproducibility evidence
- THEN the command exits non-zero
- AND the diagnostic names the unsupported recipe identity

### Requirement: Reproducibility claim wording remains bounded

Mantle MUST keep docs and machine-readable reports clear about what each reproducibility proof class proves and what it does not prove.
ID: release.verification.tech.reproducibility.claims

Docs and reports MUST distinguish release-artifact reproducibility from full-source bootstrap, cross-platform determinism, global reproducibility, and social verification sufficiency. They MUST name environmental assumptions such as host kernel, CPU architecture, sandbox implementation, and filesystem behavior when those assumptions are outside the proof boundary.

#### Scenario: Docs separate artifact reproducibility from bootstrap claims

- GIVEN a contributor reads the release reproducibility documentation
- WHEN a report reaches `self-rebuild-match` or `external-witness-match`
- THEN the docs explain that the named release artifacts reproduced under recorded assumptions
- AND they do not claim that this alone proves full-source bootstrap or global reproducibility

#### Scenario: Report records assumptions outside the proof boundary

- GIVEN a reproducibility report is emitted
- WHEN it lists its proof boundary
- THEN host/kernel/CPU/sandbox assumptions are recorded as assumptions
- AND they are not counted as source-built inputs or independent witnesses

### Requirement: Release verification rejects impure material for existing proof classes

Release verification MUST treat impure build material as proof-blocking for the
existing reproducibility and deterministic proof classes. If an artifact,
witness rebuild, or deterministic proof receipt records hermeticity mode
`impure`, the verifier MUST fail closed for proof classes that require pure,
practical, or strict evidence.

#### Scenario: Impure release artifact is not reproducible evidence

- GIVEN a release evidence bundle records an artifact built in impure mode
- WHEN release verification evaluates `self-rebuild-match`,
  `external-witness-match`, `policy-satisfied`, or deterministic-release claim
  eligibility
- THEN the verifier does not promote the artifact into those classes
- AND it reports impure execution as the blocker

#### Scenario: Impure witness is rejected for agreement

- GIVEN a witness rebuild attestation records hermeticity mode `impure`
- WHEN release verification evaluates independent witness agreement
- THEN the witness cannot satisfy `external-witness-match`
- AND the verifier reports the witness as skipped or failed due to impure mode

### Requirement: Release verification consumes deterministic proof receipts without overclaiming

Release verification MUST promote a release artifact beyond `self-rebuild-match`
to a stronger deterministic-release claim only when it is backed by
deterministic build proof receipts whose verdicts are `deterministic-match` for
every required artifact output in the release digest set.

A deterministic-release claim MUST remain scoped to the named release artifacts,
workflow identity, derivation identities, toolchain/provider identities, and
recorded proof matrix. It MUST NOT claim global Mantle determinism, all-package
reproducibility, or full-source bootstrap determinism unless separate evidence
proves those broader claims.

#### Scenario: Release has deterministic proof for every artifact

- GIVEN a release evidence bundle with required artifact digests
- AND each required artifact has a deterministic-build proof receipt with verdict
  `deterministic-match`
- AND the receipt BLAKE3 digest set equals the release artifact digest set
- WHEN release verification evaluates deterministic claim eligibility
- THEN it may report a deterministic-release claim scoped to those artifacts

#### Scenario: One artifact lacks deterministic proof

- GIVEN a release with multiple required artifacts
- AND at least one artifact lacks a `deterministic-match` proof receipt
- WHEN release verification evaluates deterministic claim eligibility
- THEN the release remains at the strongest lower satisfied proof class
- AND the report identifies the missing deterministic proof evidence

#### Scenario: Deterministic release claim remains bounded

- GIVEN a release artifact has a valid deterministic-build proof receipt
- WHEN Mantle renders human-readable release verification output
- THEN it states the claim scope as release-artifact determinism under the
  recorded workflow and proof matrix
- AND it does not call the whole build system Nix-like deterministic by default

