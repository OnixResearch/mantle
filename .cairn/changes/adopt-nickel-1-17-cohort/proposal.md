# Proposal: Adopt the Nickel 1.17 evaluator cohort

## Why

Mantle embeds `nickel-lang 2.0.0` and `nickel-lang-core 0.16.1`. Its Nix command-line tool currently resolves to Nickel `1.16.0`.

Mantle evaluation, diagnostics, direct deserialization, vendored bootstrap sources, and release evidence must use one reviewed evaluator cohort.

## What Changes

- Update the embedded libraries to `nickel-lang 2.2.0` and `nickel-lang-core 0.18.0`.
- Update the command-line evaluator to Nickel `1.17.0` from exact upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`.
- Refresh vendored Nickel sources with the repository-owned import process.
- Preserve bounded diagnostics, direct deserialization, evaluation budgets, and bootstrap source checks.
- Add positive and negative compatibility fixtures for parser, contract, import, diagnostic, and deserialization behavior.
- Record the exact cohort in release and bootstrap evidence.

## Impact

Mantle will use the current reviewed Nickel release for embedded and command-line evaluation. Mantle retains build, store, sandbox, evidence, and release authority.

## Dependencies

This change uses Nickel release `1.17.0`, `nickel-lang 2.2.0`, and `nickel-lang-core 0.18.0`.

## Non-goals

- Do not change Mantle derivation meaning or store identity.
- Do not replace the repository-owned vendoring process.
- Do not update unrelated dependencies only to obtain Nickel.
- Do not claim evaluator correctness or Nix equivalence.
