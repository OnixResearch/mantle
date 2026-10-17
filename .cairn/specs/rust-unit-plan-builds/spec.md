# Rust Unit Plan Builds Specification

## Purpose

Defines the `rust-unit-plan-builds` capability.

## Requirements

### Requirement: Rust semantics stay in the frontend

r[mantle.rust_unit_plan.frontend_boundary] Mantle MUST build Cargo units through the generic native dynamic-plan path. Rust package, target, feature, and toolchain meaning MUST stay in the Rust-planning frontend. The worker, scheduler, and store MUST NOT gain Rust-specific branches, and the lane MUST NOT use `.drv` output discovery or any store interface inside the sandbox.

#### Scenario: Lowered plan is ordinary

- GIVEN a producer emits a Rust unit plan
- WHEN the worker admits and schedules it
- THEN admission MUST use the same validation, limits, and registration as any other dynamic plan

#### Scenario: Unit reaches for the store

- GIVEN a unit builder that attempts to reach a store daemon socket or write outside its declared outputs
- WHEN the unit runs in the sandbox
- THEN the sandbox MUST provide no store interface
- AND the attempt MUST fail without creating store objects

### Requirement: A sandboxed producer plans the graph

r[mantle.rust_unit_plan.producer] The Rust unit-plan producer MUST run as a sandboxed derivation with network denied and only declared inputs: workspace source, registry sources, Rust toolchain, linker toolchain, planner, and unit helper. It MUST obtain the unit graph from Cargo metadata and the Cargo unit graph under the pinned toolchain, MUST reject unknown unit-graph versions, and MUST write the plan to a declared dynamic-plan output together with an evidence record that binds the lock, unit-graph, toolchain, and plan digests.

#### Scenario: Producer emits a plan

- GIVEN a supported workspace with a lock file and vendored registry sources
- WHEN the producer derivation builds
- THEN it MUST write a plan that the worker admits and an evidence record with the four digests

#### Scenario: Missing registry source

- GIVEN a workspace whose lock references a registry package absent from the declared sources
- WHEN the producer runs offline
- THEN it MUST fail with `unit-plan-oracle-failure` naming the package
- AND it MUST NOT emit a plan

### Requirement: Units are lowered by a pure adapter

r[mantle.rust_unit_plan.lowering] Mantle MUST lower planned unit effects to plan units through a pure adapter. Each unit MUST be input-addressed, MUST use a store-path builder, MUST reference every source, toolchain, helper, and dependency path through a declared input or placeholder, MUST pass `-C metadata` derived from the Rust-plan unit identity, MUST remap every source store path to a stable label, and MUST NOT contain absolute non-store paths. Equivalent planning inputs MUST produce identical plan bytes.

#### Scenario: Equivalent inputs

- GIVEN two producer runs over equivalent workspace facts supplied in different orders
- WHEN both lower their unit effects
- THEN both plans MUST have the same canonical bytes and plan digest

#### Scenario: Host path in an effect

- GIVEN a unit effect whose arguments contain an absolute host path
- WHEN the adapter lowers it
- THEN lowering MUST fail with `unit-plan-host-path` naming the unit
- AND no plan MUST be emitted

### Requirement: Units declare direct dependencies only

r[mantle.rust_unit_plan.dependency_closure] A lowered unit MUST declare only its direct dependency units as inputs. Every library unit output MUST contain a manifest of its direct dependency outputs, so that reference scanning records them and consumer sandboxes contain the transitive dependency closure.

#### Scenario: Binary with a deep dependency closure

- GIVEN a binary whose transitive library closure exceeds the per-unit input limit while its direct dependencies do not
- WHEN the plan is lowered and built
- THEN the binary unit MUST declare only direct dependencies
- AND it MUST link with every transitive library available in its sandbox

#### Scenario: Missing manifest

- GIVEN a library output without the dependency manifest
- WHEN a dependent unit's helper assembles search paths
- THEN the helper MUST fail the unit with a typed error naming the library

### Requirement: Each package has its own source object

r[mantle.rust_unit_plan.per_crate_sources] The producer MUST expose each package source root as a separate dynamic-plan source slice, and each unit MUST reference only its own package's slice. A package whose source content is unchanged MUST keep its slice store path and, when its dependencies are unchanged, its unit derivation paths.

#### Scenario: Edit one workspace package

- GIVEN a built workspace with packages `a` and `b` where `b` depends on `a`
- WHEN only package `b` changes
- THEN package `a`'s units MUST keep their derivation paths and MUST be reused without execution

#### Scenario: Edit outside every package

- GIVEN a change to a workspace file outside every package source root
- WHEN the workspace builds again
- THEN the producer MUST rerun and every Rust unit MUST be reused without execution
- AND a downstream verifier MAY rebuild without causing unchanged Rust units to execute

