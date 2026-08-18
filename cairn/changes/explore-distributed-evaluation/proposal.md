# Change: Explore distributed evaluation feasibility

## Why

Mantle distributes realization but not evaluation. Remote workers must receive concrete build inputs that the client already evaluated or lowered. They never evaluate Nickel source.

The client-evaluates boundary is deliberate and recorded in accepted specs, ADRs, and the portable-client design. The same recordings do not answer whether evaluation itself can be distributed safely later.

The question is open and worth a bounded assessment because several seams already exist: isolated per-root worker sessions, a strict same-binary eval worker with a framed protocol, streaming eval workers, a pure evaluation-budget core, content-addressed source staging, and the remote-build data plane.

A successful assessment must decide, from deterministic evidence, whether any route can move evaluation authority across a trusted boundary without breaking streaming overlap, dynamic goals, source identity, or the existing concrete-input contract.

## What Changes

- Inventory the evaluation architecture seams and bind each to its source path, accepted spec, or ADR identity.
- Analyze exactly two candidate routes:
  - eval-as-a-service: transport the existing strict eval-worker request and response over a bounded transport with content-addressed source staging;
  - evaluate-once producer: keep the trusted-client boundary and distribute evaluation in time through exported concrete graphs, following the `nickel-export-core` and `mantlepkgs` pattern.
- Record the open blocker surface: streaming eval→build overlap, dynamic goals, evaluator suspension, import staging, and the worker authority boundary.
- Add a bounded probe that serializes an isolated eval-worker request across one socket transport and verifies the response, proving whether the worker boundary carries hidden host-local state.
- Emit a deterministic feasibility report, a pure classifier, an oracle checkpoint, and an ADR.
- Change no evaluation, remote-build, scheduler, provider, or publication behavior.

## Non-Goals

- Wiring distributed evaluation into the product or scheduler.
- Changing who evaluates in the current build path.
- Adding a second evaluator or claiming evaluator equivalence.
- Implementing evaluator suspension or import-from-derivation.
- Claiming distributed evaluation, remote-build correctness, evaluator correctness, or release eligibility from this assessment alone.

## Impact

- **Files:** one pure Rust assessment core and probe shell under `src/`, focused fixtures, lifecycle evidence, one ADR, and the README reference list if an external source is consumed.
- **Testing:** positive and negative classifier fixtures, a bounded eval round-trip probe, and Cairn proposal/design/tasks gates.
- **Specs:** new accepted `distributed-evaluation` spec describing the evidence-gated assessment capability.
