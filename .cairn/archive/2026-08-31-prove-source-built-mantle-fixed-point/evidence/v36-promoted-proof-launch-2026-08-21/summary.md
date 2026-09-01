# V36 promoted source-built proof launch

## Question

Is source commit `082bde0f` ready for a fresh promoted proof on Leviathan?

## Inspected evidence

- Remote pueue task `269` completed a strict preserved-provider Cargo-free fixed point.
- Both diagnostic stages completed 789 units and produced matching Mantle binaries.
- The source transfer used root-anchored exclusions and exact rsync checksum parity.
- Profile refresh task `270` produced and verified the paired source/vendor profile.
- Task `270` failed only in an operator postcondition that used the wrong report field name.
- Follow-up task `271` checked the correct v2 fields and confirmed exact refresh/verify digest parity.
- The profile BLAKE3 is `3cffb96f2d7a51a13b2c2d1e4131771edee340d72213c949ebd761defd0a84a1`.
- Task `276` removed only two derived Rust-provider scratch directories.
- It retained both staging roots, final providers, StageX evidence, plans, logs, and attempt status files.
- Leviathan had `783605526528` free bytes before launch.
- The immutable proof disk bound remains `700000000000` bytes.

## Decision

Launch a fresh strict proof with no substitution and 16 jobs. Do not reuse a provider cache or resume state.

Remote pueue task `277` started at `2026-08-21T22:12:20-04:00`. The output path was absent before launch.

## Owner

The Mantle source-built fixed-point change owns the proof and final receipt review.

## Next action

Monitor task `277` with `pueue_wait`. Preserve its output, hidden staging root, log, final task record, and `attempt-status.json`.

## Non-claims

Launch evidence does not prove completion. Only a verified v2 receipt with matching stages can close I5 and V2.
