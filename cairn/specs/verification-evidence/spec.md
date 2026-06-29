# Verification Evidence Specification

## Purpose

Defines Mantle's proof-before-claim evidence policy for status, completion, feature-support, and validation claims.

## Requirements

### Requirement: Proof before claim

r[verification_evidence.proof_before_claim] Mantle work MUST NOT claim status, completion, feature support, pass/fail validation, or build success unless current evidence has been produced or inspected before the claim is made.

#### Scenario: claim includes current evidence

- GIVEN an agent, task, evidence file, commit message, status reply, or final summary states that Mantle functionality works, validation passed, work is complete, or the tree is clean
- WHEN that claim is made
- THEN the claim MUST name or include current evidence such as command output, Cairn evidence file, task transcript, build report, test log, VCS status output, or inspected artifact content.
- AND the claim MUST be narrower than or equal to the evidence being cited.

#### Scenario: evidence is absent

- GIVEN current evidence has not been produced or inspected for a potential claim
- WHEN status is reported
- THEN the report MUST say the claim is not proven or state only the narrower fact that is supported by available evidence.
- AND it MUST NOT generalize from mock tests, stale logs, prior sessions, or uninspected artifacts to end-to-end support.

#### Scenario: Cairn task completion is evidence-gated

- GIVEN a Cairn task is marked complete
- WHEN the task records its completion summary
- THEN the task MUST reference durable evidence or an oracle checkpoint that supports the completed task text.
- AND review or gate summaries MUST reject completion claims whose evidence proves only a narrower behavior.

#### Scenario: post-archive validation claim is archived

- GIVEN a change has been archived
- WHEN a commit message, evidence summary, status reply, or final response claims post-archive Cairn validation passed
- THEN the archived change evidence transcript MUST include the exact post-archive validation command and output.
- AND the claim MUST NOT rely only on pre-archive gates or unstored chat transcript output.

### Requirement: Tracey coverage readiness

r[verification_evidence.tracey_coverage_readiness] Mantle MUST maintain deterministic coverage traceability for accepted Cairn requirements before presenting release-readiness or archive-readiness claims that depend on Tracey coverage.

#### Scenario: accepted requirements have traceability disposition

GIVEN an accepted requirement exists under `cairn/specs/`
WHEN Tracey coverage readiness is evaluated
THEN the requirement MUST have either implementation/verification references, an evidence-backed bridge reference, or an explicit tracked debt disposition.
AND comment-only bridge references MUST cite inspected implementation paths or durable evidence before being counted as satisfying traceability.

#### Scenario: coverage failures stay bounded

GIVEN `cairn tracey coverage --root . --json` reports missing requirements
WHEN an agent or operator reports readiness status
THEN the report MUST include the exact coverage validity, referenced count, missing count, dangling count, and next missing group.
AND it MUST NOT claim global Tracey coverage is green unless the command reports `valid: true`.

### Requirement: Spec admission claims require current evidence [r[verification_evidence.spec_admission_proof_before_claim]]

Mantle MUST NOT claim that a frontend artifact kind is spec-admitted, deployable, or supported unless current evidence shows the artifact manifest validated against the declared frontend spec and the build report or receipt contains the matching validation attestation.

#### Scenario: Supported frontend artifact claim cites attestation [r[verification_evidence.spec_admission_proof_before_claim.scenario.claim]]

GIVEN a task, evidence file, release note, status reply, or docs page claims support for a frontend artifact kind
WHEN the claim is made
THEN it MUST cite current build-report or receipt evidence containing spec id, version, hash, validator identity, artifact ref, validation result, and provenance
AND the claim MUST be no broader than the inspected spec-admitted artifact evidence.

#### Scenario: Missing attestation blocks support claim [r[verification_evidence.spec_admission_proof_before_claim.scenario.missing]]

GIVEN a build produced an artifact ref but no spec-validation attestation
WHEN status is reported or a Cairn task is considered complete
THEN Mantle MUST NOT claim the artifact kind is spec-admitted or deployable
AND the report MUST state that spec admission evidence is missing.

### Requirement: Admitted artifact export claims require current evidence [r[verification_evidence.admitted_artifact_export_proof_before_claim]]

Mantle MUST NOT claim that a frontend artifact can be exported, transferred, deployed, or used as a supported deploy handoff unless current evidence shows the artifact was spec-admitted and the export result preserved matching artifact identity, content digest, spec proof, and provenance.

