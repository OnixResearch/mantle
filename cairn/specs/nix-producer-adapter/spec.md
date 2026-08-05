# Nix Producer Adapter Specification

## Purpose

Defines the `nix-producer-adapter` capability.

## Requirements

### Requirement: Versioned backend-neutral producer contract

r[nix_producer_adapter.backend_contract] Mantle MUST define a versioned `nix-producer-v1` contract that every Nix producer backend satisfies. The contract MUST accept bounded expression references, evaluation arguments, a system label, and backend policy, and MUST produce a `.drv` closure directory, the selected root `.drv` identity, and typed producer identity facts. Contract validation, outcome classification, and receipt-input construction MUST be pure cores over in-memory data.

#### Scenario: Backend output satisfies the contract

- **GIVEN** a registered backend and a valid bounded evaluation request
- **WHEN** the backend completes
- **THEN** it MUST return a `.drv` closure directory, the root `.drv` identity, and producer identity facts
- **AND** the identity facts MUST name backend kind, backend version, and source or binary identity

#### Scenario: Backend output misses contract data

- **GIVEN** a backend run that ends without a complete `.drv` closure or without identity facts
- **WHEN** the adapter validates the outcome
- **THEN** it MUST reject the run with a stable contract-violation class
- **AND** it MUST NOT emit partial artifacts

### Requirement: Explicit backend selection

r[nix_producer_adapter.backend_selection] Producer policy MUST name exactly one registered backend per run. Unknown backend kinds, missing backend binaries, and unsupported platforms MUST fail closed before evaluation, and the adapter MUST NOT fall back from a pinned backend to an ambient one.

#### Scenario: Registered backend is selected

- **GIVEN** policy selects a registered backend available on the host
- **WHEN** the adapter plans the run
- **THEN** it MUST bind the backend identity into the request before launch
- **AND** the receipt MUST record the backend that actually ran

#### Scenario: Unknown or unavailable backend

- **GIVEN** policy selects an unknown kind, or a backend whose binary is missing or unsupported on this platform
- **WHEN** admission runs
- **THEN** the adapter MUST reject before evaluation with a stable selection-error class
- **AND** it MUST NOT substitute another backend

### Requirement: Pinned fix backend source

r[nix_producer_adapter.fix_pinned_source] The `fix` backend source tree MUST enter Mantle as a fixed-output source record that binds the upstream repository, the exact revision, and the content hash. The `fix` backend build MUST reference only this record and MUST NOT fetch, clone, or resolve floating refs at build or run time.

#### Scenario: Pinned source is admitted

- **GIVEN** a source record with a declared repository, exact revision, and matching fixed-output hash
- **WHEN** Mantle admits the `fix` source
- **THEN** it MUST bind the record identity into the backend build plan
- **AND** the same record MUST produce the same admitted source on replay

#### Scenario: Source hash mismatches

- **GIVEN** fetched source content whose hash differs from the declared fixed-output hash
- **WHEN** source admission runs
- **THEN** Mantle MUST reject the source before any build step
- **AND** it MUST NOT fall back to a network refetch or a floating revision

### Requirement: Mantle-built fix backend binary

r[nix_producer_adapter.fix_mantle_built_toolchain] The `fix` backend binary MUST build from the pinned source through Mantle's ordinary derivation pipeline with the Zig toolchain and C library dependencies as explicit derivation inputs. The output MUST carry signed PathInfo and an artifact attestation, and the build MUST NOT use ambient host toolchain paths or a host-built `fix` binary. Every toolchain input MUST be receipt-bound through one of the admitted toolchain sources: the pinned upstream binary tarball source record, or signed nixpkgs binary-cache closures admitted through `mantle store pull` with explicit trusted keys.

#### Scenario: Sandboxed build succeeds

- **GIVEN** the pinned `fix` source and declared Zig, libcurl, and libgit2 inputs from admitted toolchain sources
- **WHEN** the `fix` build derivation runs
- **THEN** it MUST produce the `fix` binary inside the Mantle sandbox
- **AND** the output MUST be admitted with signed PathInfo and an artifact attestation

