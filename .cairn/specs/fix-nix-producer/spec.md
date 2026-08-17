# Fix Nix Producer Specification

## Purpose

Defines the `fix-nix-producer` capability.

## Requirements

### Requirement: Pinned fix source record

r[fix_nix_producer.pinned_source] The `fix` source tree MUST enter Mantle as a fixed-output source record that binds the upstream repository, the exact revision, and the content hash. Producer builds and adapter runs MUST reference only this record and MUST NOT fetch, clone, or resolve floating refs at run time.

#### Scenario: Pinned source is admitted

- **GIVEN** a source record with a declared repository, exact revision, and matching fixed-output hash
- **WHEN** Mantle admits the `fix` source
- **THEN** it MUST bind the record identity into the producer build plan
- **AND** the same record MUST produce the same admitted source on replay

#### Scenario: Source hash mismatches

- **GIVEN** fetched source content whose hash differs from the declared fixed-output hash
- **WHEN** source admission runs
- **THEN** Mantle MUST reject the source before any build step
- **AND** it MUST NOT fall back to a network refetch or a floating revision

### Requirement: Mantle-built fix toolchain

r[fix_nix_producer.mantle_built_toolchain] The `fix` binary MUST build from the pinned source through Mantle's ordinary derivation pipeline with the Zig toolchain and C library dependencies as explicit derivation inputs. The output MUST carry signed PathInfo and an artifact attestation, and the build MUST NOT use a host `zig`, host C libraries, or a host-built `fix` binary.

#### Scenario: Sandboxed build succeeds

- **GIVEN** the pinned `fix` source, a pinned Zig toolchain derivation, and declared libcurl, libgit2, and pkg-config inputs
- **WHEN** the `fix` build derivation runs
- **THEN** it MUST produce the `fix` binary inside the Mantle sandbox
- **AND** the output MUST be admitted with signed PathInfo and an artifact attestation

#### Scenario: Host toolchain leaks into the build

- **GIVEN** a build plan that references a host `zig` path or an undeclared library
- **WHEN** plan admission or sandbox setup runs
- **THEN** Mantle MUST reject the plan before execution
- **AND** the failure MUST name the undeclared input class

### Requirement: Producer adapter reuses the foreign import ABI

