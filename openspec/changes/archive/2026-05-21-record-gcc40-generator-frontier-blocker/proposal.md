# Record GCC 4.0 generator frontier blocker

## Why

`gcc.4.0` remains partial because selected generator boundary smokes do not prove full native generator correctness. The native boundary receipt already records checked `genattrtab` and `genoutput` slices, but its `native_frontier.blockers` list should explicitly retain the remaining generator-correctness blocker alongside cc1 and demangle blockers.

## What Changes

- Add a checked native-frontier blocker entry for bounded generator output slices.
- Make parity validation fail closed if the generator frontier blocker is omitted or its derivation marker drifts.
- Keep `gcc.4.0` evidence-backed partial/non-promoted.

## Scope

- Update `bootstrap/evidence/gcc-4.0-native-boundary.json`.
- Update `src/bootstrap_parity.rs` fixture/validation/regressions.
- Update canonical bootstrap spec scenarios without replacing prior GCC ladder history.

## Non-goals

- Native/full GCC 4.0 generator correctness.
- New generator output semantics beyond the existing bounded `genattrtab` and `genoutput` slices.
- Promoting `gcc.4.0` to complete for live-bootstrap, Guix, or StageX.