#### Scenario: Ambient host toolchain leaks into the build

- **GIVEN** a build plan that references an undeclared host path or a toolchain input from no admitted source
- **WHEN** plan admission or sandbox setup runs
- **THEN** Mantle MUST reject the plan before execution
- **AND** the failure MUST name the undeclared input class

#### Scenario: Toolchain source is receipt-bound

- **GIVEN** a toolchain input admitted through signed cache substitution
- **WHEN** the build receipt is recorded
- **THEN** the receipt MUST name the toolchain source class and the trusted cache identity
- **AND** it MUST mark the toolchain as a binary trust input, not a source-built compiler

### Requirement: Explicit host-Nix backend

r[nix_producer_adapter.host_nix_backend] The existing host-Nix producer path MUST be addressable as an explicit `host-nix` backend behind the same contract. Receipts for the `host-nix` backend MUST record its binary path and version fact and MUST mark its trust posture as ambient and not reproducible by Mantle.

#### Scenario: Host-Nix backend records ambient posture

- **GIVEN** policy selects the `host-nix` backend and a host Nix is present
- **WHEN** a producer run completes
- **THEN** the receipt MUST record the binary path and version fact
- **AND** it MUST mark the backend trust posture as ambient

#### Scenario: Receipts distinguish backend postures

- **GIVEN** one receipt from the `fix` backend and one from the `host-nix` backend
- **WHEN** the receipts are compared
- **THEN** the `fix` receipt MUST identify a pinned Mantle-built binary
- **AND** the `host-nix` receipt MUST NOT present the ambient binary as Mantle-built

### Requirement: Backends reuse the foreign import ABI

