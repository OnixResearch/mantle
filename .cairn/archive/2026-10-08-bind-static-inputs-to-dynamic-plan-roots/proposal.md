# Proposal: Bind static inputs to dynamic-plan roots

## Why

Mantle builds native dynamic-plan roots and exports them as requested roots
(`crates/crunch-build/src/worker.rs:1638-1654` wants every plan root with
`is_root = true`), but nothing evaluated before the build can consume them.
The Nickel input model offers sources, derivation files, preconverted
derivation edges, output selections, and inline derivations only
(`crates/crunch-glue/src/types.rs:191-202`). The scheduler has an
`AwaitingDerivation` goal state for work whose derivation arrives later, but
only tests construct it (`crates/crunch-build/src/goal.rs:104`).

Nix fills this gap with `builtins.outputOf`. The reviewed `cargo-dyndrv`
reference depends on it and adds a symlink wrapper derivation per root only to
satisfy output naming rules (`evidence/cargo-dyndrv-review.md`). Without an
equivalent, a Rust binary built through a plan cannot be placed into a
wrapper, image, or system closure declared in Nickel, and a lock-driven vendor
assembly produced by a plan cannot feed a statically declared Cargo build.

## What Changes

- Add a typed Nickel input that names a producer derivation, one of its
  declared dynamic-plan outputs, a plan root unit id, and a unit output.
  r[mantle.dynamic_plan_output_inputs.typed_reference]
- Give the consumer an evaluation-time identity: the symbolic reference binds
  into its derivation through existing derivation fields, so its derivation
  path and input-addressed output paths are known before the producer runs.
  r[mantle.dynamic_plan_output_inputs.request_identity]
- Expose the reference to builders as a deterministic placeholder that the
  worker replaces with the bound unit output path at dispatch; mount that path
  with its closure and include it in reference scanning.
  r[mantle.dynamic_plan_output_inputs.dispatch_binding]
- Bind after plan acceptance: the consumer waits for the producer, then for
  the named root unit. A rejected plan, a missing root, a missing output, or a
  failed root fails the consumer with a typed reason.
  r[mantle.dynamic_plan_output_inputs.binding_failures]
- Record the symbolic reference, plan digest, bound unit derivation path, and
  bound output path in build reports and provenance.
  r[mantle.dynamic_plan_output_inputs.provenance]

## Impact

- **Immediate consumer**: the Rust unit-plan lane
  (`build-cargo-units-as-dynamic-plans`), whose binaries must feed statically
  declared wrappers, images, and OnixOS closures. The lock-driven vendor
  assembly (`add-lock-driven-vendor-fetches`) is the second consumer.
- **Immediate outcome**: a Nickel derivation can depend on a plan root's
  output and receive its exact path at build time.
- **Durable capability**: static and dynamic graph parts compose without
  import-from-derivation, evaluator suspension, or per-root naming wrappers.
- **Maintenance owner**: Mantle build-engine owner, covering
  `lib/derivation.ncl`, `crates/crunch-glue`, and the binding path in
  `crates/crunch-build`.
- **Repeatability evidence**: identity golden fixtures, dispatch-binding
  fixtures, typed failure fixtures, and report rows.
- **Compatibility**: derivations without the new input keep their identities
  and behavior.

## Scope

The change covers the Nickel contract, the crunch-glue input and conversion,
the deterministic placeholder, late binding in the worker, dispatch-time
replacement, sandbox mounting and reference scanning of the bound path,
reports, documentation, an ADR for the request-identity decision, and positive
and negative fixtures.

## Non-Goals

- Import-from-derivation or evaluator suspension (rejected by ADR 0011).
- Wire compatibility with Nix `builtins.outputOf` or `DrvWithVersion` dynamic
  outputs.
- Consuming units that are not plan roots; plans choose what they export
  through `roots`.
- Proving that a producer is deterministic; the request identity carries the
  same determinism assumption as any input-addressed dependency.
- Content-addressed resolution of the consumer itself (owned by
  `resolve-content-addressed-inputs-before-dispatch`).

## Success Criteria

- A Nickel derivation that consumes a plan root builds in one `mantle build`
  run after the producer and the root, and its builder sees the root's exact
  output path.
- Its derivation path is identical across evaluations with the same producer
  derivation and reference names.
- A rejected plan, missing root, missing output, or failed root fails the
  consumer with the matching typed reason and no successful output.
- Reports record the symbolic reference and the bound paths.
