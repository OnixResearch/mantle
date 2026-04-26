# Design: StageX-class bootstrap without quorum

## Context

Crunch already has a normalized seed-provider seam, source-root manifest work,
no-host-tools protected execution, self-hosting proof bundles, release evidence,
reproducibility reports, and witness verification. Those pieces are necessary
but not sufficient for the StageX target. The remaining gap is that current
source-root evidence can still rely on host compiler/build tools as trusted
roots, while StageX roots the chain in a tiny audited seed and stage0 lineage.

## Goals / Non-Goals

**Goals**
- Replace host-compiler trust in the target profile with a declared audited seed
  and source-only stage lineage.
- Produce the existing normalized `bootstrap/seed.ncl` provider through that
  lineage so later bootstrap derivations do not care about raw stage0 layout.
- Bind lineage provider evidence into self-build proof bundles.
- Gate a no-quorum verified-release profile on full-source lineage proof plus a
  verified reproducibility report.

**Non-Goals**
- Social quorum, multi-maintainer signing thresholds, or signer governance.
- OCI-native package management parity with StageX.
- Rebuilding the whole package universe under this profile in the first pass.
- Verifying hardware, firmware, CPU microcode, or kernel source provenance.

## Decisions

### 1. Add a StageX-class lineage manifest instead of overloading source-root trust notes

**Choice:** introduce a stricter lineage contract/profile that rejects host
compiler/build tools as roots. Existing source-root manifests may remain useful
for intermediate evidence, but StageX-class validation has a separate fail-closed
rule set.

**Rationale:** the existing full-source-root work allowed explicit trust notes
for host tools during provider construction. That is honest, but it cannot carry
the StageX-class claim. A stricter profile avoids weakening the old evidence or
overclaiming from it.

**Implementation:** model lineage as a stage graph: seed nodes, source archive
nodes, generated artifact nodes, transition-tool nodes, patches, expected
provider output roles, and environment assumptions. The first closed seed-class
allowlist contains exactly `hex0-seed`; new seed classes require a separate
OpenSpec change and ADR. Required seed audit-bound fields are `seed_class`,
instruction set or bytecode language, entry point, I/O contract, allowed syscall
or host-interface surface, checked-in human-readable source path, source-to-byte
reproduction transcript, manual audit note, `audit_seed_max_bytes`, and seed
digest. The default `audit_seed_max_bytes` budget is exactly 4096 bytes and is
stored in the manifest next to the seed digest, not inferred from docs.
Crunch-owned fingerprints are lowercase BLAKE3 hex digests. A non-BLAKE3 digest
is accepted only when the manifest field `interoperability_reason` is non-empty
and names the upstream format or protocol that requires it. The pure validator
rejects forbidden root kinds, unsupported seed classes, seeds whose byte length
exceeds `audit_seed_max_bytes`, missing seed audit bounds, missing BLAKE3
digests, non-BLAKE3 digests without reasons, and unbound generated artifacts
before any build runs.

**Rejected alternative:** redefine current source-root manifests to disallow all
host tools immediately.

**Why not:** that would break useful intermediate workflows and make migration
harder. The new target profile can be strict without removing lower-maturity
paths.

### 2. Keep the normalized provider boundary as the compatibility seam

**Choice:** the lineage output must normalize to the same provider contract used
by `bootstrap/seed.ncl` today.

**Rationale:** this confines StageX-lineage complexity to the provider layer.
The later chain (`make`, `dash`, `binutils`, `musl`, `gcc`, `busybox`, `bwrap`,
`rust`, `crunch`) already has proof coverage and should not learn about raw
stage0-posix/live-bootstrap paths.

**Implementation:** provider-boundary tests compare required roles rather than
byte-identical internals: target-prefixed tools, headers, libraries, metadata,
retained tools, reduction metadata, and notes. A static audit rejects direct raw
layout reads from later derivations.

**Rejected alternative:** rewrite all bootstrap derivations to build directly on
live-bootstrap's filesystem layout.

