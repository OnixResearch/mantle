## Why

Native Rust topology now runs `aws-lc-sys` past package metadata and target cfg env blockers, but cc-rs stops because Cargo profile runtime variables are absent. The current clean probe shows `OPT_LEVEL not defined`.

## What Changes

- Derive bounded Cargo-compatible build-script profile variables from Mantle's selected profile.
- Provide deterministic `OPT_LEVEL`, `DEBUG`, and `NUM_JOBS` to native build-script child execution.
- Record whether the self-probe moves past the missing `OPT_LEVEL` blocker.

## Impact

- **Files**: `src/rust_plan.rs`, Cairn change/evidence files.
- **Testing**: focused `rust_plan::` tests, clean self-probe, `cairn validate --root .`, `git diff --check`.
