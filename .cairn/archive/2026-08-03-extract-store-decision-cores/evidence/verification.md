# Store decision-core verification

Date: 2026-08-03
Baseline revision: `cb27a0442047a78c0770acb35cad6b3daf0ec146`

## Result

The focused acceptance checks pass. Mantle now uses two alloc-only cores for selected GC and final-NAR repair decisions.

`crunch-gc-core` owns bounded graph normalization, reachability, live and dead classification, checked reclaim summaries, ordered path intents, dry-run disposition, and BLAKE3 plan identity.

`crunch-repair-core` owns current, repair, and rejection decisions. It also owns target-bound BLAKE3 plan identity, transaction intents, rollback intents, and report classification.

`crunch-store` remains the imperative shell. It owns service reads, scans, NAR rendering, SHA-256 calculation, signing, deletion, persistence, verification, cleanup, and rollback execution.

## Focused test evidence

The final focused test rail passed. Its log is `target/store-cores-final-tests.log`.

- `crunch-gc-core`: 11 passed.
- `crunch-repair-core`: 7 passed.
- `crunch-store gc::`: 13 passed.
- `crunch-store repair::`: 11 passed.
- `mantle --test store_gc_cli`: 3 passed.
- `mantle --test integration store_repair_final_nar`: 3 passed.
- `cargo fmt --all -- --check`: passed.

The GC fixtures cover deterministic order, cycles, overlapping roots, dry-run parity, missing facts, duplicate identities, path validation, and arithmetic overflow.

The repair fixtures cover current facts, repairable facts, invalid identities, incomplete content, invalid signatures, sidecar policy, rollback intents, and incomplete execution reports.

The shell fixtures preserve exact-path selection, action-result retention, explicit castore roots, sidecar behavior, mutation order, dry-run candidates, and accepted CLI reports.

## Architecture and quality evidence

The following focused checks passed:

- Both cores passed `wasm32-unknown-unknown` checks. See `target/store-cores-wasm-final.log`.
- Both cores passed `cargo octet check`. See `target/store-cores-octet.log`.
- Both cores passed focused Tiger Style checks. See `target/store-gc-tiger.log` and `target/store-repair-tiger.log`.
- `crunch-store` has no Tiger Style finding in `gc.rs` or `repair.rs`. See `target/store-shell-tiger.log`.
- Both cores and `crunch-store --lib` passed strict, no-dependency Clippy. See `target/store-cores-quality-final.log`.
- Dependency, purity, and scope checks passed. See `target/store-cores-no-std-static.log`.
- The maintained core inventory, dependency allowlist, ownership review, workspace membership, and Rust target list include both cores.
- Cairn validation and all three lifecycle gates passed.

Cairn gate receipts:

- Proposal: `cebc91339f25b959ff877e8edf0fccc30081beda60ceb8254e9920d0cc1ed733`
- Design: `fe36546c6d3bca65f2120116fb8b304de9467bb8e4258bec2164c918449a6335`
- Tasks: `8d81839628de5de2fa508adaaa67ff54fe351d683e3478180e96f226f9f7ee5f`

## Repository-wide baseline limits

The full first-party quality rail reached Clippy, then failed in unchanged `crunch-store/src/provenance.rs` code. The two findings concern `manual_is_multiple_of`. See `target/store-cores-first-party-quality.log`.

The full Tiger Style rail retains unrelated findings in `provenance.rs`, `pull.rs`, and other existing files. It reports no finding in the changed GC or repair adapters. See `target/store-cores-first-party-tigerstyle.log`.

The full no-std wrapper cannot detect the wasm target in its preinstalled-toolchain probe. Direct wasm checks passed in the detached wasm-enabled development shell. Static API-shape and ownership checks retain unrelated existing findings. Neither new core appears in those findings.

Traceability coverage retains three unrelated release-provenance gaps:

- `mantle.release_provenance.content_bound_evidence_manifest`
- `mantle.release_provenance.content_bound_requirement_coverage`
- `mantle.release_provenance.legacy_coverage_boundary`

`nix flake check -L` remains blocked while fetching `https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git`. See `target/store-cores-nix-flake-check.log`.

## Claim boundary

A core plan is a deterministic decision over supplied facts. It is not an execution receipt.

The plans do not prove observation truth, content correctness, signer authority, provenance, reproducibility, release eligibility, source trust, or cache availability.

A GC dry-run preserves the execution candidate set but grants no mutation authority. A repair success report requires execute mode and explicit execution completion.
