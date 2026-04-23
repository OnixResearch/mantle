Evidence-ID: delta-functional-core-proposal-validation-2026-04-22
Task-ID: proposal
Artifact-Type: validation
Stage: proposal
Date: 2026-04-22
Covers: functional.core.dedicated.nostd.crates.third.wave

## Commands

- `openspec validate delta-functional-core`
- `openspec_gate stage=proposal change=delta-functional-core`

## Results

### `openspec validate delta-functional-core`

```text
Change 'delta-functional-core' is valid
```

### `openspec_gate stage=proposal change=delta-functional-core`

```text
VERDICT: FAIL
```

Current proposal-gate findings after initial artifact write:

- proposal-stage evidence file was missing before this artifact was added
- same-family proposal review packet did not include `design.md` / `tasks.md`
  even though both files exist in `openspec/changes/delta-functional-core/`
- semantic-parity and named negative-case verification were underspecified in
  the first draft and were tightened after the initial rerun

## Notes

This artifact captures the required proposal-stage validation transcript before
implementation. `design.md` and `tasks.md` now exist in the change directory,
and follow-up design/tasks gates should be used to judge those artifacts
separately from the proposal packet.
