## ADDED Requirements

### Requirement: Cargo-free fixed-point command

r[rust_package_planning.cargo_free_fixed_point_command] Mantle MUST provide a first-class command that runs the bounded Cargo-free fixed-point proof without requiring an external proof script or caller-created rustc wrapper.

#### Scenario: command runs both fixed-point stages

GIVEN an operator launches `mantle self-build --cargo-free --fixed-point --out <bundle-dir>` from a supported Mantle source root
WHEN Mantle performs the proof
THEN it MUST build stage1 Mantle through the native Cargo-free topology rail
AND it MUST build stage2 Mantle by invoking the produced stage1 Mantle binary through the same native Cargo-free topology rail.

#### Scenario: command forbids Cargo in both stages

GIVEN the fixed-point command is running
WHEN either stage attempts to invoke Cargo for metadata, unit graph discovery, planning, or build orchestration
THEN the stage-local Cargo guard MUST record the attempt
AND the command MUST fail without claiming a Cargo-free fixed point.

#### Scenario: command owns toolchain compatibility

GIVEN the selected rustc/linker combination needs compatibility handling for Mantle topology execution
WHEN the fixed-point command prepares a stage
THEN it MUST either use reviewed command-owned normalization recorded in the proof bundle
OR fail closed with an actionable toolchain diagnostic
AND it MUST NOT require the caller to provide an untracked external rustc wrapper.

#### Scenario: command emits durable fixed-point evidence

GIVEN the fixed-point command completes, blocks, or detects a mismatch
WHEN the bundle directory is inspected
THEN it MUST contain preflight metadata, per-stage receipts, command streams, status codes, Cargo guard status, smoke outputs, copied Mantle binary paths when produced, per-stage BLAKE3 digests, fixed-point status, and non-claims.

#### Scenario: command reports success only for matching stage binaries

GIVEN both stages produce Mantle binaries
WHEN the fixed-point command compares outputs
THEN it MUST report success only when the stage1 and stage2 Mantle binary BLAKE3 digests match
AND it MUST report a deterministic mismatch status when the digests differ.

#### Scenario: command remains a bounded proof

GIVEN the fixed-point command reports success
WHEN the summary is reviewed
THEN it MUST state that the result is not Crunch bootstrap, release reproducibility, source-built compiler/toolchain closure, or full Cargo compatibility evidence.
