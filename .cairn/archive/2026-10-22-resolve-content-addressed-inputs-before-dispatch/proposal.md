# Proposal: Resolve content-addressed inputs before dispatch

## Why

Before this change, Mantle builds floating content-addressed (CA) outputs but
cannot stop work when a CA dependency's derivation changes while its output
bytes stay the same. The isolated worktree and executable proving this
behavior are recorded in `evidence/baseline-2026-10-01.md`:

- CA output paths are recorded per unresolved derivation path through
  `insert_ca_mapping` and `resolve_ca_mapping`.
- The shared `action_ref_for_derivation` hashes the unresolved derivation
  text, and cache checks key on the unresolved derivation path. A dependent's
  identity therefore moves whenever a dependency's derivation moves, even
  when the dependency's output does not.
- Dependents see CA inputs at provisional paths, and their outputs are
  rewritten after the build through `collect_ca_input_rewrites` and
  `apply_input_rewrites`.
- Native dynamic plans reject placeholders on CA unit outputs because their
  paths are unknown at registration.

The reviewed `cargo-dyndrv` reference makes every Cargo unit CA to get early
cutoff from Nix's resolved derivations (`evidence/cargo-dyndrv-review.md`).
Mantle's Rust unit-plan lane, the hardware reference's compile units, and
bootstrap chains all have dependencies whose derivations change more often
than their outputs.

## What Changes

- Resolve before dispatch: once every direct CA input or previously resolved
  CA-dependent intermediate is realized, substitute its admitted output path
  into arguments and environment, move its input edge to input sources, and
  repeat through downstream dependents.
  r[mantle.ca_input_resolution.resolved_derivation]
- Key cache lookup, CA output mappings, and shared action-result identity on
  the resolved derivation for direct and transitive CA dependents. Derivations
  in CA-independent subgraphs keep their current identities exactly.
  r[mantle.ca_input_resolution.resolved_identity]
- Reuse a dependent without executing it when its resolved derivation matches
  an admitted result, even when a CA dependency's unresolved derivation
  changed. r[mantle.ca_input_resolution.early_cutoff]
- Record realisations as signed records that bind resolved identity, output
  name, and output path; admit them under the PathInfo trust policy and share
  them through existing substitution and action-result sources.
  r[mantle.ca_input_resolution.realisation_records]
- Let dynamic-plan placeholders reference CA unit outputs; the worker binds
  them from realized paths during resolution instead of rejecting them at
  registration. r[mantle.ca_input_resolution.dynamic_plan_binding]
- Fail closed on unrealized inputs, conflicting realisations, and untrusted
  records. r[mantle.ca_input_resolution.negative_controls]

## Impact

- **Immediate consumer**: the Rust unit-plan lane
  (`build-cargo-units-as-dynamic-plans`), where CA units stop rebuilding
  dependents when an edit leaves a package's outputs unchanged. The hardware
  reference's compile-to-link edges are the second consumer.
- **Immediate outcome**: early cutoff for derivation graphs with CA outputs,
  and CA units that plans can reference.
- **Durable capability**: resolved identity as the cache and sharing key for
  CA-dependent work, local and remote.
- **Maintenance owner**: Mantle build-engine and store owners, covering
  `orchestrate.rs`, `worker.rs`, `registry.rs`, `action_result.rs`, and CA
  mapping storage in `crunch-store`.
- **Repeatability evidence**: identity golden fixtures for CA-independent
  derivations, a three-node early-cutoff fixture with zero child or grandchild
  executions, realisation trust negatives, and a CA plan-placeholder fixture.
- **Compatibility**: CA-independent derivations keep their derivation paths,
  output paths, and action refs. Their CA mappings retain the same keys, but
  reuse still requires an admitted signed action result and PathInfo;
  mapping-only results rebuild. CA-dependent mappings change keys once.

## Scope

The change covers the pure resolution function, resolved identity, cache and
action-result keying, signed realisation records, trust admission, dynamic
plan placeholder binding, retirement of post-build input rewrites, reports,
documentation, an ADR, and positive and negative fixtures.

## Non-Goals

- Changing identities of CA-independent derivations.
- Making the Rust unit-plan lane content-addressed by default; adoption is a
  separate change after this one.
- Nix realisation wire compatibility beyond the reviewed Nix adapter
  boundary.
- Removing self-reference rewriting of CA outputs (owned by
  `relocate-dynamic-output-references`).
- Determinism or reproducibility claims; early cutoff proves reuse of
  identical admitted content only.

## Success Criteria

- Changing a CA dependency's derivation without changing its output bytes
  reruns that dependency and reuses every dependent without executing it.
- CA-independent derivations keep their recorded identity goldens.
- A plan unit that references a CA unit output through a placeholder builds.
- Conflicting or untrusted realisations never authorize reuse.
