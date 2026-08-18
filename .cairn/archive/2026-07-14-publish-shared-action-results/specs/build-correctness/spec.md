# Build Correctness Specification

## Purpose

Define immutable shared action-result records and authoritative reuse admission separately from CAS storage and execution.

## ADDED Requirements

### Requirement: Shared action results are immutable content-bound records
r[build_correctness.shared_action_result_records]

Mantle MUST represent a publishable action result as a versioned immutable record binding the requested action ref, ordered output declarations and object refs, PathInfo refs, action receipt ref, required reference-scan and sandbox/network-policy evidence, producer identity and policy, signatures, publication-policy identity, and bounded non-claims. Mantle-owned record identity MUST use domain-separated BLAKE3. A local derivation-to-output mapping, mutable index row, object-presence fact, or execution success alone MUST NOT constitute an action-result record.

#### Scenario: Admitted result becomes publishable

- GIVEN an action completed and all declared outputs, objects, PathInfo, receipts, scans, policies, and required signatures passed ordinary admission
- WHEN Mantle constructs the action-result record
- THEN it MUST bind those canonical facts to one immutable record ref
- AND the record MAY be published only after every referenced required artifact is durable.

#### Scenario: Incomplete local mapping is not promoted

- GIVEN a local CA derivation mapping names an output path but lacks a complete admitted action receipt, object closure, required signatures, or policy evidence
- WHEN Mantle evaluates it for shared publication
- THEN Mantle MUST reject or retain it as a local advisory hint
- AND it MUST NOT publish or report it as a shared admitted action result.

#### Scenario: Record identity is deterministic

- GIVEN two result records contain equivalent canonical facts in different map, output, signature, or reference traversal orders
- WHEN Mantle canonicalizes them
- THEN both MUST produce the same action-result ref
- AND host paths, publication time, transport order, and mutable index position MUST NOT affect that ref.

### Requirement: Shared action-result admission fails closed
r[build_correctness.shared_action_result_admission]

Mantle MUST treat shared action-result lookup as advisory discovery and MUST admit a candidate only after validating its action ref, output declarations and object refs, object completeness, PathInfo, receipt linkage, signatures, producer policy, sandbox/network policy, reference-scan policy, and requested claim strength. If multiple otherwise admissible candidates for one action ref name differing output object sets, strong reuse MUST fail with bounded conflict evidence rather than select by arrival, source order, or last writer.

#### Scenario: Matching shared result avoids execution

- GIVEN a discovered candidate matches the requested action and every required trust, policy, object, receipt, and scan fact
- WHEN Mantle plans reuse
- THEN it MAY admit the candidate outputs without rerunning the action
- AND the report MUST identify the selected result ref, trust basis, and discovery source.

#### Scenario: Poisoned candidate is rejected

- GIVEN a candidate has a stale action ref, altered output ref, incomplete object tree, missing or invalid signature, unsupported producer, mismatched policy, malformed receipt linkage, or path-only identity
- WHEN Mantle evaluates reuse
- THEN it MUST reject the candidate with deterministic diagnostics
- AND it MUST NOT mutate admitted store state or report a cache hit from that candidate.

#### Scenario: Conflicting admitted results expose nondeterminism

- GIVEN two candidates for the same action ref each pass individual shape and trust checks but name different output object sets
- WHEN strong reuse admission compares the candidate set
- THEN Mantle MUST reject automatic strong reuse with `conflicting-action-results`
- AND bounded evidence MUST identify candidate refs and output-set digests without selecting by source order.
