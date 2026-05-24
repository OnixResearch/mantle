### Requirement: Native Rust package and target planning fragment

r[rust_package_planning.native_package_target_planning] Mantle MUST compute a bounded native Rust package/target planning fragment for supported simple packages before using Cargo oracle material as replacement-planner truth.

#### Scenario: Supported simple package target facts are computed by Mantle

r[rust_package_planning.native_package_target_planning.supported]

- GIVEN a Rust workspace package uses supported manifest-local `lib` and `bin` targets with readable source files and bounded default feature selection
- WHEN `mantle rust-plan` computes native package/target planning facts
- THEN Mantle MUST emit package identity, manifest path, target names, target kinds, crate names, source paths, selected feature facts, and path-source closure linkage computed by Mantle-owned logic.
- AND Mantle MUST NOT treat Cargo metadata as the source of truth for those emitted native planner facts.

#### Scenario: Mantle-computed fragment compares against Cargo oracle

r[rust_package_planning.native_package_target_planning.compare]

- GIVEN Cargo oracle metadata is available for the same root, profile, target triple, and feature selection
- WHEN Mantle emits the native package/target planning fragment
- THEN Mantle MUST compare the Mantle-computed package identity, manifest path, target facts, selected feature facts, and path-source closure linkage against the retained Cargo oracle.
- AND Mantle MUST include the comparison status in reviewable receipt material before any Cargo-replacement planning claim is made.

### Requirement: Native Rust package and target planning blockers

r[rust_package_planning.native_package_target_planning.blockers] Mantle MUST fail closed with deterministic blockers when the native package/target planning fragment is unsupported, incomplete, or mismatches Cargo oracle material.

#### Scenario: Unsupported package planning surfaces block before replacement claims

r[rust_package_planning.native_package_target_planning.blockers.unsupported]

- GIVEN a workspace uses unsupported target kinds, unsupported feature/workspace inheritance surfaces, build-script or proc-macro planning requirements outside this fragment, unreadable manifests or source paths, or missing source-closure material
- WHEN Mantle evaluates the native package/target planning fragment
- THEN Mantle MUST emit a deterministic blocker identifying the unsupported or missing material class.
- AND Mantle MUST NOT silently fall back to hidden Cargo planning or claim native planner success for that package.

#### Scenario: Cargo oracle mismatch blocks deterministically

r[rust_package_planning.native_package_target_planning.blockers.mismatch]

- GIVEN Mantle-computed package/target planning facts differ from Cargo oracle facts for the same supported package and selection
- WHEN Mantle compares the fragment with the Cargo oracle
- THEN Mantle MUST emit a deterministic mismatch blocker identifying the divergent field class.
- AND Mantle MUST NOT use the Cargo value to repair the Mantle-computed fragment without recording the mismatch as a blocker.

### Requirement: Native Rust package and target planning receipts

r[rust_package_planning.native_package_target_planning.receipts] Mantle MUST expose native package/target planning fragment evidence and blockers in `rust-plan` receipts.

#### Scenario: Receipt preserves oracle and native fragment evidence

r[rust_package_planning.native_package_target_planning.receipts.fragment]

- GIVEN `mantle rust-plan` is run on a workspace evaluated by the native package/target planning fragment
- WHEN Mantle emits JSON receipt material
- THEN the receipt MUST include the retained Cargo oracle digest or identity, the Mantle-computed native fragment facts, comparison status, blockers when present, and a stable receipt hash.
- AND the receipt MUST preserve bounded non-claims that this fragment is not full Cargo feature resolution, full Cargo compatibility, or Cargo-free build scheduling.

#### Scenario: Focused tests cover supported and blocked fragments

r[rust_package_planning.native_package_target_planning.tests]

- GIVEN focused Rust-plan fixtures exercise a supported simple package and unsupported or mismatched package surfaces
- WHEN the relevant `rust_plan` tests run
- THEN the supported fixture MUST prove the Mantle-computed fragment matches Cargo oracle facts.
- AND the unsupported or mismatched fixtures MUST prove deterministic blocker classes without invoking hidden Cargo planning as a repair path.

#### Scenario: Verification closes the native fragment slice

r[rust_package_planning.native_package_target_planning.verify]

- GIVEN the native package/target planning fragment implementation and fixtures are complete
- WHEN the change is verified
- THEN focused Rust-plan tests, Cairn validation, and Cairn proposal/design/tasks gates MUST pass before the change is synced and archived.
