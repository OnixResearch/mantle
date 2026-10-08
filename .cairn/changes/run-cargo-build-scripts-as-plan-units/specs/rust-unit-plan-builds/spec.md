# Specification: Rust unit-plan build scripts

## ADDED Requirements

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
