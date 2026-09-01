# Validation evidence

## Oracle checkpoint

- **Question:** Can Mantle remove all 20 build-layer Tiger Style findings without changing build identity, admission, orchestration, or publication authority?
- **Inspected evidence:** the focused strict baseline, 702 passing pre-change tests, the affected source, accepted build-correctness requirements, and the prior store-boundary evidence.
- **Decision:** use checked local repairs, typed input records, typed error propagation, and bounded iterative traversal. Reject suppression and semantic shortcuts.
- **Owner:** `repair-crunch-build-tigerstyle`.
- **Next action:** repair each source family, rerun focused tests and Tiger Style, then preserve the next exact full-check boundary.

## Outcome checkpoint

- **Question:** Did the structural repair clear the complete build and wrapper package boundary without absorbing later debt?
- **Inspected evidence:** zero build or wrapper findings, 704 passing post-change tests, strict first-party Clippy, caller checks, formatting, both full-check transcripts, and the suppression scan.
- **Decision:** accept the bounded repair. Preserve eight pipeline findings and the independent filtered Nix-source failure as later blockers.
- **Owner:** `repair-crunch-build-tigerstyle`.
- **Next action:** validate, archive, push, and integrate this change without modifying either later blocker.

## Required evidence

- Focused and repository Tiger Style transcripts.
- Positive and negative pre-change and post-change package tests.
- Strict Clippy, formatting, diff, and suppression scans.
- Local-builder and ordinary full flake-check transcripts.
- Cairn validation, Tracey coverage, lifecycle gates, and archive manifest.