r[fix_nix_producer.producer_adapter] The `fix` producer adapter MUST emit `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts through the existing direct-`.drv` closure producer path. The adapter MUST NOT add a new import ABI, translation rule, or receipt schema, and MUST NOT make `fix`-specific data part of the stable artifacts.

#### Scenario: Adapter emits standard artifacts

- **GIVEN** a Nix expression that `fix` instantiates into a `.drv` closure directory
- **WHEN** the adapter processes the directory
- **THEN** it MUST emit artifacts that pass the same validation as host-Nix-produced artifacts
- **AND** the artifacts MUST contain no `fix`-specific required fields

#### Scenario: Same expression through both producers

- **GIVEN** one bounded expression instantiated by `fix` and by the host-Nix producer
- **WHEN** both artifact sets are compared under the declared comparison policy
- **THEN** root derivation identities and graph structure MUST agree
- **AND** any drift MUST block consumption until reviewed

### Requirement: Evaluation stays in the producer shell

r[fix_nix_producer.evaluation_boundary] Nix expression evaluation with `fix` MUST occur only inside the producer shell before artifact emission. Mantle validation, translation, planning, substitution, and realization MUST NOT invoke `fix`, a Nix or Lix executable, or a Nix daemon, and MUST NOT require the `fix` binary at consumption time.

#### Scenario: Consumption runs without fix

- **GIVEN** accepted `fix`-produced artifacts and no `fix` binary on the host
- **WHEN** validation, translation, and planning run
- **THEN** they MUST complete from the artifacts alone
- **AND** they MUST NOT attempt to locate or launch `fix`

#### Scenario: Adapter requires a daemon

- **GIVEN** a `fix` command that requires a reachable Nix or Lix daemon
- **WHEN** the producer adapter plans its invocation
- **THEN** it MUST reject that command class
- **AND** it MUST restrict itself to evaluation and instantiation operations

### Requirement: Bounded producer execution

r[fix_nix_producer.bounded_execution] The adapter MUST run `fix` under an explicit bounded process policy with named wall-time, memory, and output-size limits, an owned teardown sequence, and a confined working directory. Evaluation errors, timeouts, malformed output, and limit violations MUST fail closed with stable error classes.

#### Scenario: Evaluation exceeds the wall-time budget

- **GIVEN** a `fix` evaluation that remains active beyond the configured deadline
- **WHEN** the adapter enforces the budget
- **THEN** it MUST terminate, kill, and reap the worker within the named teardown policy
- **AND** it MUST report a timeout class without accepting late output

#### Scenario: Producer output is malformed

- **GIVEN** a `fix` run whose `.drv` output fails ATerm parsing or closure completeness checks
- **WHEN** the adapter processes the output
- **THEN** it MUST fail closed with a stable malformed-output class
- **AND** it MUST NOT emit partial artifacts

### Requirement: Bounded compatibility evidence

r[fix_nix_producer.compatibility_evidence] Compatibility claims MUST be recorded as receipt data that names the `fix` revision, the reference-Nix version, the language-suite pins, the nixpkgs universe pin, and the agreement counts. Receipts MUST NOT claim general Nix equivalence, package correctness, realization success, or evidence validity beyond the named pins.

#### Scenario: Evidence record is complete

- **GIVEN** a differential run with recorded pins and agreement counts
- **WHEN** the evidence record is admitted
- **THEN** it MUST bind every pin and count as typed fields
- **AND** the receipt MUST state the exact evidence scope

#### Scenario: Pins drift from the evidence

- **GIVEN** a `fix` or nixpkgs pin that differs from the recorded evidence pins
- **WHEN** a receipt references the evidence
- **THEN** it MUST mark the evidence stale for the new pins
- **AND** it MUST NOT present the old counts as covering the new pins

### Requirement: Hash domain separation

r[fix_nix_producer.hash_domain_boundary] The adapter MUST preserve Nix-required hash algorithms for `.drv` identity, store paths, NAR hashes, and NARInfo data, while Mantle-owned receipts, source records, policy digests, and artifact identities use BLAKE3. A digest supplied in the wrong domain MUST fail closed.

#### Scenario: Nix identity keeps its algorithms

- **GIVEN** a `fix`-produced `.drv` closure with Nix-format store paths
- **WHEN** the adapter emits import artifacts
- **THEN** derivation and store-path identity MUST use the Nix-required algorithms
- **AND** the Mantle receipt identity MUST use BLAKE3

#### Scenario: Cross-domain digest is supplied

- **GIVEN** a BLAKE3 digest supplied where a Nix-compatible derivation identity is required, or the reverse
- **WHEN** the adapter validates identities
- **THEN** it MUST reject the artifact before emission
- **AND** the error MUST name the expected hash domain

### Requirement: Fix producer validation

r[fix_nix_producer.validation] The `fix` producer work MUST include positive, negative, parity, budget, evidence, and hash-domain fixtures plus focused adapter, build, and lifecycle checks. Negative fixtures MUST fail for their target boundary, and an unrelated failure MUST NOT count as correct rejection evidence.

#### Scenario: Supported fixture matrix passes

- **GIVEN** fixtures for pinned admission, sandboxed build, instantiation, producer parity, evidence binding, and clean cancellation
- **WHEN** focused validation runs
- **THEN** every fixture MUST produce its expected terminal class
- **AND** no `fix` process may remain after the run completes

#### Scenario: Negative fixture hits its target boundary

- **GIVEN** a fixture with a hash mismatch, floating revision, unsupported platform, evaluation error, malformed output, budget timeout, or wrong-domain digest
- **WHEN** focused validation runs
- **THEN** it MUST produce the expected stable error class
- **AND** a failure from an unrelated subsystem MUST NOT satisfy the fixture
