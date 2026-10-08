# Design: Run Cargo build scripts as plan units

## Goal and scope

Build-script execution becomes a plan unit whose outputs feed dependent
compilations and build scripts. Native libraries enter only through typed
declarations.

## Current behavior

- The Cargo unit graph models build-script execution as `run-custom-build`
  units that depend on the compiled script and on the execution units of
  dependencies with `links` keys.
- `rust-plan` parses `rustc-cfg`, `rustc-env`, `rustc-link-lib`,
  `rustc-link-search`, `rerun-if-changed`, and metadata directives
  (`src/rust_plan.rs:787-797`) and runs scripts on the host with a scrubbed
  environment (`src/rust_plan.rs:15066-15099`).
- The lane from `build-cargo-units-as-dynamic-plans` blocks
  `run-custom-build` units with `unit-plan-build-script-run-unsupported`.
- Nickel recipes locate inputs by name at build time
  (`lib/offline_cargo.ncl:119-139`). The evaluator has no string context, so
  store references cannot be inferred from interpolated strings.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Three wrappers | Separate `target-env`, `build-wrap`, and `env-wrap` tools, as in the reference | Rejected: three tool closures for one concern | None |
| Host discovery | pkg-config or compiler probes against host paths | Rejected: breaks sandbox hermeticity | Sandbox negative fixture |
| Reference-graph inference | Derive native inputs from string references, as `writeExtern` does | Rejected: Nickel has no string context; roles are explicit instead | None |
| Helper modes and typed declarations | One helper with target-facts, run-build-script, and rustc modes; role-named native inputs | Selected | Directive and native-input fixtures |

## Contract and component ownership

- **Pure core**: directive grammar, the supported directive set, propagation
  rules (immediate versus transitive link arguments and `links` metadata), and
  the native-input record contract.
- **Lowering adapter**: emits target-description units, build-script
  execution units with declared environments, and dependent references to
  `flags` outputs.
- **Unit helper**: runs the target query or the script, captures stdout,
  applies the pure parser, and writes `flags/args-immediate`,
  `flags/args-transitive`, `flags/env`, and `flags/links`.
- **Nickel contract**: native inputs keyed by Cargo package id, each with
  `inputs` (role name to derivation), `env`, and `path`, where values refer to
  roles through a `{{role:<name>}}` marker that the producer resolves.

## Decisions

### Decision: One helper with modes

**Choice:** Extend the unit helper with target-facts and run-build-script
modes instead of adding tools.

**Rationale:** One small static tool closure and one input-validation surface.

### Decision: Target facts are their own unit

**Choice:** One unit per toolchain and triple records the compiler's cfg and
host facts, and every build script for that triple consumes it.

**Rationale:** The facts change only with the toolchain, so the unit is reused
across all packages and runs.

### Decision: Supported directives are enumerated

**Choice:** Support `rustc-cfg`, `rustc-check-cfg`, `rustc-env`,
`rustc-link-lib`, `rustc-link-search`, `rustc-link-arg` and its target-kind
variants, `rustc-flags`, `metadata`, legacy `KEY=VALUE` metadata, `warning`,
`error`, `rerun-if-changed`, and `rerun-if-env-changed`. Any other directive
fails the unit.

**Rationale:** Silently accepting an unknown directive would change
compilation without review.

### Decision: Native inputs are role-named data

**Choice:** A declaration names derivations by role and refers to them in
values through role markers. The producer resolves roles to store paths inside
its sandbox and adds them as unit inputs.

**Rationale:** The evaluator cannot infer references from strings, and
data-only declarations follow the no-behavior rule proposed in
`adopt-data-only-dependency-exports`.

### Decision: `rerun-if-*` does not affect identity

**Choice:** The helper records these directives in `flags` for reports but
does not act on them.

**Rationale:** Unit identity already covers every declared input.

## Failure behavior and ordering

Stable failures are `build-script-directive-unsupported`,
`build-script-error-directive`, `build-script-nonzero-exit`,
`build-script-links-conflict`, and the evaluation-time
`native-input-role-undeclared` for a value that refers to a role the
declaration does not name. Directive files keep script output order.
Propagated arguments are ordered by dependency topology, then unit id.

## Tests

- Positive: `proc-macro2`, `serde` with derive, and `libc` graphs; a `-sys`
  crate against a declared Mantle-built library; `links` metadata reaching a
  dependent build script.
- Negative: an unknown directive, `cargo::error`, a nonzero exit, an
  undeclared role marker, a network attempt, a host-path probe, a removed
  native declaration, and two packages with one `links` key.
- Reuse: changing a native input reruns only affected execution units and
  their dependents.

## Risks / Trade-offs

- Some build scripts probe the C compiler in ways the sandbox exposes.
  Declared C toolchains and negative fixtures bound this.
- Native declarations are written per package. A later exports-record adoption
  can derive them from dependency outputs.
- Directive coverage lags new Cargo features; unknown directives fail closed.

## Claim boundary

Evidence proves directive parsing, propagation, and declared native inputs for
the tested graphs. It does not prove build-script correctness, native library
correctness, or release eligibility.
