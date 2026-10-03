# Proposal: Add dynamic-plan source slices

## Why

A native dynamic plan can reference only store paths that already exist.
Plan registration parses `sources[].path` without creating anything
(`crates/crunch-build/src/worker.rs:420-437`), and a unit's prepare step fails
with `SourceNotFound` when a source has no PathInfo
(`crates/crunch-build/src/orchestrate.rs:931-951`). A producer can therefore
give its generated units either pre-existing inputs or one of its own whole
outputs. When units read a whole output, every unit changes identity whenever
any byte of that output changes.

Frontends that need per-unit sources hit this limit first. A Cargo unit
planner needs one source object per package, so that editing one package
leaves the other packages' unit identities unchanged. The reviewed external
reference, `cargo-dyndrv`, adds each crate root to the Nix store from inside
the builder through a restricted daemon socket (`builder-rpc-v0` `AddToStore`;
see `evidence/cargo-dyndrv-review.md`).

Mantle should not place a store protocol inside the sandbox. ADR 0011 made
dynamic graph growth bounded data that the worker admits after the producer
completes. Source creation should follow the same rule: the producer declares
slices of its own outputs, and the worker admits them.

## What Changes

- Add a versioned plan schema, `mantle-plan-v2`, whose source entries may be
  slices: a declared producer output, a relative subpath, a store name, and a
  required expected NAR BLAKE3. `mantle-plan-v1` decoding, canonical bytes,
  and plan digests stay unchanged.
  r[mantle.dynamic_plan_source_slices.versioned_schema]
- Admit every slice after plan validation and before unit registration:
  locate the subtree inside the named producer output, verify its content
  identity, and publish it as a content-addressed store object with signed
  PathInfo through the existing verified-source admission capability.
  r[mantle.dynamic_plan_source_slices.content_admission]
- Derive each slice's logical store path from its content and declared store
  name only, so identical content from different producer runs, outputs, or
  subpaths maps to one path.
  r[mantle.dynamic_plan_source_slices.content_identity]
- Bound slices by count, subpath bytes and depth, and admitted bytes. Reject
  undeclared outputs, escaping or absent subpaths, symlink traversal, digest
  mismatches, and conflicting duplicates before the first publication, and
  publish one plan's slices all or nothing.
  r[mantle.dynamic_plan_source_slices.bounded_rejection]
- Record each slice's source id, producer output, subpath, declared and
  observed digests, store path, and disposition in native dynamic-plan report
  rows. r[mantle.dynamic_plan_source_slices.provenance]

## Impact

- **Immediate consumer**: the Rust unit-plan lane
  (`build-cargo-units-as-dynamic-plans`), which needs one source object per
  package.
- **Immediate outcome**: plans can give each generated unit its own source
  object without a sandbox store interface.
- **Durable capability**: any plan producer (Cargo, generated C and C++,
  hardware generation, lock-driven vendor assembly) can split its output into
  independently cached sources.
- **Maintenance owner**: Mantle build-engine owner, covering
  `crates/crunch-build/src/dynamic_plan.rs`,
  `crates/crunch-build/src/dynamic_plan/wire.rs`, and the native plan
  registration path in `crates/crunch-build/src/worker.rs`.
- **Repeatability evidence**: `mantle-plan-v1` golden parity,
  `mantle-plan-v2` positive and negative admission fixtures, a two-run
  identity fixture, and report rows.
- **Compatibility**: existing producers keep emitting `mantle-plan-v1` with
  unchanged canonical bytes and digests; `mantle-plan-v2` is selected by its
  schema string.

## Scope

The change covers the `mantle-plan-v2` wire and admitted types, slice grammar
and limits, pure admission planning, worker-side subtree resolution and
verified-source admission, report rows, documentation, an ADR for the
schema-version decision, and positive and negative fixtures.

## Non-Goals

- A store protocol, daemon socket, recursive-nix, or `builder-rpc-v0`
  equivalent inside the sandbox.
- Slices of anything other than the producing derivation's declared outputs.
- Source trust, license, or provenance claims about slice content beyond its
  content identity.
- Changing `mantle-plan-v1` semantics or the `.drv` compatibility path.
- Early cutoff for content-addressed unit outputs (owned by
  `resolve-content-addressed-inputs-before-dispatch`).

## Success Criteria

- A `mantle-plan-v2` plan that declares N slices publishes N
  content-addressed sources and schedules its units in the same run.
- A producer rerun whose output changed only outside a slice leaves that
  slice's store path and the dependent unit derivation paths unchanged.
- Every rejected slice leaves registry, goal, scheduler, and success report
  state unchanged and yields one structured rejection row.
- Accepted `mantle-plan-v1` fixtures keep their canonical bytes and plan
  digests.
