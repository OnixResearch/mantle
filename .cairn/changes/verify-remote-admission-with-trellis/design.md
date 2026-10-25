# Design: Verify remote admission with Trellis

## Context

`crates/crunch-remote-core/src/attempt.rs` defines the functional core for assignment, fencing, event classification, phase transitions, retry, authorization, and report application. The implementation uses finite enums and bounded maps, which makes a small formal model practical.

Mantle already accepts Valence-authorized Trellis proof evidence as an opaque release sidecar. It must keep that evidence path separate from runtime decisions.

## Decisions

### Decision: Put the generic proof model in Trellis

**Choice:** A paired Trellis change will add a product-neutral `fenced_attempt_admission` module. The model will use small explicit state and event enums, fixed-width identities, a fence generation, event identity/digest facts, result linkage, and authorization facts.

The Trellis module will provide `spec fn`, `exec fn`, `ensures`, and named proof functions. It will not import Mantle types or perform I/O.

**Rationale:** Trellis owns reusable verified logic and verifier evidence. Mantle remains the owner of remote-build semantics and adapter correctness.

### Decision: Prove a closed safety set

**Choice:** The first proof set covers:

- a report with stale, future, wrong-job, or wrong-attempt fence facts cannot be applied;
- terminal attempts reject later state-changing reports;
- the same event identity and digest is an idempotent no-op;
- one event identity with a different digest is rejected;
- completion is reachable only after a result-ready state with matching result identity and output-admission authority;
- reassignment strictly advances a nonzero fence when representable;
- every rejected decision preserves the modeled state.

Retry timing, transport progress, storage commits, and liveness remain outside this proof set.

**Rationale:** These properties protect durable state and output admission and map directly to the current pure core.

### Decision: Use an explicit Mantle refinement projection

**Choice:** Mantle will implement a pure projection from admitted `RemoteAttemptState`, report, and authorization facts into the Trellis model. A finite parity rail will compare Mantle decisions with the Trellis executable model for every phase and report-kind pair plus bounded fence, event, digest, linkage, and authorization cases.

Unsupported or newly added Mantle variants fail the projection until the model and proof are updated.

**Rationale:** A proof over a disconnected model gives no useful implementation evidence. The projection and parity rail make model drift visible without claiming source equivalence.

### Decision: Keep proof evidence external to runtime authority

**Choice:** Trellis exports the proof artifact. Kamacite preserves its canonical envelope. Valence validates role, identity, assumptions, and non-claims. Mantle binds the accepted sidecar through its existing opaque evidence mechanism.

Ordinary remote admission continues to use Mantle's Rust core. A proof sidecar does not authorize a report or output.

**Rationale:** Proof evidence supports review. It must not become an ambient runtime trust root.

### Decision: Pin cross-repository identity without relative product paths

**Choice:** Development can use the sibling `../trellis` checkout. Durable evidence and CI must name a Trellis revision, source digest, verifier/toolchain identity, requirement IDs, proof artifact digest, and Valence receipt. Mantle will not depend on a workspace-relative path at runtime.

**Rationale:** Sibling paths are local workspace conveniences, not portable product identity.

## Functional core and imperative shell

Trellis owns the verified pure model. Mantle owns a pure projection and executable parity logic. Repository checkout, verifier execution, artifact export, Kamacite conversion, Valence validation, and evidence storage remain shell operations.

## Risks and trade-offs

- A simplified model can omit a decisive field. The projection must reject unmapped variants and record modeled fields explicitly.
- Verus proofs can pass while Mantle drifts. The parity rail and source-digest evidence must fail on drift.
- Cross-repository sequencing can stall. Each change records the exact required revision and owner.
- Proof evidence can be overstated. Reports must retain the closed safety set and explicit non-claims.

## Claim boundary

The accepted result proves only the named properties of the abstract fenced-attempt model and the tested projection cases at recorded revisions. It does not prove Mantle implementation equivalence, persistence atomicity, transport reliability, cryptographic correctness, remote worker correctness, liveness, or release eligibility.

