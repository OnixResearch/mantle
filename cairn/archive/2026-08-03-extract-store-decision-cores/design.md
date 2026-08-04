# Design: Extract Mantle store decision cores

## Context

`crunch-store/src/gc.rs` owns `GcPlan`, reachability classification, filesystem discovery, size calculation, deletion, and database replacement. `crunch-store/src/repair.rs` already has `plan_final_nar_repair`, but the same crate also owns NAR rendering, signing, staging, persistence, rollback, cleanup, and verification.

Mantle's existing small `*-core` crates provide a suitable pattern. GC and repair have separate semantics and mutation authority, so they remain separate mechanisms.

## Goals

- Make GC and repair decisions independent from store and host handles.
- Preserve existing mutation and wire behavior.
- Keep each core small, testable, and suitable for alloc-only compilation.
- Extend canonical architecture and ownership checks instead of adding one-off grep scripts.

## Decisions

### 1. Create two mechanism cores

`crunch-gc-core` owns normalized decisions for:

- retained-root and closure membership;
- live and dead logical path classification;
- retained sidecar and local-result references;
- orphan candidate admission;
- reclaimable-byte summary from supplied size observations;
- ordered mutation intent and dry-run report construction.

`crunch-repair-core` owns normalized decisions for:

- current, repairable, and rejected final-NAR facts;
- exact selector and identity compatibility;
- signature replacement disposition;
- artifact sidecar preservation or refresh disposition;
- dry-run and execution report construction;
- deterministic rollback and verification outcome classification from supplied observations.

### 2. Normalize observations before core admission

The shell maps store-owned types into bounded core DTOs. The GC core does not scan directories or query services. It receives root, graph, path, object, sidecar, retention, and size observations.

The repair core does not render a NAR, read PathInfo, obtain a key, sign, or write a sidecar. It receives recorded and freshly observed final-NAR facts, identity facts, signature facts, sidecar facts, and execution observations.

Core outputs identify logical objects and ordered intents. Shell adapters map those intents back to store paths and concrete service operations.

### 3. Keep mutation authority in `crunch-store`

`crunch-store` retains:

- the cross-process store mutation lock;
- PathInfo, castore, directory, and blob service access;
- filesystem and database scans;
- size and metadata observations;
- NAR rendering and required SHA-256 calculation;
- signing-key access and signature creation;
- file and output deletion;
- staged sidecar writes, database replacement, rollback, and post-write verification.

A core plan does not prove that any effect ran.

### 4. Preserve hash roles

Mantle-owned new plan or fixture identities use BLAKE3. Final-NAR SHA-256 remains because Nix PathInfo and narinfo interoperability require it. Core DTOs retain the algorithm and semantic role instead of treating all digest text as interchangeable.

### 5. Enforce alloc-only core boundaries

Both cores use `#![no_std]` with `alloc` where their dependency closure permits. Public core APIs use owned bounded data and typed errors. Std-facing borrowed convenience, store types, async traits, and host paths remain in adapters.

The maintained no-std inventory, ownership review, Octet topology, and host plus wasm checks include both crates.

### 6. Migrate in independent slices

Repair moves first because its pure planner already exists. GC follows after an inventory freezes every live-root source, including active Rust unit cache retention work.

Each slice freezes current reports and mutation candidates before changing adapters.

## Alternatives

### Add one broad store core

Rejected. GC and repair have different facts, authority, lifecycle, and negative cases. A broad crate would become a policy dumping ground.

### Put async store traits in the core

Rejected. Async service traits move effect authority inward and require mocks to test decisions.

### Keep current module-only functions

Rejected. The current crate dependency graph cannot prevent future pure planning from reaching store or host APIs.

## Risks and Controls

- **Live roots are omitted during normalization**: inventory every accepted retention source and add missing-root negatives before moving GC.
- **Logical intent maps to the wrong concrete path**: use typed logical identities and adapter parity fixtures.
- **Repair wire facts drift**: freeze current reports, final-NAR fields, signatures, and sidecar dispositions.
- **Rollback is overmodeled as pure**: the core classifies supplied rollback observations but never claims rollback occurred.
- **No-std APIs become awkward**: keep borrowed and store-type ergonomics in std adapters.

## Verification

Run baseline and post-change GC and repair tests. Add synchronous positive and negative tests for both cores, shell fault-injection tests for each mutation phase, host and wasm checks, no-std ownership and purity rails, Octet, Tiger Style, Cairn validation and gates, and relevant Nix checks.
