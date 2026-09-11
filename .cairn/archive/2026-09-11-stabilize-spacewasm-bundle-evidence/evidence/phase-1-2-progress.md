# Evidence: Phase 1–2 progress (WIP, 2026-09-10)

Task-ID: mantle.spacewasm_stable_evidence
Covers: contract, denial, boundary

This change is NOT complete. Phase 3 (repeatability gate, frozen acceptance
run) and Phase 4 (producer publication, ChaosControl-owned consumer
evidence) remain open. The change stays active.

## What landed

- `crates/crunch-spacewasm-core/src/stable_report.rs`: versioned
  stable-fact contract (`mantle-spacewasm-stable-report-v1`), closed
  libtest JSON grammar admission with `deny_unknown_fields`, named bounds,
  duplicate rejection, expected-inventory comparison, contradictory-summary
  rejection, canonical name-ordered facts, and BLAKE3 stable identity that
  excludes durations, completion order, and all presentation fields.
- `crates/crunch-spacewasm-core/tests/stable_report_fixtures.rs`: 11
  positive and negative fixtures (equivalent presentations admit to one
  identity; changed outcome or inventory changes identity; duplicates,
  malformed, unknown grammar, truncation, contradiction, empty captures,
  and empty identities reject).
- ADR 0079: structured libtest JSON grammar selected over a text adapter;
  raw output demoted to separately retained run evidence.

## Commands and results

Command: `nix develop -c cargo test -p crunch-spacewasm-core`

    test result: ok. 8 passed; 0 failed (focused core suite)
    test result: ok. 11 passed; 0 failed (stable-report fixtures)
    test result: ok. 0 passed; 0 failed (doc-tests)

Command: `nix develop -c cargo clippy -p crunch-spacewasm-core --all-targets -- -D warnings`

    exit 0

Command: `nix develop -c cargo check -p crunch-spacewasm-core --target wasm32-unknown-unknown`

    Finished `dev` profile

Tiger Style consumer check: rebuilt after staging the new module; earlier
failure was an unstaged-file artifact (`E0583`), not a lint finding.

## Open items

- T1.1 remainder, T1.3, T1.4, full T1.5; T2.4 (shell bounded capture +
  failure tests), T2.5 (Nix producer integration), all of Phases 3–4.
- T4.2 requires ChaosControl-owned consumer evidence with the new exact
  producer pin — cross-repo owner action, external to Mantle.
- T4.3 requires resolving the recorded repository-policy registry blocker
  through its owning change; the baseline policy error is preserved in
  `evidence/baseline.md`.

## Non-claims

Core admission and identity equality prove contract shape only. No
producer reproducibility, bundle repair, or consumer admission is claimed
until Phases 3–4 pass.