**Why not:** that spreads target-specific state across every later stage and
makes the proof harder to audit.

### 3. Treat proof binding as a first-class artifact

**Choice:** self-build proof metadata records exact `provider_kind =
"stagex-lineage"` plus seed digest, lineage manifest digest, stage graph digest,
provider digest, source digest, stage1/stage2 digests, bootstrap-tool digests,
protected-exec audit digest, and proof bundle digest.

**Rationale:** the release verifier needs machine-checkable evidence that the
binary came from the StageX-class path, not just a human-readable log.

**Implementation:** add exact provider kind `stagex-lineage` and a structured
`stagex_lineage` proof block. The canonical proof block fields are:
`seed_class`, `audit_seed_max_bytes`, `seed_digest`,
`lineage_manifest_digest`, `stage_graph_digest`,
`normalized_provider_digest`, `staged_source_digest`, `stage1_crunch_digest`,
`stage2_crunch_digest`, `bootstrap_tool_digests`, optional
`protected_exec_audit_digest`, and `proof_bundle_digest`. Digest fields use
lowercase BLAKE3 hex unless the lineage manifest records an interoperability
reason. Legacy and intermediate providers keep their existing classifications
and cannot satisfy this block.

**Rejected alternative:** infer maturity from successful command names or docs.

**Why not:** command-line success is ambiguous; the evidence must be carried by
canonical proof data.

### 4. Define no-quorum verified-build as a release profile

**Choice:** add a verification profile that requires StageX-class proof plus a
verified reproducibility report, while explicitly marking quorum as out of scope.

**Rationale:** the user wants the technical target now and quorum later. Mixing
these would either block useful progress on social policy or accidentally claim
multi-party trust before it exists.

**Implementation:** add the exact bundle verification entry point
`crunch release verify <bundle-dir> --require-stagex-no-quorum`. In JSON mode it
emits a `stagex_no_quorum` object with fields `status`, `class`,
`quorum_status`, `provider_kind`, `proof_bundle_digest`,
`reproducibility_report_digest`, `release_id`, `artifact_set_digest`, and
`failure_reasons`. Success requires all of these checks:

1. release-evidence bundle verifies;
2. proof bundle contains `provider_kind = "stagex-lineage"` and a valid
   `stagex_lineage` proof block;
3. reproducibility report is canonical and covers the exact published artifact
   set;
4. canonical profile output binds proof bundle digest and reproducibility report
   digest to the same release identifier and artifact set;
5. `status = "satisfied"`, `class = "stagex-verified-no-quorum"`, and
   `quorum_status = "not_evaluated"`;
6. output never emits `quorum-satisfied` for this profile.

**Rejected alternative:** reuse existing witness `quorum-satisfied` final class.

**Why not:** that term is social-policy loaded and belongs to the later quorum
change.

## Risks / Trade-offs

**Large stage0 integration.** Mitigate by landing pure manifest validation and
provider-boundary tests before attempting the long proof.

**Accidental overclaiming.** Mitigate with negative verifier tests for legacy
fetched provider, host-tool-trusted source-root provider, prerequisite-only
proofs, and witness-only agreement.

**Long proof runtime.** Keep smoke fixtures for validator/provider tests, and
reserve the full proof for final acceptance evidence.

**Reference drift.** Pin imported stage0/live-bootstrap sources by digest and
record provenance; do not depend on branch tips.

## Validation Plan

- Pure lineage validator tests for accepted seed lineage and all forbidden root
  classes.
- Provider-boundary tests proving normalized contract satisfaction and no raw
  layout reads by later stages.
- Protected-exec negative tests proving host compiler/build-tool/Nix escapes fail
  before claim evidence is emitted.
- Full self-build proof with the StageX-class lineage provider.
- Release profile tests for satisfied no-quorum evidence and negative cases:
  missing reproducibility report, legacy provider, host-tool-trusted provider,
  self-proof-only, and witness-only agreement.
- Documentation checks proving the profile says quorum is out of scope.
