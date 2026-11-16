# 0156. Adopt only verified cold StageX outputs for native rows

Status: Accepted

## Context

The source-built cold proof retains canonical StageX transition and provider
exports with action-producer evidence. Native bootstrap recipes instead consume
two reserved bare StorePaths. A completed cold proof does not publish signed
PathInfo for those names. The older development-only construction path performs
its own legacy transition and adoption; its cached outputs or reports cannot
stand in for current cold execution.

The immediate consumers are the five native parity rows: `binutils.tcc`,
`gcc.4.0`, `gcc.4.7`, `gcc.10`, and `full-musl-binutils`. Their old qualification
loop omits explicit early roots and does not produce every required current
row-local receipt. Signing a directory establishes neither its cold producer
nor a native behavior-matrix result.

## Decision Drivers

- Preserve complete source/producer/action reconciliation and exact cohort identity.
- Do not introduce provider inputs, legacy fallback or store adoption into the cold proof.
- Reuse existing receipt verification, bounded tree capabilities and signed store admission.
- Keep the reviewed source/runner of an active attempt immutable.
- Qualify all five current rows with actual observations, not refreshed historical JSON.
- Make failure and partial publication distinguishable from complete adoption.

## Decision

Provide an operator-consumer library boundary and separate workspace binary
targets rather than another proof-execution route or a new main CLI variant.
Build the operator targets with Mantle's existing lockfile and workspace
patches. Independent `cargo -Zscript` manifests cannot use the vendored Git
cohort without their own lockfiles; do not fetch another dependency cohort or
duplicate the lockfile to make the entrypoints work. Adoption requires
a complete reverified cold v2 receipt, independently selected semantic plan and
raw receipt identities, exact producer-bound StageX exports, explicit existing
signing authority, and fresh nonoverlapping local store/state destinations.
Development or partial attempts cannot pass this boundary.

Place both Rust entrypoints in the existing `tools/` tree. Every declared
Cargo target must be present in the staged source set; `scripts/` is deliberately
excluded. Move the sources rather than broadening source capture or narrowing
target admission. The checker has a separate compatibility-family decision
for its original-validator projections, not a new aggregate receipt schema.

The versioned adoption report is an explicit compatibility surface in the
machine-artifact inventory, not an internal-only DTO. Its Rust owner and
receipt-bound consumer checks govern the handoff; classification does not
add it to the curated generated-schema cohort or confer proof/store authority.

The canonical transition export contains original and intermediate artifacts in
addition to runtime tools. Copy each complete canonical export into an owned
bounded snapshot and compare its exact receipt-bound artifact identity before
projecting runtime members. A source-only pre/post check around a selective copy
is insufficient against an ABA substitution. Use existing no-follow bounded-tree
capabilities and the verifier's canonical artifact measurement; do not introduce
another tree-identity framing or forge legacy plan/report/audit files.

Project the existing eight transition runtime directories and require their
nine tools. Publish the canonical provider tree separately. The projection and
reserved logical identities have one pure owner shared with the existing
construction code. Existing default copy limits and proof admission rules do
not change; the operator snapshot has explicit bounded geometry suitable for
the verifier's complete export inventory.

Validate both staged results before signed admission. Use narrow source-admission
capabilities, strict local backend policy, selected signing keys and current
content/readback checks. Emit a successful adoption receipt only after both
reserved paths are admitted and verified. A partial effect is not an atomic-pair
claim or permission to start native qualification.

For Snix, output, state and report paths are absent. Casita retains its existing
operator-owned trust model: state may contain only a preprovisioned regular
`casita-trusted-public-keys` file. Adoption neither writes that policy nor copies
private key material. Open the selected backend before retaining private
snapshots so they do not appear as pre-existing, unrecorded backend content.
Both current NAR bytes and canonical artifact identities are checked on signed
readback; a matching signature alone is insufficient.

The native-row operator then checks that complete adoption, current source/profile
and signing context, runs all five source-selected rows without substitution,
and gathers every original validator's source, predecessor, regeneration,
artifact/NAR, runtime, rejection, relocation, fallback, acceptance and trust
observation. New evidence is validated against explicit current source without
editing the immutable producer checkout. Historical receipts guide schema only;
missing observations fail rather than becoming passed defaults.

## Consequences

- Mantle's bootstrap/proof operator workflow owns this capability. The immediate
  outcome is a supported provenance-bound path from completed cold exports to
  fresh signed native-row inputs; the durable contribution is a repeatable
  adoption/qualification procedure, not a one-run path copy.
- Adoption is not new provider construction, a fixed point, native-row acceptance,
  compiler correctness or release eligibility. Each remains separately evidenced.
- The active frozen `869a461d` attempt keeps its original source and runner.
  New Rust/operator source is not retroactively covered by that receipt. Final
  changed-source promotion requires its own published freeze, isolated runner
  and canonical qualification, source-program regeneration and complete review.
- Bounded positive and negative checks establish the operator mechanisms only;
  genuine completed-proof adoption and all five real native rows remain required
  before their corresponding acceptance claims.
