## Why

Mantle can hydrate the ignored Cargo directory source and legacy provider state into a fresh clone, and it separately has a fixed-point self-hosting proof from a prepared checkout. Those facts do not establish that an independently identified handoff contains every source payload consumed by the complete stage0 → stage1 → stage2 proof. The current proof path still permits builtin fetchers to use the network, so a missing source record can remain hidden behind live acquisition.

## What Changes

- Add an operator-facing full-proof source profile that binds fresh-clone Cargo/provider inputs and the complete evaluated fixed-fetch source closure into one externally identified source bundle.
- Add a connected producer path that materializes and fixed-output-validates missing declared fetch sources before bundle publication.
- Add fail-closed self-build source-override mode: every builtin fetch must resolve from pinned imported source state or fail before network I/O.
- Extend the checked-in proof helper and audit bundle so both proof stages use fresh copied source state, report the enforced source-state identity, and reject live-fetch fallback.
- Execute the complete proof from a Git clone that began without `vendor-deps/`, an initially empty Cargo home/source state, and only the externally identified source bundle as source-payload authority.
- Keep claims bounded to that committed source closure, host/tool prerequisites, selected provider, platform, and proof run.

## Impact

- **Files**: `src/source_bundle.rs`, `src/self_build.rs`, `src/main.rs`, `crates/crunch-build/src/fetch_build_service.rs`, `crates/crunch-pipeline/src/lib.rs`, `scripts/prove-self-hosting.sh`, `tests/self_hosting.rs`, operator/proof documentation, machine contracts, ADR 0032, and lifecycle evidence.
- **Testing**: baseline and post-change focused core/CLI tests; missing/extra/tampered source and live-fetch denial tests; proof-helper argument/environment tests; source-bundle contract tests; strict first-party quality, Tiger Style, dependency policy, Nix evaluation, Cairn gates, Tracey, and one full fresh-clone fixed-point execution.
