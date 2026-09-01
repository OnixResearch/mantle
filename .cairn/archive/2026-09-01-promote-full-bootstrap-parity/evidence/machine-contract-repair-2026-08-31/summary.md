# Machine-contract producer classification repair

## Goal

Remove the repository-wide root JSON producer inventory blocker without adding
a catch-all family, misclassifying stable public data as debug output, or
claiming generated document contracts where dedicated validation still owns the
surface.

## Baseline

The machine-contract checker stopped before freshness validation. It reported
53 Rust source files with root JSON serialization but no inventory family
decision.

## Portfolio search

Three correlated classification mechanisms were reviewed.

### One catch-all internal family

This would make the checker green but erase owner and compatibility boundaries.
It was rejected.

### One family per source file

This would preserve path precision but invent dozens of false public API
boundaries. It was rejected.

### Existing owner families plus two bounded compatibility families

This groups only files with an existing product authority:

- StageX, source-built fixed-point, Rust binding, and bootstrap evidence sources
  join `bootstrap.validation-reports`;
- Mantlepkgs version output joins `mantlepkgs.catalog-domain-reports`;
- content-bound and chapter transport output joins `release.evidence-reports`;
- remote credential output joins `remote.execution-reports`;
- the versioned evaluation budget report becomes
  `evaluation.budget-reports`;
- the independently checked operator inventory, catalog, and remediation records
  become `operator.command-contract-reports`.

This route was validated.

## Generated freshness

After coverage became complete, the checker reached one previously hidden stale
contracted producer identity. The official `--generate` mode refreshed the
inventory. No schema, generated contract, or fixture bytes changed.

The final checker reports:

```text
machine schema contract check: PASS (23 contracted, 56 classified)
```

The checker self-test also passes.

## Owner validation

Passed:

- `crunch-eval-budget-core`: 9 tests;
- evaluator budget CLI: 14 tests;
- operator contract core: 20 tests;
- operator contract generation/check pipeline;
- Nickel typecheck and JSON export inside that pipeline;
- `git diff --check`.

The operator pipeline refreshed `config/operator-command-descriptors.json`,
`config/operator-surfaces.ncl`, and `config/operator-surfaces.json` from the
Rust command graph.

## Review checkpoint

- Question: How should 53 unclassified root JSON producers enter the registry?
- Inspected evidence: checker findings, source serialization sites, existing
  inventory families, dedicated evaluator tests, and operator generation rail.
- Decision: extend existing owner families and add exactly two public
  compatibility families.
- Owner: Mantle machine-artifact registry, with evaluation and operator owners
  retaining their dedicated validation rails.
- Next action: commit this repair, then clear the first-party Clippy baseline.

## Non-claims

Classification is not a generated document contract. The compatibility and
internal entries do not freeze every field or promise cross-version stability.
They do not prove evaluation, bootstrap, remote, release, or operator behavior,
authorization, reproducibility, or release eligibility.
