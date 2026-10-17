# Proposal: Run Cargo build scripts as plan units

## Why

Most real crate graphs contain build scripts; `proc-macro2`, `serde`, and
`libc` each ship one. The Rust unit-plan lane
(`build-cargo-units-as-dynamic-plans`) blocks every build-script execution
unit, so it covers only small workspaces until build scripts run as units.

A build script needs a described target (`TARGET`, `HOST`, `CARGO_CFG_*`),
package facts (`CARGO_PKG_*`, `CARGO_FEATURE_*`), a writable `OUT_DIR`, and
sometimes a C toolchain and native libraries. Its `cargo:` and `cargo::`
directives change dependent compilations: cfg flags, environment variables,
link libraries and search paths, and `links` metadata that reaches dependent
build scripts as `DEP_<LINKS>_<KEY>`.

`rust-plan` already parses directives into `BuildScriptMetadataSummary`
(`src/rust_plan.rs:787-797`) and builds build-script environments, but it runs
the scripts on the host (`src/rust_plan.rs:15066-15099`). The offline Cargo
lane blocks `links`, pkg-config, and native C metadata
(`examples/rust_compatibility_surface_matrix.ncl:199-203`). The reviewed
`cargo-dyndrv` reference handles build scripts with three helpers
(`target-env`, `build-wrap`, and `env-wrap`), rustc argument files, and an
`extern.json` of per-package native inputs whose store references it derives
with `exportReferencesGraph` (`evidence/cargo-dyndrv-review.md`).

## What Changes

- Add target-description units: one per toolchain and triple, recording the
  compiler's cfg values and host tuple as a typed environment file.
  r[mantle.rust_unit_plan.build_scripts.target_facts]
- Add build-script execution units: the unit helper runs the compiled script
  with a declared environment and `OUT_DIR` set to the unit's `out` output,
  and writes the parsed directives to a typed `flags` output.
  r[mantle.rust_unit_plan.build_scripts.run_units]
- Propagate directives as data: dependent compilations read `flags` through
  rustc argument files and environment files, `links` metadata reaches
  dependent build scripts as `DEP_<LINKS>_<KEY>`, and unsupported directives
  fail the unit. r[mantle.rust_unit_plan.build_scripts.directive_propagation]
- Declare native inputs in Nickel: a typed record keyed by Cargo package id
  with role-named store inputs, environment values, and PATH entries. The
  producer resolves roles to store paths and adds them as unit inputs.
  r[mantle.rust_unit_plan.build_scripts.native_inputs]
- Keep build scripts sandboxed: no network, no host paths, and no Cargo;
  `rerun-if-*` directives are recorded but do not affect identity.
  r[mantle.rust_unit_plan.build_scripts.sandbox_boundary]
- Mark build-script, proc-macro-with-build-script, and `links`/pkg-config
  matrix rows supported for the `unit_plan` lane only with passing fixtures.
  r[mantle.rust_unit_plan.build_scripts.matrix_evidence]

## Impact

- **Immediate consumer**: the Rust unit-plan lane. Its first targets are
  graphs with `proc-macro2`, `serde` with derive, and `libc`, plus one `-sys`
  crate linked against a Mantle-built native library.
- **Immediate outcome**: typical crate graphs build as per-unit derivations.
- **Durable capability**: typed, data-only native dependency declarations for
  Rust builds. This is a candidate first consumer for
  `adopt-data-only-dependency-exports`, subject to that change's admission
  gate.
- **Maintenance owner**: Mantle Rust-planning owner, covering the lowering
  adapter, the unit helper, and the lane's Nickel contract.
- **Repeatability evidence**: directive propagation fixtures, native-input
  fixtures, negative controls, and matrix evidence.
- **Compatibility**: `rust-plan` host execution and receipts are unchanged,
  and the default offline Cargo lane is unchanged.

## Scope

The change covers target-description units, build-script execution units,
directive parsing and propagation through the helper, `links` metadata, native
input declarations, matrix rows, reports, documentation, and positive and
negative fixtures.

## Non-Goals

- Network access or source downloads from build scripts.
- Detecting native libraries on the host; every native input is declared.
- Claims about third-party build-script correctness or `-sys` crate behavior.
- Behavior-bearing dependency hooks; native inputs contribute data only.
- Content-addressed build-script outputs and early cutoff.

## Success Criteria

- A graph containing `proc-macro2`, `serde` with derive, and `libc` builds
  through the lane.
- A `-sys` crate links against a declared Mantle-built library, and removing
  the declaration makes its build-script unit fail without finding the library.
- Changing a declared native input reruns only the affected build-script
  execution units and their dependents.
