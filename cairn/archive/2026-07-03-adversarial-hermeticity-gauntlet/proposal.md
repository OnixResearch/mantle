## Why

Mantle’s strict hermeticity and protected-exec paths are promising, but they need adversarial tests that intentionally introduce hidden tools, ambient environment, network access, timestamps, locale changes, umask changes, random temp paths, and undeclared store paths. The goal is to prove Mantle fails closed or records the impurity rather than silently producing overbroad evidence.

## What Changes

- Add an adversarial hermeticity gauntlet profile with declared perturbation axes and expected outcomes.
- Exercise strict/practical modes, protected-exec supervision, network policy, environment normalization, umask/timestamp controls, and reference scanning.
- Emit deterministic gauntlet reports that separate accepted clean cells from rejected or degraded cells.
- Require release/global reproducibility evidence to cite gauntlet results only within the tested axes.

## Impact

- **Files**: hermeticity fixtures, gauntlet runner/report, strict-mode tests, docs, Cairn verification-evidence spec delta.
- **Testing**: positive clean strict build, negative hidden host exec, negative network attempt, negative ambient-env leak, Cairn validation/gates.
