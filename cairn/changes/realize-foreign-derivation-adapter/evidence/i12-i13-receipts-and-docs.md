# I12-I13 realization receipts and documentation

Task-ID: I12, I13
Covers: r[foreign_derivation_import.realization_receipt]
Date: 2026-08-02

## Receipt

`mantle-foreign-realization-receipt-v1` has a domain-separated BLAKE3 identity.
It binds these facts:

- executable-plan, import-receipt, source-bundle, and build-report identities;
- selected root node IDs and exact target paths;
- execution-profile IDs and BLAKE3 values;
- cache mode and ordered receipt-bound cache URLs;
- source and output PathInfo NAR observations;
- per-unit built, fetched, substituted, reused, failed, or blocked disposition;
- ordered fetch attempts, affected failed roots, and completed unit count;
- `realized` or `partial-realization` strongest state;
- explicit evaluator, package, reproducibility, output-trust, and remote-authority non-claims.

A complete receipt can supply one selected root to `store pull --closure`.
The pull path verifies schema, status, strongest state, self-digest, and root count before store mutation.
Normal cache signature, NAR, prefix, and PathInfo checks still apply.

## Documentation

Updated files:

- `README.md`;
- `docs/foreign-derivation-import-trust-model.md`;
- `docs/foreign-realization-operator-guide.md`;
- `docs/machine-artifact-contracts.md`;
- `schemas/machine-contracts/inventory.ncl`.

The operator guide covers planning, source preparation, local realization, receipt review, and cache hydration.
It keeps the OnixOS system-assembly and boot boundary explicit.

## Failure rule

Preflight rejection writes no realization receipt and makes no store change.
A failure after execution starts writes partial evidence when the worker returns bounded outcomes.
No receipt reports a stronger state than its completed evidence supports.
