# Tasks: Run PR CI and align Clippy target coverage

## Phase 1: Trigger and coverage

- [x] [serial] T1.1 Add `pull_request` to the read-only flake-check workflow. r[ci_alignment.pull_request]
- [x] [serial] T1.2 Change flake Clippy to `--workspace --all-targets` with canonical vendored exclusions. r[ci_alignment.clippy_targets]

## Phase 2: Verification

- [x] [serial] T2.1 Evaluate the flake Clippy derivation and record it. r[ci_alignment.clippy_targets]
