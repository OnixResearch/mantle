## Why

Operators need one current, copyable guide for Mantle proof work: prerequisites, commands, expected outputs, evidence bundle locations, how to interpret blockers, and what the proof does not claim. Without that guide, proof knowledge stays in task transcripts and agent memory.

## What Changes

- Add an operator guide for self-build, Cargo-free fixed-point, and Nix-free demo bundle validation.
- Include prerequisites, environment setup, exact commands, output bundle interpretation, and cleanup guidance.
- Explain success claims and non-claims in the same language as the evidence validators.
- Add a drift check or test that keeps the guide aligned with CLI help and proof bundle fields.

## Impact

- **Files**: README/docs, CLI help snippets if needed, proof guide tests or drift checks, Cairn verification-evidence spec delta.
- **Testing**: docs link/path checks, command-snippet sanity checks, negative stale-command fixture, Cairn validation/gates.
