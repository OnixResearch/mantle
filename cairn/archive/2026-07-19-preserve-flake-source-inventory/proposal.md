# Preserve flake self-description in filtered sources

## Why

Mantle's clean Nix `nextest` and `crunch` checks fail before unrelated tests because `tests/examples_inventory.rs` reads `flake.nix`, while the shared `cleanSourceWith` filter omits that file. The exact failure reproduces at both the pre-adoption parent and the clean artifact-auth adoption commit, so it is an independent source-inventory defect.

## What changes

- Admit the repository-owned `flake.nix` file into the shared filtered source.
- Extend the source-inventory test to require that self-description while retaining `.git` exclusion.
- Validate from a clean task-owned commit so unrelated dirty bootstrap work is not included.

## Impact

This affects only Nix source composition and its focused inventory evidence. It does not alter bootstrap/source-seed implementation, artifact-auth behavior, runtime authority, generated locks, or product semantics.