#### Scenario: Export support claim cites receipt evidence [r[verification_evidence.admitted_artifact_export_proof_before_claim.scenario.claim]]

GIVEN a task, evidence file, release note, status reply, or documentation page claims admitted-artifact export support
WHEN the claim is made
THEN the claim MUST cite current build-report, receipt, sidecar, or command evidence containing artifact ref, content digest, spec id, spec version, spec hash, validation result, and provenance
AND the claim MUST be no broader than the inspected artifact export evidence.

#### Scenario: Missing export evidence blocks transfer claims [r[verification_evidence.admitted_artifact_export_proof_before_claim.scenario.missing]]

GIVEN an artifact has a ref or spec-admission attestation but has not been exported through the admitted-artifact boundary
WHEN status is reported or a Cairn task is considered complete
THEN Mantle MUST NOT claim deploy transfer or artifact export support
AND the report MUST state that admitted-artifact export evidence is missing.

### Requirement: Build correctness receipts [r[verification_evidence.build_correctness_receipts]]

Mantle MUST emit deterministic receipts for action-correct build claims. A receipt MUST bind action ref, Nickel evaluation receipt ref when the action was produced from Mantle `.ncl`, input object refs, toolchain refs, produced object refs, reference scan ref, sandbox report ref, network policy result, producer identity, signature refs when present, execution status, and build-or-reuse reason. Human and JSON output MUST keep claims bounded to the exact action/object evidence present.

#### Scenario: Successful action receipt is complete [r[verification_evidence.build_correctness_receipts.scenario.success]]

- GIVEN Mantle executes an action under enforced policy and admits produced CAS objects
- WHEN it emits a build correctness receipt
- THEN the receipt MUST include action ref, Nickel evaluation receipt ref when applicable, input refs, toolchain refs, produced object refs, reference scan ref, sandbox report ref, producer identity, and execution status
- AND the receipt MUST be deterministic for equivalent declared inputs and outputs

#### Scenario: Reuse receipt names trust basis [r[verification_evidence.build_correctness_receipts.scenario.reuse]]

- GIVEN Mantle accepts a reused or substituted output
- WHEN it emits a build correctness receipt
- THEN the receipt MUST name the prior receipt or signature material that justified reuse
- AND the receipt MUST explain the matched action ref, object refs, policy, and producer trust basis

#### Scenario: Evidence does not overclaim [r[verification_evidence.build_correctness_receipts.scenario.non-goals]]

- GIVEN Mantle emits a successful action-correct receipt
- WHEN human or JSON evidence is rendered
- THEN the evidence MAY claim the produced objects match the declared action and receipt policy
- AND it MUST NOT claim compiler correctness, source-to-binary reproducibility, full bootstrap correctness, frontend module correctness, deploy success, or physical-target determinism unless separate evidence exists

#### Scenario: Secret material is redacted [r[verification_evidence.build_correctness_receipts.scenario.secret-redaction]]

- GIVEN a build action or output involves secret descriptors
- WHEN Mantle emits receipts, reports, logs, or diagnostics
- THEN it MUST include only redacted descriptors or encrypted/object refs
- AND it MUST NOT include decrypted secret bytes or inline plaintext secret content

### Requirement: Release reproducibility evidence transcripts

r[verification_evidence.release_reproducibility_transcripts] Mantle MUST record durable, tracked evidence before claiming deterministic-release or release-reproducibility proof status for a specific release evidence bundle.

#### Scenario: transcript binds proof commands and digests

GIVEN a release evidence bundle is described as reproducible, deterministic-release eligible, or verified with a deterministic-release proof
WHEN the claim is recorded in a task, evidence file, release summary, or status reply
THEN a tracked evidence transcript MUST cite current command output for the reproduce command, required verifier command, and receipt checker command.
AND the transcript MUST record the release id, reproducibility report digest, deterministic proof digest, sandbox isolation evidence digest, rebuilt artifact digest set, and any provider fixed-point proof status used by the claim.

#### Scenario: generated proof payloads stay untracked

GIVEN deterministic release proof receipts, rebuild outputs, or release evidence bundles are produced under generated artifact directories
WHEN durable evidence is committed
THEN Mantle MUST commit concise lifecycle evidence text rather than generated proof payloads.
AND tracked evidence MUST identify the generated paths and digests without staging ignored `target/` artifacts.

#### Scenario: absent transcript blocks release proof claims