### Requirement: Units run through one unit helper

r[mantle.rust_unit_plan.unit_helper] Every lowered unit MUST run a single Mantle-built static helper that reads its output paths from the build environment, writes rustc arguments to an argument file, invokes the declared rustc, and writes the dependency manifest. The helper MUST NOT consult the network, host paths, or ambient Cargo state.

#### Scenario: Library unit builds

- GIVEN a lowered library unit whose dependencies are built
- WHEN the helper runs
- THEN it MUST produce the library artifacts and the manifest in `$out`

#### Scenario: Ambient Cargo state

- GIVEN a sandbox environment that contains Cargo configuration variables
- WHEN the helper runs
- THEN it MUST ignore them
- AND its argument file MUST contain only lowered values

### Requirement: Unsupported graphs fail closed

r[mantle.rust_unit_plan.supported_fragment] The lane MUST support library,
binary, proc-macro and build-script compilation units, and bounded sandboxed
build-script execution units with declared compiled-script and target-facts
inputs, for the declared lane triples. It MUST block unbound build-script
execution units, test, doc and check modes, unsupported source kinds,
undeclared triples, and plans beyond dynamic-plan limits with stable blockers.
It MUST NOT fall back to Cargo compilation.

#### Scenario: Build-script execution without its binding

- GIVEN a workspace unit graph containing a build-script execution unit whose
  compiled-script execution binding is absent
- WHEN the adapter lowers it
- THEN it MUST fail with `unit-plan-build-script-run-unsupported` naming the
  package
- AND no plan MUST be emitted

#### Scenario: Unsupported target mode

- GIVEN a unit effect for a test target
- WHEN the adapter lowers it
- THEN it MUST fail with `unit-plan-mode-unsupported` naming the unit

#### Scenario: Plan beyond limits

- GIVEN a unit graph that lowers to more units than the dynamic-plan unit limit
- WHEN the producer lowers it
- THEN it MUST fail with `unit-plan-limit` naming the limit

### Requirement: The lane is opt-in and labeled

r[mantle.rust_unit_plan.evidence_lane] Mantle MUST expose the lane through a Nickel entry point separate from `offlineCargoPackage`. It MUST label lane evidence with class `cargo-unit-graph-dynamic-plan` and project build status `opt-in-project-build-lane`, MUST record the non-claims `not-cargo-free-execution`, `not-full-cargo-compatibility`, `not-compiler-correctness`, `not-release-reproducibility`, and `not-bootstrap-correctness`, and MUST add a `unit_plan` lane to the Rust compatibility surface matrix.

#### Scenario: Evidence and build report for a lane build

- GIVEN a successful lane build
- WHEN the producer publishes its evidence record and Mantle writes the JSON build report
- THEN the producer evidence MUST carry the evidence class, build status, non-claims, and plan digest
- AND the build report MUST carry the accepted plan digest, source-slice admission, and root binding

#### Scenario: Overclaim

- GIVEN a lane evidence record that claims Cargo-free execution
- WHEN evidence validation runs
- THEN it MUST reject the record

### Requirement: Work reduction is measured

r[mantle.rust_unit_plan.work_reduction] The lane MUST record a versioned evidence bundle that compares a fresh build, a one-package edit, an edit outside every package, and a clean client with shared results. The bundle MUST record requested, executed, reused, and invalidated unit counts by unit kind, transferred and reused bytes, and plan digests. Elapsed time MAY be recorded as a diagnostic and MUST NOT be a pass condition.

For the already-supported default-feature two-package app fixture, the lane
MUST compare actual consumer-visible app stdout, stderr, and exit status with
the default `mantle.offlineCargoPackage` lane on the same immutable source,
lockfile, pinned toolchain, target, profile, and declared default feature set.
Both apps MUST receive the same arguments and environment. Parity MUST be
limited to their observed stdout, stderr, and exit under those matched
conditions. Raw executable or store-tree byte equality MUST NOT be required
or reported; unavailable or unsupported comparisons MUST remain unproven.

#### Scenario: Matched supported app behavior

- GIVEN both lanes built the same supported two-package default-feature app
  from matched immutable source, lockfile, toolchain, target, and profile
- WHEN both app executables run with identical arguments and environment
- THEN their actual stdout content, stderr content, and exit status MUST each
  be checked against the same declared fixture expectation and compared
- AND differing observed content or exit status MUST fail parity; if either
  comparison cannot run, parity MUST remain unproven

#### Scenario: Clean client

- GIVEN a publisher's signed unit results and a clean client that trusts the publisher's key
- WHEN the client builds the same workspace
- THEN every unit MUST be reused with zero executions
- AND the bundle MUST record the transferred bytes

