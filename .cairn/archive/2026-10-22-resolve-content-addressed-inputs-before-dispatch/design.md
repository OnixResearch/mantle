# Design: Resolve content-addressed inputs before dispatch

## Goal and scope

Derivations with content-addressed (CA) inputs are resolved to concrete inputs
before dispatch, and the resolved form becomes their cache and sharing
identity. Derivations without CA inputs are unaffected.

## Prechange behavior

The isolated prechange worktree and executable are recorded in
`evidence/baseline-2026-10-01.md`. In that version:

- CA derivations have outputs without precomputed paths. The worker records
  the realized path per unresolved derivation path and output.
- Dependents' outputs are rewritten from provisional to final CA paths after
  their build.
- `action_ref_for_derivation` domain-hashes the unresolved ATerm, and
  PathInfo cache checks use the derivation path plus CA mappings.
- Native plan placeholders require statically known dependency paths, while
  plan units may be content-addressed.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Status quo | Key everything on unresolved derivations | Rejected: no early cutoff | None |
| Content-keyed cache outside the graph | Compiler-driver cache keyed by content, as in `extend-compile-cache-to-cc` | Complementary, not a replacement: no derivation authority or remote dispatch | None |
| Resolved derivations | Substitute realized paths, key on the resolved derivation, record realisations | Selected | Early-cutoff and identity-parity fixtures |
| Deferred output paths | Make every dependent's output path unknown until resolution | Rejected: changes identities of derivations without CA inputs | None |

## Contract and component ownership

- **Pure resolution core**: input is a derivation plus admitted realisation
  facts for direct CA outputs and any previously CA-resolved intermediates;
  output is a resolved derivation or a typed failure. Substitution replaces
  provisional paths with realized paths in arguments and environment, moves
  admitted inputs from input derivations to input sources, and leaves
  CA-independent input-addressed edges unchanged. Resolution is deterministic
  and idempotent.
- **Pure identity**: the resolved identity is the Mantle BLAKE3 domain hash of
  the resolved derivation's canonical bytes. For a derivation in a
  CA-independent subgraph, resolution returns the derivation unchanged.
- **Shell** (worker, orchestrator, store): collects realisation facts after
  inputs complete, runs resolution before cache lookup, looks up and admits
  realisation records, dispatches the resolved derivation, persists signed
  realisation records, and reports both identities.

## Decisions

### Decision: Resolve at dispatch, not at evaluation

**Choice:** Resolution runs after every CA input of a goal completes and
before that goal's cache lookup and dispatch.

**Rationale:** Realized paths exist only after inputs build. Evaluation-time
identities stay stable for scheduling, reports, and waiters.

### Decision: Resolved identity keys CA-dependent work only

**Choice:** Cache lookup, CA mappings, and action refs use the resolved
identity for derivations with a direct or transitive CA prerequisite.

**Rationale:** A CA-resolved intermediate carries an admitted signed output
fact to its children. Resolution does not change CA-independent derivations,
so their existing identities and golden fixtures stay valid.

### Decision: Realisations are signed records

**Choice:** A realisation binds resolved identity, output name, output path,
and signer. It is admitted under the trusted-key policy used for PathInfo,
matching full key material.

**Rationale:** A realisation authorizes reuse. Unsigned or name-only trust
would let a mapping redirect a dependent to arbitrary content.

### Decision: Resolved builds do not use post-build input rewriting

**Choice:** A resolved derivation references realized input paths directly, so
post-build input rewriting no longer applies to it. The rewrite path is
removed once no unresolved CA-input build remains.

**Rationale:** Inputs are concrete before the build, so rewriting output bytes
afterwards is unnecessary, and one mechanism per concept stays reviewable.

## Failure behavior and ordering

Resolution failures are stable: `ca-input-unrealized`,
`ca-realisation-conflict`, `ca-realisation-untrusted`, and
`ca-resolution-limit`. A conflict between admissible realisations blocks reuse
and is recorded as nondeterminism evidence, as shared Rust unit results
already do. Realisation facts are applied in canonical input order.

## Tests

- Positive: early cutoff across a CA producer, input-addressed child, and
  input-addressed grandchild with zero dependent executions; a resolved build
  reused by a clean client through signed realisations; a plan unit that
  references a CA unit output through a placeholder.
- Negative: an unrealized input, conflicting realisations, unsigned and
  untrusted realisations (including a replaced intermediate record), and a
  wrong-domain identity.
- Compatibility: derivation path, output path, and action-ref goldens for
  CA-independent derivations, and existing CA fixtures.

## Risks / Trade-offs

- Resolution touches the identity path of every CA-dependent build. Golden
  fixtures and the unchanged-derivation property limit the blast radius.
- Realisation records add store state, but do not act as garbage-collection
  roots; reuse still requires available signed PathInfo and complete content.
- Early cutoff depends on deterministic CA outputs. Nondeterministic outputs
  reduce reuse but do not make it unsound, because reuse requires an admitted
  record for the exact resolved identity.

## Claim boundary

Resolution evidence proves recorded realisation facts, resolved identities,
and reuse decisions. It does not prove output determinism, compiler
correctness, or release eligibility.
