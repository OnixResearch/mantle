# Extract Mantle store decision cores

## Why

Mantle uses compiler-enforced functional cores for several mechanisms. Store garbage collection and final-NAR repair still place pure decisions beside store queries, directory scans, file metadata reads, deletion, signing, database replacement, staging, rollback, and async orchestration in `crunch-store`.

The current functions contain useful seams. `plan_final_nar_repair` is already pure, and GC already separates several classifiers from mutation. Small mechanism cores can make those boundaries compiler-enforced without creating one broad store-policy crate.

## What Changes

- Add `crunch-gc-core` for normalized reachability, retention, candidate, reclaim-summary, and mutation-plan decisions.
- Add `crunch-repair-core` for final-NAR repair admission, no-op, rejection, signing disposition, sidecar disposition, and report decisions.
- Keep store discovery, castore and PathInfo access, filesystem scans, size observations, mutation locks, deletion, database replacement, NAR rendering, signing, staging, rollback, and post-write inspection in `crunch-store`.
- Use explicit normalized observation DTOs at each core boundary.
- Preserve current CLI behavior, dry-run behavior, store identities, accepted Nix SHA-256 fields, BLAKE3 identities, mutation order, and reports.
- Extend Mantle no-std, Octet, ownership, and positive and negative fixture rails to the new cores.

## Capability Delta

A new `store-lifecycle` spec domain defines compiler-enforced GC and repair decision boundaries, compatibility, architecture enforcement, and bounded claims.

## Impact

- **Core source**: pure parts of `crates/crunch-store/src/gc.rs` and `crates/crunch-store/src/repair.rs`.
- **Shell source**: `crunch-store` service access, scans, NAR rendering, signing, mutation, rollback, and persistence.
- **Dependencies**: the two cores use `no_std` plus `alloc` where practical and consume normalized Mantle-owned facts instead of store handles or host paths.
- **Active work**: GC planning must preserve Rust unit result retention and any other accepted live-root authority added by active changes.
- **Public behavior**: no command, JSON, store path, signature, final-NAR field, sidecar, or deletion behavior changes without a separate versioned requirement.

## Non-Goals

- Combining GC and repair into one broad `crunch-store-core` crate.
- Changing reachability policy, retention authority, deletion order, mutation locking, rollback authority, or repair eligibility.
- Replacing required Nix final-NAR SHA-256 fields with BLAKE3.
- Moving filesystem, castore, PathInfo service, signing, or database authority into a core crate.
- Claiming content correctness, deletion safety, trust, sandboxing, or release readiness from planning tests.

## Verification Expectations

Implementation must run focused GC and repair tests before core changes. It must then run host and `wasm32-unknown-unknown` core checks, positive and negative core tests, shell integration and fault tests, compatibility fixtures, Mantle no-std and Tiger Style rails, Cairn validation and gates, and the relevant Nix checks.
