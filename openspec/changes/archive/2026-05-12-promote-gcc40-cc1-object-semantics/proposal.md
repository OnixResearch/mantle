## Why

`gcc.4.0` remains a live-bootstrap and Guix blocker because its installed `cc1` is only a pass1 bridge message. Recent work promoted libgcc member semantics and driver query behavior, but did not prove any installed `cc1` frontend interaction.

## What Changes

- Promote one bounded `cc1` behavior: compile a simple C/preprocessed input to object via the validated TinyCC handoff.
- Add derivation-local smoke that invokes the installed `cc1` directly with GCC-shaped `-quiet <input> -o <object>` arguments.
- Keep the row partial/blocking; this is not full native GCC frontend correctness.

## Impact

- Files: `bootstrap/gcc-4.0.ncl`, GCC placeholder inventory, bootstrap parity tests/specs.
- Verification: eval shell syntax, Crunch build, bwrap cc1 smoke, parity tests/report, OpenSpec validation.
