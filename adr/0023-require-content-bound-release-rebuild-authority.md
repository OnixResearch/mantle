# ADR 0023: Require content-bound authority for release rebuild proofs

## Status

Accepted

## Context

Mantle's deterministic release receipt previously bound paths, sandbox profiles,
and matching output digests, but a rebuild command could still read the published
binary and copy its bytes into each fresh output root. Repeated equality therefore
did not prove that the artifact was rebuilt from source.

A stronger release claim needs one reviewable authority boundary shared by proof
execution and every admission consumer. It must distinguish exact approved
inputs from the complete release bundle, published targets, aliases, prior proof
outputs, and ordinary reproduction outputs.

## Decision Drivers

- Make matching output digests necessary but insufficient for release admission.
- Bind source, recipe, executable, ordered arguments, tools, provider, policies,
  selected targets, and fresh roots by content identity.
- Reject byte-identical aliases, hardlink aliases, symlinks, undeclared inputs,
  reused roots, and read/write overlap before proof execution.
- Keep policy and classification in a pure bounded core with a thin filesystem and
  sandbox shell.
- Apply one fail-closed rule to release verification, standalone checking,
  summaries, Nix witnesses, and bootstrap-parity evidence.
- Preserve legacy receipt parsing without preserving legacy promotion authority.

## Decision

Mantle promotes only `mantle-deterministic-proof-receipt-v2`. The receipt embeds
a canonical `mantle-content-bound-rebuild-descriptor-v1` and
`mantle-rebuild-authority-plan-v1`, plus their BLAKE3 digests. Every run cites
those digests, records the approved read identities it observed, and reports any
authority violation.

The descriptor binds published target identities for comparison but does not
grant them read authority. Approved reads are limited to exact regular-file
source, recipe, executable, tool, and provider capabilities plus synthetic
sandbox, effect, and normalization policy identities. Each proof run gets fresh
output and store roots. The shell materializes approved files under the proof
root, verifies copied content, mounts only those files read-only, and omits the
complete release bundle.

The pure authority planner rejects target-identical content, target filesystem
object aliases, symlinks, unsupported input kinds, whole-bundle inputs, published
targets, prior or ordinary outputs, undeclared inputs, reused roots, and read/write
overlap. Version 1 receipts remain parseable diagnostics but always classify as
missing genuine rebuild evidence.

Bootstrap parity treats accepted genuine release evidence as partial evidence
only. Missing, legacy, or incomplete evidence blocks that row; accepted release
evidence does not complete Guix or StageX parity.

## Alternatives Considered

### Trust two matching fresh output directories

Rejected because both runs can copy the same published target and match without
performing a source rebuild.

### Scan recipe text without capability-scoped execution

Rejected because text scanning catches obvious bundle/target references but does
not define or enforce the complete read-authority boundary.

### Mount the full release bundle read-only

Rejected because read-only target bytes are still target-byte authority. A proof
must omit the bundle and mount only exact approved capabilities.

### Make v1 stricter in place

Rejected because existing v1 receipts do not carry the identities needed to
prove the stronger claim. Reinterpreting them would silently change published
evidence semantics.

## Consequences

- Production proof operators must supply an explicit content-bound toolchain
  closure and a reviewed source-build recipe.
- Proof receipts and summaries are larger but independently reviewable.
- Inputs that are convenient but not exact regular-file capabilities fail closed.
- Legacy evidence remains useful for diagnosis but cannot promote release,
  witness, checker, summary, or bootstrap-parity claims.
- The bounded claim is artifact equality under the recorded authority plan. It
  does not prove compiler or verifier soundness, full-bootstrap reproducibility,
  or global determinism.
