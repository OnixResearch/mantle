# I9-I12 executable plan evidence

Task-ID: I9, I10, I11, I12
Covers: r[foreign_derivation_import.foreign_builtin_lowering], r[foreign_derivation_import.executable_plan]
Date: 2026-08-01

## Question

Can Mantle emit and validate a deterministic receipt-bound executable plan while it keeps foreign and Mantle digest roles separate?

## Inspected evidence

- `src/foreign_executable_plan.rs`
- `src/foreign_graph_compiler.rs`
- `src/foreign_import_cmd.rs`
- `tests/foreign_import_cli.rs`
- `tests/fixtures/foreign-import/*plan.snapshot.json`
- `docs/foreign-derivation-import-trust-model.md`
- `docs/machine-artifact-contracts.md`
- `schemas/machine-contracts/inventory.ncl`
- Pueue task `7896`: focused core, compiler, plan, CLI, and documentation checks
- Pueue task `7897`: strict first-party Clippy
- Pueue tasks `7898`, `7900`, and `7901`: machine-artifact checker triage and baseline comparison

## Decision

I9 through I12 are complete.

`foreign-import plan` now emits `mantle-foreign-executable-plan-v1` inside the command report. The plan binds the accepted receipt identity, all selected roots, dependency-ordered native units, exact path maps, source requirements, execution profiles, diagnostics, audits, and non-claims.

The plan identity is a role-labeled BLAKE3 digest over every plan field except itself. Each unit records canonical target-prefix ATerm, exact foreign and target paths, a recomputable HDM, native fetch facts, and role-labeled digest facts.

The pure validator checks plan identity, profile identity, dependency order, root coverage, ATerm projections, HDMs, exact maps, source requirements, builtin bounds, digest domains, and leftover foreign references. A failed sibling root returns no successful plan.

Foreign fixed-output SHA-256 values remain in the `foreign-compatible` domain. Mantle HDM and ATerm values remain in the `mantle-target` BLAKE3 domain. The plan identity remains in the `mantle-plan` BLAKE3 domain. Cross-domain substitutions fail closed.

Positive and negative tests cover ordered mirrors, fixed outputs, executable downloads, Git revisions and export policy, malformed hashes, empty candidates, unsupported builtins, digest substitution, plan tampering, projection drift, leftover references, and partial roots.

Exact focused results:

```text
foreign executable plan: 3 passed; 0 failed
foreign graph compiler: 7 passed; 0 failed
foreign import CLI: 12 passed; 0 failed
foreign import trust-model doc check passed
foreign import trust-model checker self-test passed
```

The strict first-party Clippy commands completed with `-D warnings`. Vendored `snix-castore` emitted one existing `dead_code` warning.

The machine-schema checker still reports the same unrelated StageX and source-build inventory debt on baseline commit `36833dcb`. After the new inventory entries, its output contains no foreign import, graph compiler, or executable plan finding.

## Claim boundary

The executable plan proves bounded compilation only. It does not prove source availability, scheduler execution, store admission, realization, output trust, package correctness, or reproducibility.

## Owner

Mantle foreign executable planning core and CLI shell.

## Next action

Run V1 through V3, record final transcripts, synchronize the accepted spec, and archive the change.
