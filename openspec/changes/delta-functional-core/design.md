# Design: delta functional core

## Context

`crunch-delta` already contains a useful pure slice:

- `model.rs` defines transfer fixtures, manifests, and tallies
- `negotiation.rs` defines protocol/chunk-profile negotiation
- `planner.rs` computes reuse plans over sender/receiver state

Those modules are deterministic and heavily testable today, but they still live
next to std-only code:

- `manifest.rs` probes castore services, uses async traits, and walks remote/
  local store state
- `substitution.rs` negotiates HTTP/session flows, handles `PathInfo`, persists
  attestations, and integrates with `crunch-store`
- fixture helpers still use std collections and benchmark-only conveniences

A straight file move is not enough. The current pure modules still expose
`HashSet`, `snix_castore::B3Digest`, and std error traits directly. That would
carry std-shaped boundary assumptions into the new core and make the no-std
claim weaker than the earlier waves.

## Goals / Non-Goals

**Goals:**

- create a real `crunch-delta-core` crate for transfer model, negotiation, and
  planning
- normalize the core API onto owned no-std-safe types instead of leaking
  `HashSet`, `snix_castore::B3Digest`, `PathInfo`, or async traits
- keep `crunch-delta` as the std adaptor/re-export layer for manifest building,
  substitution, and store/network integration
- extend the adopted-core validation inventory, allowlist, ownership review,
  and shell-boundary proof to the third wave
- preserve current delta transfer behavior while changing internal boundaries

**Non-Goals:**

- moving async substitution or manifest probing into the no-std crate
- redesigning the delta wire protocol itself
- extracting every fixture/benchmark helper into the core immediately
- changing external CLI behavior as the goal of this change

## Decisions

### 1. Add `crunch-delta-core` as a dedicated third-wave crate

**Choice:** create `crates/crunch-delta-core/` with `#![no_std]` plus
`extern crate alloc`, and move the pure transfer model, negotiation, and
planning logic there.

**Rationale:** this matches the established enforcement pattern. The compiler,
not convention, blocks store/network/runtime dependencies from creeping into
protocol negotiation and reuse planning.

**Alternative:** keep the code inside `crunch-delta` behind comments or module
naming.

**Why not:** that keeps the pure slice one edit away from a silent std leak.

### 2. Normalize digests and sets at the std adaptor boundary

**Choice:** `crunch-delta-core` will expose crate-local digest and collection
shapes that are no-std-safe:

- `DeltaDigest([u8; 32])` as the core-local BLAKE3 digest surface
- `BTreeSet<DeltaDigest>` for receiver membership sets that are currently
  `HashSet<B3Digest>`
- deterministic `Vec<...>` ordering for outputs, children, supported versions,
  and supported chunk profiles

`crunch-delta` will convert to/from `snix_castore::B3Digest`, `HashSet`,
`PathInfo`, and other runtime-owned types before and after the core call.

**Rationale:** the core cannot honestly depend on `snix_castore` today, and the
current public structs cannot stay no-std while they expose `HashSet`.
Boundary-normalizing those types is part of the extraction, not optional polish.

**Trade-off:** `crunch-delta` must translate to/from existing std/runtime types.
That is acceptable because translation belongs in the shell.

### 3. Keep manifest probing, substitution orchestration, and fixture builders in `crunch-delta`

**Choice:** `crates/crunch-delta/src/{manifest.rs,substitution.rs,fixtures.rs}`
remain std-owned. `model.rs` types such as `ClosureFixture`, `ReceiverManifest`,
`OutputFixture`, and the planner/negotiation-facing node/value shapes move into
`crunch-delta-core`, but benchmark/store fixture *builders* in `fixtures.rs`
stay in `crunch-delta` and construct those core-owned types from std test data.
Those std modules build normalized core inputs from castore/store/runtime
state, call `crunch-delta-core`, then continue async framing, transfer, and
attestation persistence.

**Rationale:** `manifest.rs` and `substitution.rs` still own async traits,
`PathInfo`, `StorePathRef`, HTTP/session framing, and `crunch-store`
integration. `fixtures.rs` still owns benchmark-only helpers and store-backed
fixture construction. Those are shell responsibilities, not core
responsibilities.

### 4. Preserve the existing std-facing crate name and facade

**Choice:** callers keep using `crunch-delta` as the std-facing surface while it
re-exports or wraps `crunch-delta-core` items where needed.

**Rationale:** this follows the first-wave pattern and reduces downstream churn.
It also keeps the facade near `substitution.rs`, which still owns the runtime
integration story.

