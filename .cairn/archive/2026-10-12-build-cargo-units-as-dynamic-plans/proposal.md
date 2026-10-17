# Proposal: Build Cargo units as dynamic-plan derivations

## Why

Mantle has two Rust build surfaces, and neither gives fine-grained reuse with
derivation authority:

- `mantle.offlineCargoPackage` (`lib/offline_cargo.ncl`) is the default
  project lane. It runs `cargo build --locked --offline` once per package
  inside the sandbox (`lib/offline_cargo.ncl:163`). Cargo's target directory
  does not outlive the derivation, so any input change recompiles the whole
  crate graph.
- `mantle rust-plan` reports either Cargo-oracle evidence or bounded
  `--no-cargo-oracle` native topology evidence; supported unit execution
  remains a separate host-side verification workflow. Neither receipt makes
  `rust-plan` the default project build lane.

Mantle already owns the execution half. Native dynamic plans (ADR 0011) turn
producer-generated units into ordinary sandboxed derivations with signed
PathInfo, shared action results, substitution, and remote dispatch. The
hardware reference uses `mantle-plan-v1` without scheduler special cases;
the current Cargo-unit producer emits `mantle-plan-v2` so package source
slices are independently admitted.

The reviewed `cargo-dyndrv` reference (`evidence/cargo-dyndrv-review.md`)
supplies the Rust half: plan with `cargo build --unit-graph` inside a build
step and emit one derivation per Cargo unit. Its Nix mechanisms (`.drv`
outputs, the `builder-rpc-v0` store socket, and per-root naming wrappers) do
not transfer; its unit model does. ADR 0081 records the decision.

## What Changes

- Keep Rust semantics in the Rust-planning frontend and use only generic
  dynamic-plan mechanisms in the worker and store: no `.drv` discovery, no
  store interface inside the sandbox, and no Rust branches in the scheduler.
  r[mantle.rust_unit_plan.frontend_boundary]
- Add a sandboxed producer derivation that runs the Rust planner offline, with
  Cargo metadata and the Cargo unit graph as the planning source, and writes a
  declared dynamic-plan output with one unit per supported compilation unit.
  r[mantle.rust_unit_plan.producer]
- Lower planned unit effects to plan units through a pure adapter:
  input-addressed units, store-path builders, placeholders for every source,
  toolchain, and dependency path, `-C metadata` from rust-plan unit identity,
  source path remapping, and no host paths. r[mantle.rust_unit_plan.lowering]
- Reference only direct dependencies as unit inputs, and carry transitive
  reachability through a dependency manifest in each library output.
  r[mantle.rust_unit_plan.dependency_closure]
- Give each package its own source object through dynamic-plan source slices.
  r[mantle.rust_unit_plan.per_crate_sources]
- Run units through one Mantle-built static unit helper that reads its output
  paths from the build environment and invokes rustc with an argument file.
  r[mantle.rust_unit_plan.unit_helper]
- Fail closed on build-script execution units, unsupported modes, sources, and
  triples, and plan limits, with named blockers and no fallback to Cargo
  compilation. r[mantle.rust_unit_plan.supported_fragment]
- Add a Nickel entry point, a `unit_plan` lane in the Rust compatibility
  surface matrix, and the evidence class `cargo-unit-graph-dynamic-plan` with
  explicit non-claims. r[mantle.rust_unit_plan.evidence_lane]
- Prove work reduction with fresh, one-package edit, outside-package edit, and
  shared-hit runs recorded as counts. r[mantle.rust_unit_plan.work_reduction]

## Impact

- **Immediate consumer**: the signed two-package path-workspace fixture in
  `examples/cargo_unit_plan.ncl` and the bounded `unit_plan` rows in
  `examples/rust_compatibility_surface_matrix.ncl`. Mantle's current
  `Cargo.lock` has 938 external packages; its offline Cargo graph measured
  925 units, 711 distinct package sources, maximum 65 direct dependencies,
  and 948146 compact graph JSON bytes. Its 711 sources exceed the generic
  `mantle-plan-v2` 256-source-slice cap; the full workspace is not an
  admitted self-host fixture for this lane.
- **Immediate outcome**: the two-package fixture builds per-unit
  derivations with separately admitted sources and a bound app consumer.
  Edit-dependent zero-rebuild receipts remain a work-reduction proof gate.
- **Durable capability**: a Rust frontend on the generic dynamic-plan path
  with remote dispatch, substitution, and shared results, reusable by any
  Mantle consumer that builds Rust.
- **Maintenance owner**: Mantle Rust-planning owner, covering
  `crates/mantle-rust-plan-core`, `crates/mantle-rust-plan-app`, the lowering
  adapter, the unit helper, and the `lib/` entry point.
- **Repeatability evidence**: lowering goldens, producer sandbox fixtures,
  negative blockers, and a work-reduction bundle.
- **Compatibility**: `offlineCargoPackage` stays the default project lane;
  `rust-plan` receipts, evidence rails, and ADR 0035 cache semantics are
  unchanged.

## Scope

The change covers the producer derivation, the lowering adapter, the
dependency manifest, per-package slices, the unit helper, the Nickel entry
point, the matrix lane, reports and evidence, documentation, the review of
ADR 0081, and positive and negative fixtures.

## Non-Goals

- Cargo-free execution claims; Cargo remains the planning source in this lane.
- Build-script execution, `links` metadata, and native library inputs (owned
  by `run-cargo-build-scripts-as-plan-units`).
- Content-addressed units and early cutoff (after
  `resolve-content-addressed-inputs-before-dispatch`).
- Static Nickel consumers of plan roots (owned by
  `bind-static-inputs-to-dynamic-plan-roots`).
- Pipelined compilation that starts dependents from `.rmeta` before `.rlib`.
  Revisit trigger: work-reduction evidence showing the critical path dominated
  by upstream library code generation.
- Test, doc, and check unit modes.
- Replacing `offlineCargoPackage` as the default project lane.
- Nix `.drv` generation, `builder-rpc-v0`, recursive-nix, or copying
  `cargo-dyndrv` code.

## Success Criteria

- The supported representative fixtures build through `mantle build` as one
  derivation per unit, and an unchanged rerun executes zero units.
- For the same supported default-feature two-package source and lock, the
  default `offlineCargoPackage` build and unit-plan app have matching
  observed stdout, stderr, and exit under the same inputs and run
  conditions; unsupported or unavailable comparisons remain unproven.
- An edit to one workspace package reruns only that package's units and their
  dependents; an edit outside every package source reruns only the producer.
- A clean client with trusted keys reuses every unit from shared results
  without executing it.
- Unsupported graphs fail with named blockers and never fall back to Cargo
  compilation.
