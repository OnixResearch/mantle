# Design: no-std functional core

## Context

Crunch already has several pure islands:

- `crunch-attestation` canonicalization, schema validation, and digesting
- `crunch-project` manifest, lockfile, merge, and stale/refresh planning logic
- `crunch-shell` activation planning
- parts of `crunch-glue` and `crunch-build` that compute plans before effects

But those islands still mostly sit inside `std` crates. That means the compiler
still allows `std::fs`, `std::env`, `std::process`, wall-clock time, and other
ambient dependencies to leak back into code that wants to be pure.

This change makes the first compiler-enforced cut. It does not try to make the
entire workspace no-std. It introduces a durable pattern: pure no-std core
crates plus thin std shells/adapters.

## Goals / Non-Goals

**Goals:**

- make the first functional-core / imperative-shell boundary compiler-enforced
- define which current domains move first and which stay std for now
- keep core APIs on plain data, deterministic transforms, and explicit results
- require repeatable verification that the new core really stays no-std
- preserve current user-visible behavior while changing internal boundaries

**Non-Goals:**

- migrate every crunch crate in one pass
- make sandbox execution, store mutation, networking, or CLI parsing no-std
- redesign the whole workspace around one mega `crunch-core` crate
- commit yet to later no-std extraction for every remaining domain

## Decisions

### 1. Use dedicated no-std crates as the enforcement boundary

**Choice:** the first functional core will live in dedicated `#![no_std]`
crates that opt into `extern crate alloc`. Mixed std crates will depend on
those core crates instead of hiding purity behind comments or conventions.

**Rationale:** crate boundaries are the strongest honest proof. A dedicated
no-std crate makes ambient I/O, clock, process, and env reads a compile error.
That is stronger than “please keep this module pure” and clearer than weaving
`std` feature flags through large mixed crates.

**Alternative:** keep existing crate names and add optional `std` features
inside each one.

**Why not:** that keeps too much mixed ownership in one place, makes accidental
`std` coupling easier, and turns the first boundary into a feature-matrix job.

### 2. Keep shell layers responsible for all effects and std-shaped inputs

**Choice:** std crates remain responsible for filesystem access, subprocesses,
networking, environment reads, clocks, tempdirs, path discovery, and log
writing. Core crates accept plain owned data and return plans, normalized
values, validation results, or typed errors.

**Rationale:** this is the Tiger Style read → transform → write split. Shells
read and normalize the world. Core transforms deterministic values. Shells then
perform the chosen effects.

**Implementation shape:** shell adapters may translate `PathBuf`, `Command`,
`SystemTime`, env maps, or downloaded bytes into plain records, strings, byte
vectors, enums, and bounded collections before calling the core.

### 3. Pilot two domains first: attestation and project management

**Choice:** the first extraction wave targets:

- `crunch-attestation-core`: schema types, canonicalization, digesting, and
  pure release/policy transforms that do not touch the filesystem
- `crunch-project-core`: manifest/lock models, versioning, merge/drift logic,
  refresh planning and outcome application, and generated-input planning that
  do not perform I/O directly

The existing `crunch-attestation` and `crunch-project` crates become std
adapters/re-export surfaces around those no-std cores.

**Rationale:** these domains already contain substantial pure logic, have clear
shell boundaries today, and give high value with lower runtime risk than moving
scheduler, store, or sandbox code first.

**Alternative:** start with build/store runtime crates.

**Why not:** those crates still mix too much async I/O and OS interaction for a
first no-std landing. They are better second-wave targets once the pattern is
proven.

### 4. Keep current public crate names as the std-facing surface

**Choice:** callers may keep depending on `crunch-attestation` and
`crunch-project` as the stable std-facing crates while those crates re-export
or wrap items from `*-core` crates.

**Rationale:** this reduces churn for downstream callers and lets the first
boundary land without forcing every consumer to switch names immediately.

**Trade-off:** there will be some duplicate crate names in the workspace. That
is acceptable because the core/shell role split is clearer than a mass rename.

### 5. Prove the boundary with both compile and dependency checks

**Choice:** verification must include:

- `cargo check` for each new core crate on a `no_std` target such as
  `wasm32-unknown-unknown`
- a repo-local dependency check that core crate manifests do not pull in
  std-only runtime crates (`tokio`, `reqwest`, `ureq`, `tempfile`, `clap`,
  etc.)
- positive and negative unit tests in the new core crates
- adapter tests in std crates that prove shell translation happens outside the
  core boundary

**Rationale:** a no-std claim needs more than `#![no_std]` in one file. The
crate must compile for a no-std target, and its dependency surface must stay
clean over time.

### 6. Record later candidates, but do not migrate them in this change

**Choice:** this change will document later extraction candidates instead of
implementing them now. Current likely second-wave candidates are:

- `crunch-glue` derivation planning/data normalization
- `crunch-build` goal/state-machine planning helpers
- `crunch-store` attestation assembly and closure planning helpers
- `crunch-shell` activation planner once its std path/argv coupling is reduced

**Rationale:** first wave should establish pattern, not chase total coverage.
The follow-on list keeps momentum without turning this change into a rewrite.

## Interaction Sequence

### Project-management flow

1. std shell resolves project root, reads files, performs git/HTTP/local-path
   I/O, and snapshots any needed environment.
2. shell converts that input into plain `crunch-project-core` request values.
3. `crunch-project-core` validates manifest/lock state, computes refresh or
   drift outcomes, and returns typed results.
4. std shell writes lockfiles/generated inputs and formats user-visible output.

### Attestation flow

1. std shell finds files, reads attestation bytes, and gathers release witness
   inputs from disk or network.
2. shell converts those inputs into plain `crunch-attestation-core` values.
3. `crunch-attestation-core` canonicalizes, hashes, validates, and evaluates
   policy with no ambient filesystem or clock access.
4. std shell persists or displays the resulting attestation and verification
   outputs.

## Verification

- `openspec validate no-std-functional-core`
- `cargo check -p crunch-attestation-core --target wasm32-unknown-unknown`
- `cargo check -p crunch-project-core --target wasm32-unknown-unknown`
- adapter/core tests for both pilot domains, including negative cases
- dependency-boundary check that core crate manifests avoid std-only runtime
  dependencies

## Risks / Trade-offs

**Boundary churn in public APIs** → Mitigation: keep current std-facing crate
names and use re-exports/wrappers first.

**Pure-vs-shell classification mistakes** → Mitigation: start with domains that
already have obvious I/O seams and add explicit inventory work before code
moves.

**no-std target check becomes stale** → Mitigation: make it part of regular
validation, not one-time migration evidence.

**Over-scoping into runtime crates** → Mitigation: keep second-wave candidates
recorded, but out of implementation scope for this change.
