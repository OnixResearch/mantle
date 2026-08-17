# Design: provider fixed-point release verifier

## Context

The fixed-point proof bundle already writes `meta.json`, `preflight.json`, `non-claims.txt`, per-stage receipts, status files, smoke outputs, and copied stage binaries. The release verifier already has optional deterministic-proof inputs with a required flag. This change follows that model for fixed-point evidence.

## Decisions

### 1. Verify the existing bundle, do not rerun the proof

The verifier reads the fixed-point bundle from disk and validates its durable evidence. It does not rebuild Mantle, run topology execution, or inspect ambient toolchains.

### 2. Keep validation as functional core

Pure validation operates on parsed in-memory summaries, preflight metadata, non-claim text, and file/digest facts. The shell owns reading files, hashing binaries, and printing release verification output.

### 3. Keep claims bounded

A valid fixed-point proof result is a separate JSON field under release verification. It must not make `deterministic_release.eligible` true and must keep `not-release-reproducibility` plus `not-full-cargo-compatibility` present.

## Validation

Run focused `cargo_free_self_build::` tests, release command tests, `cargo build -p mantle --bin mantle`, `git diff --check`, Cairn validate, and proposal/design/tasks gates.