r[nix_producer_adapter.foreign_import_abi_reuse] Every backend MUST emit `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts through the existing direct-`.drv` closure producer path. The adapter MUST NOT add a new import ABI, translation rule, or receipt schema, and MUST NOT make backend-specific data part of the stable artifacts.

#### Scenario: Backend emits standard artifacts

- **GIVEN** any registered backend that produced a `.drv` closure directory
- **WHEN** the adapter processes the directory
- **THEN** it MUST emit artifacts that pass the same validation as existing producer artifacts
- **AND** the artifacts MUST contain no backend-specific required fields

#### Scenario: Same expression through two backends

- **GIVEN** one bounded expression instantiated by the `fix` backend and by the `host-nix` backend
- **WHEN** both artifact sets are compared under the declared comparison policy
- **THEN** root derivation identities and graph structure MUST agree
- **AND** any drift MUST block consumption until reviewed

### Requirement: Evaluation stays in the producer shell

r[nix_producer_adapter.evaluation_boundary] Nix expression evaluation MUST occur only inside producer backend shells before artifact emission. Mantle validation, translation, planning, substitution, and realization MUST NOT invoke any backend, a Nix or Lix executable, or a Nix daemon, and MUST NOT require a backend binary at consumption time.

#### Scenario: Consumption runs without any backend

- **GIVEN** accepted producer artifacts and no backend binary on the host
- **WHEN** validation, translation, and planning run
- **THEN** they MUST complete from the artifacts alone
- **AND** they MUST NOT attempt to locate or launch a backend

#### Scenario: Backend realization command is rejected

- **GIVEN** a backend command that builds, substitutes, or activates (for example `fix build`, `fix run`, or `fix switch`)
- **WHEN** the adapter plans its invocation
- **THEN** it MUST reject that command class
- **AND** it MUST restrict backends to evaluation and instantiation operations

#### Scenario: Instantiation uses the daemon only as a store-write transport

- **GIVEN** a backend instantiation that writes `.drv` files through a reachable daemon
- **WHEN** the adapter collects the output
- **THEN** it MUST copy the concrete `.drv` closure into an owned bounded directory
- **AND** it MUST admit the closure through Mantle's own ATerm parsing, never through daemon build or query operations

### Requirement: Bounded backend execution

r[nix_producer_adapter.bounded_execution] Every backend shell MUST run its evaluator under an explicit bounded process policy with named wall-time, memory, and output-size limits, an owned teardown sequence, and a confined working directory. Evaluation errors, timeouts, malformed output, and limit violations MUST fail closed with stable error classes shared across backends.

#### Scenario: Evaluation exceeds the wall-time budget

- **GIVEN** a backend evaluation that remains active beyond the configured deadline
- **WHEN** the adapter enforces the budget
- **THEN** it MUST terminate, kill, and reap the worker within the named teardown policy
- **AND** it MUST report a timeout class without accepting late output

#### Scenario: Backend output is malformed

- **GIVEN** a backend run whose `.drv` output fails ATerm parsing or closure completeness checks
- **WHEN** the adapter processes the output
- **THEN** it MUST fail closed with the shared malformed-output class
- **AND** it MUST NOT emit partial artifacts

### Requirement: Per-backend bounded compatibility evidence

r[nix_producer_adapter.compatibility_evidence] Compatibility claims MUST be recorded per backend as receipt data that names the backend, its source or binary identity, the reference-Nix version, the language-suite pins, the nixpkgs universe pin, and the agreement counts. Receipts MUST NOT claim general Nix equivalence, package correctness, realization success, or evidence validity beyond the named pins.

#### Scenario: Evidence record is complete

- **GIVEN** a differential run with recorded pins and agreement counts for one backend
- **WHEN** the evidence record is admitted
- **THEN** it MUST bind every pin and count as typed fields
- **AND** the receipt MUST state the exact evidence scope

#### Scenario: Backend pin drifts from the evidence

- **GIVEN** a backend source or binary identity that differs from the recorded evidence pins
- **WHEN** a receipt references the evidence
- **THEN** it MUST mark the evidence stale for the new identity
- **AND** it MUST NOT present the old counts as covering the new identity

### Requirement: Hash domain separation

r[nix_producer_adapter.hash_domain_boundary] The adapter MUST preserve Nix-required hash algorithms for `.drv` identity, store paths, NAR hashes, and NARInfo data, while Mantle-owned receipts, source records, policy digests, and artifact identities use BLAKE3. A digest supplied in the wrong domain MUST fail closed.

#### Scenario: Nix identity keeps its algorithms

- **GIVEN** a backend-produced `.drv` closure with Nix-format store paths
- **WHEN** the adapter emits import artifacts
- **THEN** derivation and store-path identity MUST use the Nix-required algorithms
- **AND** the Mantle receipt identity MUST use BLAKE3

#### Scenario: Cross-domain digest is supplied

- **GIVEN** a BLAKE3 digest supplied where a Nix-compatible derivation identity is required, or the reverse
- **WHEN** the adapter validates identities
- **THEN** it MUST reject the artifact before emission
- **AND** the error MUST name the expected hash domain

### Requirement: Producer adapter validation

r[nix_producer_adapter.validation] The producer adapter work MUST include positive, negative, contract-conformance, backend-parity, budget, evidence, and hash-domain fixtures plus focused adapter, build, and lifecycle checks. Negative fixtures MUST fail for their target boundary, and an unrelated failure MUST NOT count as correct rejection evidence.

#### Scenario: Supported fixture matrix passes

- **GIVEN** fixtures for contract conformance per backend, pinned admission, sandboxed build, backend parity, evidence binding, and clean cancellation
- **WHEN** focused validation runs
- **THEN** every fixture MUST produce its expected terminal class
- **AND** no backend process may remain after the run completes

#### Scenario: Negative fixture hits its target boundary

- **GIVEN** a fixture with an unknown backend, unavailable binary, hash mismatch, floating revision, unsupported platform, evaluation error, malformed output, budget timeout, daemon command, or wrong-domain digest
- **WHEN** focused validation runs
- **THEN** it MUST produce the expected stable error class
- **AND** a failure from an unrelated subsystem MUST NOT satisfy the fixture
