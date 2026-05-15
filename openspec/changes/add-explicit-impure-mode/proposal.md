## Why

Mantle defaults should stay reproducibility-oriented, but operators still need a
clear escape hatch for development and compatibility cases that intentionally use
host state. Nix has an explicit impure mode; Mantle should provide a similarly
explicit `--impure` mode while making the consequences machine-readable so no
impure result is mistaken for deterministic, cache-safe, or release-proof
material.

## What Changes

- Add an explicit `impure` hermeticity mode selected by `--impure` on build-entry
  commands.
- Keep pure/practical and strict modes as the default proof-oriented path.
- Record impure execution in build results, JSON output, attestations, and proof
  classification.
- Prevent impure outputs from satisfying deterministic/release proof classes
  unless a future policy explicitly accepts an impure class.

## Capabilities

### New Capabilities
- `cli.impure-mode`: `mantle build --impure` and compatible build-entry flags.
- `build-pipeline.impure-mode`: typed handling and reporting for impure builds.

### Modified Capabilities
- `defaults.enforced-sandbox-security`: clarify that `--impure` is a labeled
  reproducibility escape hatch, not a silent weakening of security defaults.
- `release-verification-tech.reproducibility`: impure outputs are proof-blocking
  for deterministic/reproducible claims.

## Impact

- **Files**: CLI option parsing, hermeticity mode enum, build pipeline reporting,
  docs, tests.
- **APIs**: hermeticity mode becomes at least `practical`, `strict`, `impure`.
- **Testing**: CLI conflicts, JSON reporting, release-proof rejection, and docs.
