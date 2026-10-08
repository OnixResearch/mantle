# ADR 0081: Build Cargo units as native dynamic-plan derivations

## Status

Proposed (2026-09-25); revised after bounded implementation review (2026-10-01). This decision is not yet accepted as a completed build lane.

## Context

Mantle has three relevant surfaces.

`mantle.offlineCargoPackage` is the default Rust project lane. It runs one
offline Cargo build per package inside the sandbox. Cargo's target directory
does not survive the derivation, so every input change recompiles the whole
crate graph.

`mantle rust-plan` records Cargo-oracle evidence or bounded native
`--no-cargo-oracle` topology evidence and can execute supported units in a
separate host-side verification workflow. ADR 0035 rejects ordinary PathInfo
semantics for those host-executed outputs because they lack derivation authority.

Native dynamic plans (ADR 0011) let a producer build emit bounded data that
the worker turns into ordinary sandboxed derivations. The hardware reference
shows that a frontend can lower its own graph this way without scheduler
special cases.

The reviewed external reference, `cargo-dyndrv` by Obsidian Systems
(revision `d5245b85`, 2026-09-16), builds one Nix derivation per Cargo unit
from `cargo build --unit-graph`. It relies on Nix dynamic derivations,
`builtins.outputOf`, a restricted builder store socket (`builder-rpc-v0`),
content-addressed outputs for early cutoff, and three wrapper tools for build
scripts.

## Decision Drivers

- Reuse unchanged Rust units with derivation authority: sandboxing, signed
  PathInfo, shared results, substitution, and remote dispatch.
- Keep Rust meaning out of the scheduler and store (ADR 0010, ADR 0011).
- Keep store creation out of sandboxes and admit dynamic graph growth as
  bounded data.
- Keep the `rust-plan` evidence lane and ADR 0035 semantics intact.
- State plainly that Cargo still plans the graph in this lane.

## Decision

Mantle will build Cargo projects as native dynamic plans with one sandboxed
derivation per Cargo compilation unit.

1. A sandboxed producer derivation runs the Rust planner offline, with Cargo
   metadata and the Cargo unit graph as the planning source, and writes a
   declared dynamic-plan output.
2. A pure adapter over the Rust-planning core's unit effects lowers units to
   plan units. Units are input-addressed, reference only direct dependencies,
   derive `-C metadata` from rust-plan unit identity, and remap source paths.
3. Per-package source objects come from plan-declared source slices admitted
   by the worker, not from a store interface inside the sandbox.
4. Static consumers bind plan roots through a typed plan-output input, not
   through naming wrapper derivations.
5. Content-addressed units wait for resolved content-addressed identity, which
   is what makes early cutoff possible.
6. Lane units are ordinary derivations. ADR 0035 continues to govern
   host-executed `rust-plan` units; the two surfaces may share castore chunks
   but not result authority.
7. The lane is opt-in, with evidence class `cargo-unit-graph-dynamic-plan`.
   `offlineCargoPackage` stays the default until the Rust compatibility matrix
   shows parity and a separate change switches the default.

The Cairn changes `build-cargo-units-as-dynamic-plans`,
`add-dynamic-plan-source-slices`, `bind-static-inputs-to-dynamic-plan-roots`,
`resolve-content-addressed-inputs-before-dispatch`, and
`run-cargo-build-scripts-as-plan-units` carry this decision.

## Alternatives Considered

### Port cargo-dyndrv's `.drv` output

Rejected. ADR 0011 keeps `.drv` discovery as compatibility only, and the
versioned dynamic-derivation form admits input-addressed outputs only.

### Expose a builder store socket

Rejected. `builder-rpc-v0` and recursive-nix put a store protocol inside the
sandbox and create objects before admission. Plan data admitted after the
producer completes provides the same capability under named limits.

### Cache rustc calls inside offline Cargo builds

Kept as a separate optimization. The ADR 0035 wrapper daemon can speed up the
default lane, but it adds no derivation authority and no per-unit remote
dispatch.

### Promote host-executed `rust-plan`

Rejected. Host execution has no sandbox and no PathInfo authority.

### Content-addressed units now

Rejected for this bounded lane: unit outputs remain input-addressed. Resolved
content-addressed identity and CA plan-output placeholders are owned by
`resolve-content-addressed-inputs-before-dispatch`, not this adapter.

## Consequences

- The opt-in entry point, evidence class, and matrix lane have an owner and
  documentation; `offlineCargoPackage` remains the default.
- Plans are bounded by 4 MiB, 4,096 units, 256 direct inputs per unit, and
  256 package source slices. Mantle's current lock has 938 external packages;
  its offline graph measured 925 units, 711 package sources, maximum 65
  direct dependencies, and 948146 compact graph JSON bytes. Its 711 source
  slices exceed the current bound, so the full workspace is not admitted.
- Every unit pays sandbox setup cost. Work-reduction evidence must record
  unit counts, with elapsed time as a diagnostic only.
- The lane depends on Cargo's unstable unit graph. The producer pins the
  toolchain and fails closed on unknown versions.
- The signed two-package fixture proved admitted per-package slices, bounded
  unit execution, and same-state recovery; an uninterrupted final-source
  cold build and the complete work-reduction bundle remain open. Evidence
  does not prove Cargo equivalence, Cargo-free execution, compiler correctness,
  reproducibility, or release eligibility.

## Implementation review (2026-10-01)

The implemented pure adapter consumes explicit Rust-planning core unit effects
and emits `mantle-plan-v2` source-slice inputs. The offline producer admits
only declared workspace, optional vendor, toolchain, planner, and helper inputs;
the static helper compiles declared units and reads direct dependency manifests.
Those boundaries retain this ADR's frontend-owned Rust semantics and ordinary
worker admission. Typed plan-root consumers and content-addressed identity
resolution belong to their separately named Cairn changes, not to this crate;
build-script execution remains a separate change as well. This is a revision
of the decision record to match the bounded implementation, not acceptance of
T4.1 fixtures, T4.2 work reduction, immutable-source SECOND, or archive.
