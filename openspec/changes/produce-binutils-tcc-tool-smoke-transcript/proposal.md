## Why

`binutils.tcc` now has a fail-closed parity gate, but no checked transcript exists yet. The previous bounded probe built the output but could not smoke the tools because the probe environment lacked the logical `/crunch/store` closure. We need a narrow evidence-producing change that either promotes the row with real tool smokes or records the next blocker with enough detail to avoid repeating the same closure mistake.

## What Changes

- **Transcript generation**: produce `bootstrap/evidence/binutils-tcc-tool-smoke.json` only from real `bootstrap/binutils-tcc.ncl` output smokes.
- **Closure-aware probe**: run smokes in a bwrap environment with `/crunch/store` bound to the complete scratch store needed by the output scripts/tools.
- **Parity status**: keep the row blocked unless the transcript satisfies the existing checked schema.

## Capabilities

### Modified Capabilities
- `bootstrap.parity.binutils-tcc-evidence`: attempts to satisfy the checked-transcript gate with real runtime evidence.

## Impact

- **Files**: `bootstrap/evidence/binutils-tcc-tool-smoke.json`, possibly narrow parity validation fixes if the transcript schema is too strict/loose.
- **APIs**: no public API change.
- **Dependencies**: no new dependency.
- **Testing**: OpenSpec strict validation, parity unit/CLI tests, `crunch bootstrap parity-report --json`, and bwrap tool smokes.
