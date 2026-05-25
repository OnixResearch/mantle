## Why

`rust-plan --execute-topology` now executes supported mixed host and target Rust graphs from Mantle-owned native planning evidence, but repeated runs still have an ambiguous output story. The current rail can rebuild units and emit receipts, yet it does not make reuse of already-produced topology outputs a first-class, reviewable decision tied to the same explicit unit inputs that justified the original execution.

That leaves cache identity under-specified: a later run cannot reliably explain whether a unit was rebuilt because declared material changed, reused because every identity input still matched, or blocked because prior cached evidence was stale. This change makes topology output reuse fail closed and receipt-bound before broader scheduling or cache-store work.

## What Changes

- Add per-unit prior-output evidence lookup for `rust-plan --execute-topology` under the explicit execution output root.
- Reuse a prior unit output only when the current unit identity, source closure digest, dependency artifact digests, host artifact digests, toolchain identity, `rustc` argument digest, declared output paths, and output artifact BLAKE3 digests match the previous receipt.
- Emit deterministic stale-cache blockers when prior receipts or declared outputs are missing, unreadable, digest-mismatched, or no longer match current explicit input material.
- Preserve bounded CLI JSON evidence that explains `rebuilt` versus `reused` per unit without invoking Cargo or searching ambient caches.
- Add focused positive and negative `rust_plan_cli` coverage for repeated-run reuse and stale cached output behavior.

## Impact

- **Files**: likely `src/rust_plan.rs`, `tests/rust_plan_cli.rs`, and this Cairn change package.
- **Testing**: focused `rust_plan`/`rust_plan_cli` tests, `cargo fmt --check`, Cairn validate, proposal/design/tasks gates, and `git diff --check` before implementation closeout.
