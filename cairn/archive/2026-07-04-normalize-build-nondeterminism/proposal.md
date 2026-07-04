## Why

Hermetic inputs are not enough if time, locale, umask, temp paths, host/user metadata, randomness, or parallel output ordering can still perturb results. Mantle needs a deterministic normalization policy for strict builds and proof runs, plus diagnostics that block strong claims when normalization is unsupported or output divergence is observed.

## What Changes

- Normalize strict build defaults for timestamps, timezone, locale, umask, temp roots, host/user metadata, random seeds where modeled, and order-sensitive output processing.
- Bind the selected normalization policy into action and proof receipts.
- Detect unsupported normalization and output divergence as strong-claim blockers.
- Add perturbation fixtures that vary ambient nondeterministic inputs while expecting identical admitted evidence.

## Impact

- **Files**: build-request normalization, proof sandbox envelope, output/report classifiers, docs, and Cairn build-correctness spec delta.
- **Testing**: positive equivalent-build convergence fixture; negative unsupported-normalization and divergent-output fixtures; Cairn validation and gates.

## Out of Scope

- Proving deterministic behavior for compilers or tools that intentionally embed randomness outside Mantle's modeled controls.
- Changing practical mode's diagnostic convenience defaults.