#### Scenario: One-package edit

- GIVEN a built workspace and an edit to one leaf package
- WHEN the workspace builds again
- THEN the bundle MUST record executions only for that package's units and their dependents

### Requirement: Target facts come from a shared unit

r[mantle.rust_unit_plan.build_scripts.target_facts] The lane MUST emit one target-description unit per toolchain and target triple that records the compiler's cfg values and host tuple as a typed environment file. Every build-script execution unit for that triple MUST read `TARGET`, `HOST`, and `CARGO_CFG_*` values only from that unit.

#### Scenario: Two build scripts share target facts

- GIVEN two packages with build scripts for one triple
- WHEN the plan is lowered
- THEN both execution units MUST reference the same target-description unit

#### Scenario: Toolchain change

- GIVEN a toolchain change between two builds
- WHEN the plan is lowered again
- THEN the target-description unit MUST have a new derivation path

### Requirement: Build scripts run as units

r[mantle.rust_unit_plan.build_scripts.run_units] Each build-script execution MUST run as a plan unit that executes the compiled script through the unit helper with a declared environment, sets `OUT_DIR` to the unit's `out` output, and writes parsed directives to a `flags` output. A nonzero exit or an `error` directive MUST fail the unit.

#### Scenario: Script generates code

- GIVEN a build script that writes a Rust file to `OUT_DIR` and emits `rustc-cfg`
- WHEN its execution unit and the dependent compilation build
- THEN the compilation MUST read the generated file from `OUT_DIR` and receive the cfg flag

#### Scenario: Script fails

- GIVEN a build script that exits nonzero
- WHEN its execution unit runs
- THEN the unit MUST fail with `build-script-nonzero-exit`
- AND the dependent compilation MUST NOT run

### Requirement: Directives propagate as data

r[mantle.rust_unit_plan.build_scripts.directive_propagation] Dependent compilations MUST receive build-script directives only through argument files and environment files from `flags` outputs. Link arguments MUST reach the final link of every dependent binary, `links` metadata MUST reach dependent build scripts as `DEP_<LINKS>_<KEY>`, and any directive outside the supported set MUST fail the execution unit.

#### Scenario: Links metadata reaches a dependent script

- GIVEN package `a` with `links = "foo"` whose script emits `metadata=include=<path>`
- AND package `b` that depends on `a` and has a build script
- WHEN both execution units run
- THEN `b`'s script MUST see `DEP_FOO_INCLUDE` with that value

#### Scenario: Unknown directive

- GIVEN a build script that emits an unsupported directive
- WHEN its execution unit parses the output
- THEN it MUST fail with `build-script-directive-unsupported` naming the directive

### Requirement: Native inputs are declared data

r[mantle.rust_unit_plan.build_scripts.native_inputs] Native libraries and tools MUST reach build scripts only through a typed Nickel declaration keyed by Cargo package id that names derivations by role and refers to them from environment and PATH values. The producer MUST resolve roles to store paths and add them as unit inputs. A build script MUST NOT see an undeclared native input, and a value that refers to an undeclared role MUST fail evaluation.

#### Scenario: Declared native library

- GIVEN a `-sys` package with a declaration naming a Mantle-built library and its pkg-config directory
- WHEN the lane builds the package
- THEN the execution unit MUST receive the resolved paths
- AND the dependent binary MUST link against the declared library

#### Scenario: Declaration removed

- GIVEN the same package without its declaration
- WHEN its execution unit runs
- THEN the library MUST be absent from the sandbox and the unit MUST fail

### Requirement: Build scripts stay sandboxed

r[mantle.rust_unit_plan.build_scripts.sandbox_boundary] Build-script execution units MUST run without network, host paths, or Cargo. `rerun-if-changed` and `rerun-if-env-changed` directives MUST be recorded in `flags` and MUST NOT affect unit identity or scheduling.

#### Scenario: Script probes the network

- GIVEN a build script that attempts a network connection
- WHEN its execution unit runs
- THEN the attempt MUST fail inside the sandbox

#### Scenario: Rerun directive

- GIVEN a build script that emits `rerun-if-changed=src/x`
- WHEN the unit completes
- THEN the directive MUST appear in `flags`
- AND the unit's identity MUST NOT depend on it

### Requirement: Matrix rows need fixtures

r[mantle.rust_unit_plan.build_scripts.matrix_evidence] The Rust compatibility surface matrix MUST mark build-script, proc-macro with build-script, and `links`/pkg-config rows supported for the `unit_plan` lane only when a named passing fixture exists, and MUST keep them blocked otherwise.

#### Scenario: Row without a fixture

- GIVEN a matrix row marked supported for `unit_plan` without a named fixture
- WHEN the matrix drift rail runs
- THEN it MUST fail naming the row
