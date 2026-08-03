# Design: Verify remote admission with Trellis

## Context

`crates/crunch-build/src/distributed/remote_attempt.rs` defines the functional core for assignment, fencing, event classification, phase transitions, retry, authorization, and report application. The implementation uses finite enums and bounded maps, which makes a small formal model practical.

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
