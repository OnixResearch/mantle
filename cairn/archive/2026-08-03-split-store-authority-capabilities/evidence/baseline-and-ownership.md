# Baseline and ownership evidence

## Baseline

The pre-change focused run is in `target/store-authority-baseline-tests.log`.

- `crunch-store`: 292 tests passed.
- `crunch-build`: 690 tests passed.
- `crunch-pipeline`: 27 tests passed.
- Related integration groups also passed in the same log.

Baseline Cairn validation and proposal, design, and tasks gates passed. Their JSON output is under `target/store-authority-*baseline.json`.

## Pre-change call sites

Before migration, `Builder` owned `StoreHandle`. It also returned that handle through `Builder::store_handle()`.

Build orchestration used raw blob and directory services for these operations:

- blob reads;
- castore rewrites;
- NAR calculation;
- closure resolution;
- host-path ingestion;
- fixed-output hashing;
- cache export.

Pipeline orchestration used raw blob and directory services to create fetch and bubblewrap services. It used raw `PathInfoService` access after builds.

The pipeline also called root registration through the builder-owned handle. Source and administrative shells used the same broad handle.

Action-result discovery and publication used stores held by `StoreHandle`. Build-session output nodes, substitution reports, and CA mappings also lived there.

## Accepted ownership

ADR 0058 is accepted. The implementation uses these owners:

| Owner | Capability |
|---|---|
| `Builder` | `BuildStore`, `ActionResultPort` |
| Pipeline build-service wiring | `BuildServiceStore` |
| Pipeline post-build shell | `OutputLookup`, `RootRegistry` |
| Source shell | `SourceAdmission` |
| Operator shell | `StoreAdmin` |

`StoreHandle` remains only as a shell compatibility facade. Production builder code cannot retrieve it.

## Rust-unit and action-result reconciliation

The Rust-unit cache paths continue to use normal build cache and output-admission operations. Their store formats and result identities do not change.

`ActionResultPort` takes the configured action-result stores during capability splitting. It offers discovery, probing, and publication through those fixed stores.

The port has no backend or publisher replacement method. Existing signed shared-result checks remain in the action-result path.

## Local claim

This ownership review covers Rust API reachability and tested behavior. It does not prove sandbox isolation, output correctness, cache trust, or release eligibility.
