## ADDED Requirements

### Requirement: Cargo-free Rust fixed-point proof

r[rust_package_planning.cargo_free_fixed_point_proof] Mantle MUST provide audit-grade evidence for a bounded Cargo-free fixed-point proof where host Mantle builds stage1 Mantle, stage1 Mantle builds stage2 Mantle, and the produced stage1/stage2 Mantle binary digests are compared.

#### Scenario: fixed-point proof forbids Cargo in both stages

GIVEN the Cargo-free fixed-point proof is launched
WHEN stage1 and stage2 Mantle builds are executed
THEN each stage MUST replace Cargo with a failing guard
AND any attempted Cargo metadata, unit-graph, or build-orchestration invocation MUST fail the proof.

#### Scenario: stage2 is built by stage1 Mantle

GIVEN stage1 Mantle was produced by a successful Cargo-free topology execution
WHEN the fixed-point proof starts stage2
THEN stage2 MUST be planned and executed by the produced stage1 Mantle binary
AND stage2 MUST use the same Cargo-free native topology rail rather than the original host Mantle binary.

#### Scenario: fixed-point proof emits durable evidence

GIVEN the Cargo-free fixed-point proof completes, blocks, or mismatches
WHEN the proof exits
THEN it MUST write durable preflight metadata, per-stage receipts, command streams, status codes, Cargo guard status, smoke outputs, produced binary paths, BLAKE3 digests, fixed-point status, and non-claims to an audit bundle.

#### Scenario: fixed-point proof compares stage binaries

GIVEN both stage1 and stage2 builds succeed and produce Mantle binaries
WHEN final outputs are inspected
THEN the proof MUST compare stage1 and stage2 Mantle binary BLAKE3 digests
AND it MUST report success only when those digests match.

#### Scenario: fixed-point proof remains bounded

GIVEN the fixed-point proof reports success
WHEN the audit bundle is reviewed
THEN the bundle MUST state that the proof is not Crunch bootstrap, release reproducibility, source-built compiler/toolchain closure, or full Cargo compatibility evidence.