GIVEN a deterministic release proof was run only in an ephemeral working directory or chat transcript
WHEN Mantle reports release-readiness, archive-readiness, or completed release evidence status
THEN the report MUST say the deterministic-release claim is not durably recorded.
AND it MUST NOT present ephemeral generated artifacts as tracked release evidence until a lifecycle transcript exists.

### Requirement: Portable release verification replay

r[verification_evidence.portable_release_verification_replay] Mantle MUST provide durable evidence when a release verification claim is intended to be replayable from copied release artifacts rather than the original source checkout.

#### Scenario: copied artifact set verifies successfully

GIVEN a release evidence bundle and required deterministic-release proof sidecars are copied to a fresh scratch or export directory
WHEN an operator runs release verification from that copied artifact set with required deterministic-release and provider fixed-point gates enabled
THEN verification MUST succeed using only the copied release bundle, copied deterministic proof receipt, copied sandbox isolation evidence, and bundle-local provider proof material.
AND the evidence transcript MUST record the scratch root, command, verifier status, proof digests, and bounded non-claims.

#### Scenario: missing copied proof material fails closed

GIVEN the copied artifact set omits a deterministic proof receipt or sandbox isolation evidence file required by the verification command
WHEN the operator runs verification with `--require-deterministic-release`
THEN Mantle MUST fail closed with a deterministic diagnostic rather than reporting deterministic-release eligibility.
AND the failure evidence MUST show the missing proof class without weakening the requested gate.

#### Scenario: portable replay avoids ambient source claims

GIVEN release verification succeeds from a copied artifact set
WHEN the result is reported as portable evidence
THEN the claim MUST be limited to replaying the recorded release proof material from copied artifacts.
AND it MUST NOT claim source checkout cleanliness, full bootstrap reproducibility, compiler correctness, full Cargo compatibility, or deploy success unless separate evidence proves those claims.

### Requirement: Provider-bound release evidence transcripts

r[verification_evidence.provider_bound_release_evidence_transcripts] Mantle MUST record durable evidence before claiming provider-bound deterministic release evidence for a specific release evidence bundle.

#### Scenario: transcript binds provider proof to a release artifact

GIVEN a release evidence bundle is described as provider-bound, deterministic-release eligible, or verified with provider fixed-point proof evidence
WHEN the claim is recorded in a task, evidence file, release summary, or status reply
THEN a tracked evidence transcript MUST cite current command output showing the provider proof status is valid and the provider proof matched a release artifact relative path and BLAKE3 digest.
AND the transcript MUST record the release id, matched provider artifact path, matched provider artifact digest, provider proof digest, deterministic proof digest, sandbox isolation evidence digest, and verifier status.

#### Scenario: portable replay proves copied artifact sufficiency

GIVEN provider-bound release evidence is intended to be replayable outside the original bundle path
WHEN portable replay evidence is recorded
THEN the transcript MUST cite current positive verification from copied artifacts with deterministic-release and provider fixed-point gates enabled.
AND it MUST cite a negative replay showing missing required proof material fails closed instead of reporting eligibility.

#### Scenario: provider-bound release evidence stays bounded

GIVEN provider-bound deterministic release verification succeeds
WHEN the evidence is summarized
THEN the summary MUST state that the claim is limited to the recorded packaged artifact set and proof inputs.
AND it MUST NOT claim full bootstrap reproducibility, compiler correctness, deploy success, or full Cargo compatibility unless separate evidence proves those claims.

### Requirement: Refreshed provider-bound release evidence transcripts

r[verification_evidence.provider_bound_release_evidence_refresh_transcripts] Mantle MUST record durable refreshed evidence before claiming a newer current-code provider fixed-point proof is packaged as provider-bound deterministic release evidence.

#### Scenario: refreshed transcript binds current provider proof to a release artifact

GIVEN a prior provider-bound release evidence transcript recorded a current-code provider proof blocker
WHEN a newer provider fixed-point proof succeeds and is packaged into a release evidence bundle
THEN a tracked evidence transcript MUST cite current command output showing the provider proof status is valid and the provider proof matched a release artifact relative path and BLAKE3 digest.
AND the transcript MUST record the release id, matched provider artifact path, matched provider artifact digest, provider proof digest, deterministic proof digest, sandbox isolation evidence digest, and verifier status.

#### Scenario: refreshed portable replay proves copied artifact sufficiency