**Compatibility rule:** extraction must preserve a usable `crunch-delta`
facade for planner/negotiation/model consumers, even if the no-std core adopts
new internal names or conversion helpers.

### 5. Map old std-shaped surfaces to new core types explicitly

**Choice:** the migration will use these explicit boundary mappings:

| Current surface | New core surface | Conversion lives in |
|---|---|---|
| `snix_castore::B3Digest` | `crunch_delta_core::DeltaDigest` | `crunch-delta` adaptor helpers |
| `HashSet<B3Digest>` membership | `BTreeSet<DeltaDigest>` membership | `crunch-delta` manifest/substitution paths |
| runtime fixture/store state | `ClosureFixture` / `ReceiverManifest` core records | `crunch-delta` fixture + manifest builders |
| runtime negotiation/session state | `NegotiationOffer` / `NegotiatedProtocol` core records | `crunch-delta` substitution helpers |
| core `TransferPlan` / `PlanError` / `NegotiationError` | std-facing facade re-exports or wrappers | `crunch-delta::lib` |

**Rationale:** the core boundary is only reviewable if the old-to-new mapping is
written down before code starts moving.

### 6. Keep core error surfaces no-std-safe and typed

**Choice:** `crunch-delta-core` keeps `PlanError` and `NegotiationError` as
crate-local typed enums with `Display` formatting only. Any `std::error::Error`
impls, runtime transport wrappers, or adapter-specific error translation stay
in `crunch-delta`.

**Rationale:** the moved modules already have typed errors, but today they also
implement std error traits. The third-wave core must stop at typed enums and
leave std integration in the adaptor.

### 7. Extend the no-std proof rail instead of inventing a delta-specific one

**Choice:** the existing adopted-core inventory, allowlist, scope/API-shape/
ownership checkers, and `scripts/check-no-std-core.sh` grow to include the
third-wave delta core.

**Rationale:** one rail is easier to keep honest than a stack of one-off checks.
The earlier waves already taught the repo how literal the OpenSpec gates are.
The delta wave should reuse that structure, not sidestep it.

## Core / Shell Boundary

### Core (`crunch-delta-core`)

Owns:

- transfer model types needed for planning and negotiation, including
  `ClosureFixture`, `ReceiverManifest`, `OutputFixture`, and the node/value
  shapes consumed by the planner and negotiator
- normalized sender/receiver manifest shapes built from `DeltaDigest`, ordered
  `BTreeSet` membership, and deterministic `Vec` traversal order
- protocol-version and chunk-profile negotiation
- transfer-plan computation, tallies, and pure validation errors
- pure positive/negative tests for protocol and planning behavior

Must not own:

- async castore probing
- `PathInfo`, `StorePathRef`, verifying keys, or attestation persistence
- HTTP endpoint construction tied to remote substitution sessions
- benchmark-only fixture stores that depend on std collections or services

### Std adaptor (`crunch-delta`)

Owns:

- conversions between store/runtime state and normalized core requests,
  including `B3Digest` ↔ `DeltaDigest`, `HashSet` ↔ `BTreeSet`, and runtime
  manifest/substitution state ↔ typed core requests
- manifest building over castore services
- async substitution/session orchestration
- attestation/store integration and wire framing
- façade re-exports for downstream callers

## Validation Strategy

- extend the workspace/inventory/allowlist so `crunch-delta-core` is treated as
  an adopted no-std core alongside the first two waves
- add host + `wasm32-unknown-unknown` `cargo check` coverage for
  `crunch-delta-core`
- add `cargo test -p crunch-delta-core` positive and negative coverage for the
  normalized planner/negotiation/model APIs
- add a std-shell boundary test in `crunch-delta` proving store/network/
  `PathInfo`/attestation work stays outside the core and only normalized core
  requests cross the boundary
- add `cargo test -p crunch-delta delta_facade_reexports_core_planner_types`
  so the std-facing `crunch-delta` facade is verified alongside the shell/
  core split
- extend ownership review so the delta adaptor files are explicitly classified
  and future logic drift back into them is caught

## Risks / Trade-offs

**Digest/type normalization could force API churn** → Mitigation: keep the
std-facing `crunch-delta` facade stable and absorb compatibility glue there.

**Fixtures may not fit the core boundary cleanly** → Mitigation: keep
benchmark/store fixtures in the std crate for this wave; extract only the pure
planning/protocol shapes first.

**Validation drift across waves** → Mitigation: update the single adopted-core
inventory, allowlist, and runner instead of creating delta-only checks.

**Scope creep into async substitution** → Mitigation: treat `manifest.rs` and
`substitution.rs` as explicitly out-of-core for this change and keep that limit
in tasks and delta specs.