## Current projection and evidence handoff (2026-10-23)

The production projection is `crunch-remote-core::trellis_projection`. It maps
job, attempt and result identities through separate bounded full-string equality
domains, retains nonzero `u64` fences, exhaustively maps all seven phases and
seven report kinds, and carries complete retained event-history classification,
event count, worker/output authority and exact retained/result report linkage.
It recomputes the canonical payload digest before projection. Heartbeat and
checkpoint progress outside the admitted modeled class fail with
`UnmodeledPayload`; they never receive proof coverage. The context is retained
for an entire trace so result or identity changes cannot disappear by renumbering.

The parity fixture calls the exact imported Trellis `admit`, not a handwritten
copy of its algorithm. Its finite classes cover every phase/kind pair, current,
stale, future, wrong-job and wrong-attempt identities, all four authorization
classes, absent/matching/different result linkage, and new/replayed/conflicting
events against ordinary, nearly full and full retained history. Empty history
is also covered, with only its representable new-event class. Additional traces
cover first and final nonzero fences, current-fence completion without fence
increment, and consumer rejection of reassignment overflow. The producer's
representable-fence theorem is arithmetic, not an extra admission transition.

The current sibling Trellis sources were inspected rather than assuming the
producer is absent. The imported source remains pinned to
`d91188e495d65bca87551b7a06330efb2c909edf`. A current `b3sum` observation found
the imported and sibling kernel/test bytes identical:

| Artifact | BLAKE3 |
| --- | --- |
| `verified.rs` | `ceb717657f092747a03501e63a0f2a8aeda7af0aba226e21de725334d47c0549` |
| `tests/verified_positive.rs` | `1b91fa5e9b6f551a570806a26a6f0870374aa1dda947788f56ecdd03a2934a4a` |
| `tests/verified_negative.rs` | `391047a21cec9a8100fb1f6e6535297e0e63296d5f2f90fd610d48e1f2feaefd` |

The sibling's current `docs/proof-evidence-bundle.json` has BLAKE3
`fd5dfaeaf481a0bc0d39bf4fa6f31b643e64d8369ce7126c7ff7e58f53240be3`;
its `docs/proof-evidence-ir-chain.json` has BLAKE3
`306084e00e37c268af278bf29254bb6378bac6944c0246febda9928b02613db4`.
They include `module:fenced_attempt_admission::verified`, the exact pinned
kernel digest, six named proof functions, local `passed` status, proof IR
`1ba9d11cf0dcbbf9f2a98395da3535498fec7d11861bae35b2c7b3540bd836aa`,
source handoff
`1e8a63ec024f7c8c15b4c52b937bdde13cb932e767571418f70077402b0426a5`,
and an explicit non-claim that this is not a Valence property receipt. The
archived 2026-09-03 producer verification describes an older model; its reported
counts are not fresh verification of the pinned oracle or current Mantle source.

Searching those exact proof/source identities in the current sibling Kamacite,
Valence and Mantle proof fixtures found no exported matching envelope or
accepted property receipt. The remaining external prerequisites are therefore
not missing model source: they are current pinned-source verifier observations
and assumptions/toolchain identity, a real canonical Kamacite v2 envelope for
that exact handoff, and Valence acceptance of that candidate under the property
role and accepted policy. Synthetic sidecar fixtures must not fill these gaps.

Before reporting acceptance, run the importer identity check and current Mantle
parity/negative rails, capture Mantle source, fixture, policy and toolchain
BLAKE3 identities, rerun the producer's recorded Verus command against the exact
source, then export and validate the real handoff through Kamacite/Valence and
bind the measured envelope and distinct validation/logical-receipt identities.
A mapping, policy, source, assumption or role change invalidates the old receipt.
No build, test or proof was run for this handoff, and no external proof
acceptance or lifecycle gate is asserted by these observations.