GIVEN refreshed provider-bound release evidence is intended to supersede a prior blocker note
WHEN portable replay evidence is recorded
THEN the transcript MUST cite current positive verification from copied artifacts with deterministic-release and provider fixed-point gates enabled.
AND it MUST cite a negative replay showing missing required proof material fails closed instead of reporting eligibility.

#### Scenario: refreshed evidence stays bounded

GIVEN refreshed provider-bound deterministic release verification succeeds
WHEN the evidence is summarized
THEN the summary MUST state that the claim is limited to the recorded packaged artifact set and proof inputs.
AND it MUST NOT claim full bootstrap reproducibility, compiler correctness, deploy success, or full Cargo compatibility unless separate evidence proves those claims.

### Requirement: Release witness rebuild binds all signed outputs

r[verification_evidence.release_witness_rebuild_multi_output] Mantle MUST make `mantle release witness-rebuild` bind every published release output in a witness attestation to a rebuilt proof artifact whose BLAKE3 digest matches the corresponding release output. Multi-output release requests MUST be supported when the workflow proof bundle and any workflow-produced provider fixed-point proof bundle contain trustworthy artifacts for every expected digest, and the command MUST fail closed before signing witness sidecars when any expected output is missing, ambiguous, path-escaping, invalid, or digest-mismatched.

#### Scenario: multi-output witness rebuild signs matching proof artifacts

GIVEN a witness request for a release whose signed release attestation names multiple published binary digests
AND the completed witness workflow proof bundle contains rebuilt proof artifacts whose BLAKE3 digests match every published binary digest
WHEN `mantle release witness-rebuild` completes
THEN the command MUST create one witness attestation whose rebuilt digest set covers every published binary in release order
AND it MUST write audit metadata naming the rebuilt proof artifact path and digest for each signed output.

#### Scenario: provider-bound witness rebuild uses witness-produced provider proof

GIVEN a witness request for a provider-bound release whose signed release attestation names a provider fixed-point binary digest
AND the witness workflow produces a provider fixed-point proof bundle under the witness-owned provider proof directory
WHEN `mantle release witness-rebuild` evaluates rebuilt outputs
THEN the command MUST validate the provider proof bundle before accepting its stage binary as a rebuilt output candidate
AND the provider candidate MUST be selected only when its BLAKE3 digest matches the signed provider release artifact.

#### Scenario: incomplete proof artifacts fail closed

GIVEN a witness request for a release whose signed release attestation names multiple published binary digests
AND the completed witness workflow proof bundle and provider proof bundle lack a trustworthy rebuilt artifact for at least one published digest
WHEN `mantle release witness-rebuild` evaluates rebuilt outputs
THEN the command MUST exit non-zero before writing a witness attestation
AND the diagnostic MUST identify the missing or mismatched expected output.

#### Scenario: escaping proof artifact paths fail closed

GIVEN a witness workflow proof manifest or provider proof metadata names a rebuilt artifact path
AND that path escapes its witness-owned proof bundle root
WHEN `mantle release witness-rebuild` evaluates rebuilt outputs
THEN the command MUST exit non-zero before signing witness material
AND the diagnostic MUST identify the path containment failure.

### Requirement: Release source archives preserve native source identity [r[verification_evidence.release_source_archive_native_parity]]

Mantle release evidence source archives MUST preserve the tracked source files that native path-source hashing observes for provider-bound release proofs, while excluding private runtime and lifecycle evidence paths that native source hashing skips.

#### Scenario: tracked package source is replayable

GIVEN `mantle release create` packages a release source archive from a checkout
WHEN a witness extracts the source archive and runs a provider fixed-point replay
THEN the extracted tree MUST contain tracked package-root files, scripts, docs, tests, and verified vendored source files visible to native path-source hashing
AND the archive MUST NOT reduce the source tree to only the self-build staging allowlist.

#### Scenario: private and skipped paths stay out of source archives

GIVEN the checkout contains untracked files, ignored runtime output directories, private signing material, or root Cairn lifecycle evidence
WHEN `mantle release create` packages a release source archive
THEN the archive MUST exclude those paths before copying source material into a release bundle
AND exclusion MUST apply even if a private runtime path such as `target/` or `.pi/` is force-tracked.

### Requirement: Provider fixed-point replay normalizes local path identity [r[verification_evidence.provider_fixed_point_path_normalization]]

Provider-bound release witness replay MUST compile provider fixed-point stages with deterministic source, execution-output, provider helper, and receipt-bound C compiler toolchain path identity so binary digest mismatches identify source, toolchain, or build output differences instead of publisher/witness scratch path differences.

