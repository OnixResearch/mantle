# Validation evidence

Date: 2026-07-14

## Legacy-authority compatibility cycle

Mantle retained `CutoverAuthority::Legacy` while the initial complete cycle ran.

- pueue 459: focused `nickel_export` suite passed: `14 passed; 0 failed`.
- pueue 525: machine-contract generation/check passed for `16 contracted, 45
  classified`; the exact source-pin check passed.
- pueue 531: strict root-package Clippy completed under
  `-p mantle --bin mantle --tests --no-deps -- -D warnings`. The only printed
  warning came from vendored `snix-castore`; no Mantle warning or error remained.
- pueue 476: locked root binary check completed and `cargo tree --locked -p
  mantle -i nickel-export-core` resolved only
  `nickel-export-core v0.1.0 (...?rev=257faf...#257fafc1)` into Mantle.
- pueue 478: the Mantle `release_evidence::` filter and
  `crunch-release-core --lib` completed successfully.
- pueue 536: the first local Nix source-pin derivation completed successfully.
- pueue 603: Cairn validate and proposal/design/tasks gates completed
  sequentially with exit status zero; the visible final tasks receipt had no
  issues, `valid: true`, and `verdict: PASS`.

The earlier Nix-wrapped Cairn task 477 produced a passing proposal receipt, then
stalled rebuilding Cairn in the shared global queue. It was terminated without
being treated as evidence for the remaining stages; pueue 603 used the current
canonical sibling Cairn binary for all four stages.

## Canonical-authority replay

After the complete legacy cycle, `CANONICAL_CUTOVER_VALIDATED` was changed from
`false` to `true` while the explicit legacy selector remained available.

- pueue 650 exercised the built CLI with the checked evidence fixtures:
  - positive export: status `0`;
  - lexical path escape: status `3`;
  - evaluator error: status `3`;
  - conservative secret marker: status `3`;
  - source symlink: status `3`;
  - output symlink: status `3`.
- pueue 681 reran formatting, machine-contract generation/check and negative
  self-test, exact pin check and negative self-test, focused export tests,
  strict Clippy, locked binary/dependency checks, Mantle release-evidence tests,
  and `crunch-release-core --lib`; the sequential command completed with exit
  status zero. The visible final core result was `207 passed; 0 failed`.
- pueue 699 rebuilt the final source-pin Nix derivation locally with remote
  builders disabled. It verified the exact repository/revision, reran the pin
  checker's negative self-test, evaluated the typed Nickel source record, and
  compared its JSON export byte-for-byte with the generated artifact.
- Final fail-closed identity audit: pueue 744 regenerated and checked the
  machine contract, reran focused export tests and strict Clippy, and completed
  `git diff --check`; pueue 791 then recorded the focused result as `15 passed;
  0 failed`, including the negative unexplained-identity/serialization-drift
  regression.
- Final post-task lifecycle evidence:
  - pueue 716: Cairn validate, `7` changes, `30` specs, no issues, `valid: true`;
  - pueue 715: proposal gate PASS, receipt
    `49d1ce58e98886bb4ba36cfb122c161efaa3cb3dac686184f0a85a54f5b9988d`;
  - pueue 714: design gate PASS, receipt
    `312a6072cc55a3acaf0350689f1e245c2d7ad6758c5c64d910e3f6d84df240b3`;
  - pueue 717: tasks gate PASS, receipt
    `13970bbf615b1890298b950c8b40fe8bd6420b18f0acc53e17a063fa63c1b385`.

## Negative coverage

The focused Rust and CLI evidence covers lexical escape, source/destination
symlinks, source mutation after capture, stale output bytes, manifest/output
tamper, mixed evaluator descriptors, evaluator failure without a receipt,
secret-marker admission, and overclaim rejection. Exact one-receipt manifest
identity and the full Mantle v1 projection are asserted in the positive case.

## Boundaries and blockers

No lifecycle sync or archive command was run. Full self-hosting/release bundle
creation was not claimed; this isolated worktree does not contain its ignored
`vendor-deps/` payload. Release-relevant pin material and focused release core /
shell checks were proven instead.

A supplemental package-level tigerstyle invocation reached the repository's
known existing first-party backlog and failed in unrelated modules (153 reported
errors, with the visible tail in `src/protected_exec_seccomp.rs`). It is not one
of this change's acceptance rails; strict touched-package Clippy and focused
functional-core/shell tests passed.

Export evidence remains bounded: it does not prove evaluator equivalence,
complete observed import closure, semantic/build correctness, deployability, or
release eligibility.

## Post-archive validation

The accepted specification was synchronized before archive. The exact
post-archive validation command completed successfully:

```text
$ /home/brittonr/git/OnixResearch/cairn/target/debug/cairn validate --root .
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 30,
  "valid": true
}
```
