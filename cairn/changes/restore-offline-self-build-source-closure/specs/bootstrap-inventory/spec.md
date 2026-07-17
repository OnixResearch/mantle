## ADDED Requirements

### Requirement: Self-build source staging closes over current compile inputs

r[bootstrap_inventory.self_build_source_closure] Mantle MUST validate one explicit checkout-local Cargo directory source against the locked registry/git package graph and Cargo checksum metadata, stage every fixed build-required top-level source root, and require current fixed-point evidence before reporting self-build success.

#### Scenario: Complete vendor input resolves without ambient Cargo state

GIVEN `vendor-deps/` was generated from the current locked graph and `.cargo/vendor-config.toml` selects that directory source
WHEN locked offline Cargo metadata runs with an empty `CARGO_HOME`, offline network policy, and no ambient registry or git cache
THEN every locked registry/git package MUST resolve from the explicit directory source
AND missing packages, stale package checksums, stale file checksums, extra packages, or unsupported source replacements MUST fail closed before bootstrap.

#### Scenario: Staged source includes compile-time policy bytes

GIVEN a tracked workspace crate consumes generated policy bytes from the top-level `config/` root during compilation
WHEN Mantle stages the fixed source tree for self-build
THEN the exact policy bytes MUST be present at the same relative path in staged source
AND unrelated roots such as `target/`, arbitrary scratch files, and private `.pi` content MUST remain excluded.

#### Scenario: Bootstrap target preserves no-clobber publication

GIVEN the bootstrap Rust target uses Linux with libc bindings that do not expose the `renameat2` function symbol
WHEN Mantle compiles and exercises OCI, release, attempt-log, or remote-failure publication
THEN the shared Linux shell MUST invoke the kernel no-replace rename operation without depending on that function binding
AND an existing destination MUST remain unchanged together with the unpublished source.

#### Scenario: Proof encounters an intermediate frontier

GIVEN offline metadata, vendor checksum validation, proof preflight, bootstrap tools, or stage1 compilation succeeds
WHEN a later fixed-point stage fails or its evidence is incomplete
THEN Mantle MUST report the exact current blocker and diagnostics instead of self-build success
AND only a current proof bundle with admitted stage1/stage2 equality MAY support the bounded fixed-point claim.

#### Scenario: Fixed-point evidence remains narrowly scoped

GIVEN the current self-build proof succeeds from the repaired source closure
WHEN operators or maintainers report that result
THEN they MUST identify the proof mode, source/vendor boundary, selected input transport, bundle path, and stage1/stage2 equality evidence
AND they MUST NOT infer compiler correctness, seed trust removal, fresh-clone offline completeness, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.
