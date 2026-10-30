# T4.1 retrospective waiver and current-tree after observation, 2026-10-30

Task T4.1 (`tasks.md`): "Run the edit, delete-during-build, invalid-source, and coalescing
rails before and after the change. Preserve exact results." Requirement
`r[build_scheduling.watch_retraction_cancellation]`.

## Decision

The operator authorized accepting T4.1 on its after side with the before side waived
([operator authorization](../../reconcile-main-api-lineages-2026-10-17/evidence/operator-authorization-2026-10-30.md), row 6). This file records the basis. It changes nothing
that was observed, and it claims no historical pass.

## Why the before side cannot be run

`before-after-observations-2026-10-27.json` ran the same smoke driver against the genuine
pre-watch source, commit `da00f58425740adea559ef926c9dfa97b2cb8240`, built unmodified. That product rejected
`--watch` before initial settlement, so none of the four scenarios started. The scenarios
exist only because the feature does; a before run of them has no meaning, so none is
executed here and none is claimed.

## After side, executed on the current tree

| Item | Observation |
|---|---|
| Subject | commit `1776617592987af585e48e708961c962cde3407e`, tree `2e907c0c611080595e0bb882a4b430e7a9732ab4`. `git status --porcelain` was empty before the build. After the smoke it listed eight documentation files only (ADR status text, three `tasks.md` files, `adr/README.md` and the authorization record), uncommitted at that moment; no source or script file differed from the commit |
| Binary | `mantle` from `cargo build --locked --offline -p mantle --bin mantle` (exit 0); BLAKE3 `b599e0ca5ad3b6bd6327bcb2675e5e62b6b420063a9cc6f5cf5839beafc9e40d` |
| Smoke | `python3 scripts/watch-goal-retraction-smoke.py --mantle <binary> --scratch <fresh dir>` exit 0; result status `passed` |
| Assertions | one-root edit; added root; invalid imported source retention; coalesced replacements; running-root deletion; descendant teardown; no late output |
| Teardown | descendants stopped: `true`; late-write observation 21 s; cancelled output present: `false` |
| Core tests | `cargo test --locked --offline -p crunch-watch-core -- --test-threads 1`: `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` |

Raw outputs: `retrospective-waiver-2026-10-30-captures.json` (BLAKE3 `423f8dfc8541b7df5619bdbde39d18f5bfc985af96156e0b2e1cc45abb24ab20`) retains the complete result of the build run and of the smoke and core-test run, with the exact stdout and stderr and a BLAKE3 for each stream, including the source-identity and status steps. Absolute paths inside it name the original run directory.

The smoke driver (BLAKE3 `da0d23c40a76de6dc19a76139bb6ce824984085c878bb1c0d87bed888430bd13`) is byte-identical to the instrument
used in the historical-before attempt. The raw result is
`watch-after-current-tree-2026-10-30.result.json` (BLAKE3 `ed8ed179fb206717fe6fd9953c40d4b5efd89487462a024de5475b5930274366`).

## Scope

Accepted: T4.1, as a failing-before (feature absent) and passing-after capability
comparison, with the after side observed on the current tree.

Not claimed: any executed historical edit, delete-during-build, invalid-source or
coalescing scenario; whole-tree source identity beyond the clean commit and the binary
BLAKE3 above; anything about T4.3 (spec sync and archive), which is a separate lifecycle
step.
