# Current Blocker — Offline source bundle manifest

Date: 2026-06-30

## Question

Can `offline-source-bundle-manifest` be honestly drained in the current pass after the pure/source-state slice?

## Inspected evidence

- `cairn/changes/offline-source-bundle-manifest/tasks.md` still requires:
  - source bundle plan/export over actual selected build-root source/input closures;
  - VCS snapshot materialization;
  - language-neutral package-manager mirror adapters, including Cargo and at least one non-Cargo fixture;
  - bootstrap source archive, provider manifest, toolchain/source-root, and proof-input fixtures;
  - offline build preflight integration before sandbox execution;
  - CLI tests proving no-mutate plan behavior, no-network import/list/verify behavior, and fail-closed stale/missing source preflight.
- `src/source_bundle.rs` now provides a useful first slice: source-bundle manifest model, local declared source specs, BLAKE3 refs, safe path/symlink rejection, source-state import/list/verify, pin files, and CLI wiring.
- That slice does not yet derive required records from Mantle evaluation/build plans or package-manager/provider/toolchain adapters.

## Decision

Still blocked as a complete source/input transport. The current implementation proves an explicit local-record bundle core, but not the spec-required build-root closure planning, adapter fixtures, or offline preflight behavior. Marking the change drained now would overclaim offline build readiness.

## Owner

Mantle source/input transport owner / next implementation pass.

## Next action

Implement in dependency order:

1. connect `source bundle plan` to selected build roots and declared fetcher/local path inputs without mutating state;
2. add VCS snapshot and package-manager mirror adapter records, including Cargo plus one non-Cargo fixture;
3. add bootstrap/provider/toolchain/proof input fixtures;
4. make export consume that plan and fail closed on missing unsupported required material;
5. integrate imported source state into local/remote offline preflight before sandbox or remote dispatch;
6. add CLI and fixture tests for no-network import/list/verify and missing/stale preflight failures.
