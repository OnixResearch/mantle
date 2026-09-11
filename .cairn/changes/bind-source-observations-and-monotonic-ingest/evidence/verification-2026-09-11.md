# Evidence: baselines and rails (2026-09-11)

Task-ID: mantle.source_observations.verification
Covers: contract, compatibility, locator_boundary, monotonic_ingest,
        source_observation_binding

## Baseline before implementation (V1)

- `src/source_bundle.rs` §`import_source_bundle` skipped any record whose
  content-identity file already existed (`target.exists()`), without
  comparing stored bytes — a conflicting record under one identity was
  silently accepted as "present".
- Source records carried origin/adapter facts in string maps; no typed
  observation contract existed, and release evidence had no source
  observation binding.
- `crates/crunch-source-core` did not exist; `SourceAcquisition` had no
  `source_observation` field; no machine surface was registered.

## Focused suites (V7)

| Suite | Result |
| --- | --- |
| `cargo test -p crunch-source-core` | 1 + 6 + 9 + 0 doc-tests passed |
| `cargo test -p mantle --bin mantle source_bundle::` | 87 passed |
| `cargo test -p crunch-project` | 7 + 6 passed |
| `cargo test -p crunch-release-core` | 241 + 1 passed |
| `cargo test -p mantle --test release_cli` | 149 passed |

## Full rails (V8)

- `cargo check -p crunch-source-core --target wasm32-unknown-unknown`: clean.
- `cargo fmt --check` for `crunch-source-core`, `crunch-release-core`,
  `crunch-spacewasm-core`: exit 0.
- Tiger Style consumer check: exit 0.
- `cairn validate --root .`: `valid: true`; proposal/design/tasks gates PASS.
- `cairn tracey coverage --root .`: `traceability coverage ok: 155/155
  referenced`.
- Blocked rails (pre-existing, reproduced on a clean `af85ab857` checkout
  and not caused by this change):
  - `./scripts/check-first-party-clippy.sh`: `tests/store_gc_cli.rs` unused
    `meta`, `src/remote_nominal.rs` unused `as_str`.
  - `cargo -Zscript scripts/check-machine-schema-contracts.rs`: 44
    unclassified root-JSON sources under `src/`.
  These remain open for their owners; the new surface itself reports zero
  machine-contract issues.

## Non-claims

Source observations prove declared immutable revision, projection, profile,
and measured content identity only. They do not prove ownership, trust,
source correctness, or release eligibility.