#### Scenario: deterministic path mode is receipt-visible

GIVEN `mantle release witness-rebuild` runs a provider fixed-point proof for a provider-bound request
WHEN the witness proof invokes native rust-plan execution
THEN the rust-plan receipt MUST record deterministic release path mode
AND rustc arguments MUST include stable remap prefixes for the source root and execution-output root.

#### Scenario: provider helper paths are not baked into release binaries

GIVEN the publisher and witness use equivalent source-built Rust provider closures at different filesystem paths
WHEN provider fixed-point stages compile crates that read compile-time provider helper environment
THEN the compile-time environment MUST use deterministic placeholder identity for provider helper paths
AND the released binary MUST NOT depend on the publisher or witness provider scratch path.

#### Scenario: native C compiler paths are remapped deterministically

GIVEN the selected receipt-bound C compiler route points inside a local source-built toolchain root
WHEN provider fixed-point stages compile native C or assembly inputs through build-script-driven toolchains
THEN build-script child environments MUST add C prefix-map flags for the source root, execution-output root, and selected C compiler toolchain root
AND the C compiler toolchain root remap MUST take precedence over broader source-root remaps in emitted debug/source identity.

#### Scenario: build scripts still access real package roots

GIVEN deterministic release path mode is enabled for provider fixed-point replay
WHEN a build script runs during native topology execution
THEN the build script process MUST still execute from the real package root and write to the real OUT_DIR
AND deterministic path remapping MUST NOT replace filesystem paths that the build script needs for I/O.

### Requirement: Self-hosting witness replay normalizes bootstrap and generated path identity [r[verification_evidence.self_hosting_witness_replay_path_normalization]]

Mantle release witness rebuilds that run the self-hosting proof workflow MUST normalize bootstrap tool paths, Cargo target paths, build-script output paths, and staged source paths before those paths can affect Cargo fingerprints, rustc diagnostics, generated-code source spans, or final release binary bytes. The witness MUST still compare every rebuilt published output by exact BLAKE3 digest and MUST fail closed before signing when normalization is incomplete or rebuilt bytes differ.

#### Scenario: stable bootstrap aliases drive Cargo fingerprints

GIVEN a self-hosting proof uses equivalent admitted bootstrap tool objects whose content-addressed paths differ between publisher and witness machines
WHEN the generated self-build script configures Cargo, linker flags, and compile-time helper environment
THEN it MUST use stable in-sandbox aliases for bootstrap tool paths in Cargo-visible values
AND the proof manifest MUST still record the real tool object refs and real content-addressed paths used behind those aliases.

#### Scenario: generated Cargo paths are remapped before rustc embeds them

GIVEN Cargo invokes rustc for a crate whose build script produced generated Rust under `OUT_DIR`
WHEN the self-hosting proof runs in deterministic witness mode
THEN rustc arguments MUST remap `CARGO_TARGET_DIR`, crate `OUT_DIR`, staged source roots, and stable bootstrap aliases to deterministic logical prefixes
AND final release binaries MUST NOT contain host-specific generated paths such as `/tmp/cargo-target/.../build/<crate>-<host-dependent-hash>/out`.

#### Scenario: build scripts keep real filesystem paths

GIVEN deterministic witness mode remaps path identity at the rustc boundary
WHEN a build script reads checked-in package files, invokes helper tools, or writes generated outputs
THEN the build script MUST still run with real package roots and real `OUT_DIR` values for filesystem I/O
AND path normalization MUST NOT replace paths that the build script must open or create.

#### Scenario: exact witness acceptance is preserved

GIVEN a witness rebuild produces a self-hosting proof bundle whose stage artifact differs from a published release output
WHEN `mantle release witness-rebuild` evaluates rebuilt outputs
THEN the command MUST exit non-zero before writing witness sidecars even if the binary matches after stripping symbols or other post-processing
AND the audit metadata MUST report the expected digest, rebuilt digest, self-hosting fixed-point status, provider proof status when present, and any detected bootstrap/path-normalization divergence.

#### Scenario: separate-machine replay evidence stays bounded

GIVEN Aspen or another separate machine successfully rebuilds all published release outputs after self-hosting normalization
WHEN the result is summarized as witness evidence
THEN the summary MUST record witness identity, host class, release id, output digests, provider proof status, self-hosting stage digests, and final verification status
AND the claim MUST remain limited to the exact release artifact set and proof inputs unless separate evidence proves broader compiler or bootstrap correctness.
