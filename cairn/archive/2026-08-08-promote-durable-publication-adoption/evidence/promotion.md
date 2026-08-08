# Promotion evidence

## Identities and authorization

- Canonical Mantle target: `df79e1b7b546b1d67de0751a60862bf609efc51c`
- Reviewed Mantle adoption: `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf`
- Exact merge candidate: `aa577374516e6b15c3c4ef59c71c74410c0d0fab`
- Candidate first parent: `df79e1b7b546b1d67de0751a60862bf609efc51c`
- Candidate second parent: `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf`
- User authorization: the user approved the revised canonical promotion and subsequent broad validation in the current request.

A fresh pre-push fetch confirmed that Mantle `origin/main` still named the canonical target. The candidate contained both required Mantle histories.

A fresh Onix Core fetch resolved `origin/main` to `bc4629c9e766d3db82e4dab9fe8c166c360b8435`. Both that reconciliation commit and accepted admission `b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed` were ancestors.

## First-parent scope

`git diff --name-only df79e1b7b546b1d67de0751a60862bf609efc51c..aa577374516e6b15c3c4ef59c71c74410c0d0fab` reported only:

```text
cairn/changes/promote-durable-publication-adoption/design.md
cairn/changes/promote-durable-publication-adoption/proposal.md
cairn/changes/promote-durable-publication-adoption/specs/durable-publication-promotion/spec.md
cairn/changes/promote-durable-publication-adoption/tasks.md
evidence/radicle/durable-file-publication-adoption-v1.blake3
evidence/radicle/durable-file-publication-adoption-v1.json
evidence/radicle/durable-file-publication-adoption-v1.ncl
lib/durable-file-publication-adoption-receipt.ncl
```

The receipt and validator changed only the four canonical file BLAKE3 bindings and the receipt digest. Producer identity, mapping, validation record, authority boundary, and non-claims stayed unchanged.

## Validation

The matrix ran on the candidate tree.

| Check | Result |
|---|---|
| `nix develop -c cargo test -p mantle --bin mantle remote_attempt_log_store -- --test-threads 1` | PASS: 14 passed |
| `nix develop -c cargo test -p mantle --bin mantle -- --test-threads 1` | FAIL: 2232 passed, 3 failed, 68 ignored |
| `nix develop -c cargo build -p mantle --bin mantle` | PASS |
| built Mantle `--help` probe | PASS |
| `nix develop -c cargo fmt --check -p mantle` | FAIL: `src/source_built_fixed_point_shell.rs` |
| `nix develop -c cargo clippy -p mantle --bin mantle --no-deps -- -D warnings` | PASS |
| `./scripts/check-first-party-tigerstyle.sh` | FAIL: existing findings outside the durable-publication surface |
| adoption receipt Nickel typecheck | PASS |
| positive and negative adoption Nickel tests | PASS |
| `nix build .#checks.x86_64-linux.durable-file-publication-adoption -L --option secret-key-files ''` | PASS after bounded receipt refresh |
| strict Cairn validation | PASS: no issues or findings |
| Cairn proposal, design, and tasks gates | PASS |
| focused Tracey coverage | PASS: 155 of 155 requirements referenced |

The first focused Nix run exposed stale canonical hashes in the adoption receipt. The candidate refreshed only those hashes and the matching exact validator constants. The second focused Nix run passed.

The three binary failures were:

```text
bootstrap_parity::tests::binutils_tcc_real_derivation_reports_independently_receipted_completion
bootstrap_parity::tests::gcc40_real_derivation_reports_independently_receipted_completion
bootstrap_parity::tests::gcc47_real_derivation_reports_cxx_contract_backed_partial
```

These failures, the fixed-point formatting failure, and the Tiger Style findings are pre-existing broad blockers. The first-parent diff contains no Rust implementation path, so this ancestry and receipt-only candidate did not change their implementation surfaces. `restore-durable-publication-broad-validation` retains ownership of these failures. This evidence does not claim that those checks passed.

## Push and remote verification

The branch and canonical pushes used normal `git push` operations. No force option was used. After a final fetch:

```text
candidate=aa577374516e6b15c3c4ef59c71c74410c0d0fab
remote=aa577374516e6b15c3c4ef59c71c74410c0d0fab
push_and_remote_ancestry=PASS
```

Mantle `origin/main` contains the reviewed adoption commit. The durable RID remains `rad:z3tAR4For7qw8ZirkJzoDw1VNDDLM`, and the producer revision remains `951c27f59003cea9bfdb40ed4d89653d50fada1f`.

## Non-claims

This promotion proves reviewed ancestry, bounded receipt freshness, focused adoption validation, and non-destructive canonical placement. It does not prove the three bootstrap-parity tests, repository formatting, whole-tree Tiger Style, whole-Mantle correctness, fixed-point success, full-source completion, or release eligibility.

## Archive validation

Legacy Cairn created `cairn/archive/1970-01-01-promote-durable-publication-adoption`. The operator renamed it to `cairn/archive/2026-08-08-promote-durable-publication-adoption` to record the session date, as required by the repository archive procedure.

Exact post-archive command output is in:

- `evidence/post-archive-validation.json`
- `evidence/post-archive-tracey.json`

The strict validation output has `"valid": true`. The Tracey output has `"verdict": "pass"` and reports 155 of 155 requirements referenced.
