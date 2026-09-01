## Context

The focused command
`nix run .#tigerstyle -- check -- -p crunch-pipeline` reports eight findings in
one orchestration file. The accepted build-layer repair moved the repository
gate to this boundary without suppressing it.

`crunch-pipeline` coordinates evaluation, build-service composition, worker
execution, root registration, evidence collection, and failure normalization.
The pure policy is limited, but collection bounds and registration plans can
remain deterministic before the shell performs effects.

## Portfolio Search Contract

- **Goal:** make focused and repository Tiger Style checks report zero
  `crunch-pipeline` findings with accepted pipeline behavior intact.
- **Completion evidence:** zero focused findings, a repository Tiger result that
  advances beyond this package, 58 pre-change and at least 58 post-change tests,
  strict Clippy, formatting, caller compilation, and full-check evidence.
- **False completion:** lint allowances, narrower targets, warning budgets,
  finding baselines, deleted negative coverage, changed root classes, changed
  effect order, or an unbounded collection hidden behind a helper.
- **Audit risks:** dropped or duplicated eager roots, premature source-root
  publication, changed substitution source labels, reordered retained-root
  effects, changed cache-only trust, and failed-key prefix drift.
- **Budget:** current repository and pinned Tiger input only; package-local
  source and tests exposed after early failures may clear; no new dependency or
  runtime authority; at most ten focused Tiger rounds.
- **Allowed outcomes:** validated, exact later blocker, exhausted round budget,
  or a user decision for incompatible public behavior.

## Approach Registry

### Family: bounded functional planning

**Mechanism:** pass the admitted root count into eager collection. Derive a
checked registration capacity from supplied outcomes and source paths. Build a
deduplicated registration plan before root-registry I/O.

**Claim:** both growth findings can close while preserving element order,
deduplication, root classes, and commit timing.

**Artifact:** pure plan tests, existing managed-generation tests, and exact
focused Tiger output.

**State:** selected.

### Family: named orchestration inputs

**Mechanism:** pass the existing pipeline builder bundle as one value. Add named
private records for managed-generation and failed-key inputs.

**Claim:** long and ambiguous interfaces can close without changing public APIs
or runtime data flow.

**Artifact:** private types, unchanged public signatures, and caller checks.

**State:** selected.

### Family: capability-aligned decomposition

**Mechanism:** split cache-only builder construction from bundle assembly. Keep
service construction, evidence collection, and returned capabilities explicit.

**Claim:** the assertion-density finding can close through coherent narrow
functions instead of assertions over untrusted configuration.

**Artifact:** focused source diff and cache-only negative-path tests.

**State:** selected.

### Family: assertion injection or policy suppression

**Mechanism:** add assertions without established invariants, lint allowances,
budgets, baselines, or reduced target scope.

**Claim:** the check can report success without structural repair.

**Blocker:** this can add abort paths or hide resource and interface debt.

**State:** falsified and prohibited.

## Decisions

### Decision: preserve root and build effect order

**Choice:** keep worker completion before managed-generation registration. Keep
managed registration before retained-output registration. Keep batch order and
deduplication order unchanged.

**Rationale:** lint cleanup does not authorize lifecycle or root-authority
changes.

### Decision: reject excess eager messages at the existing completeness boundary

**Choice:** reserve from the session root count and reject a message that would
exceed it. The existing final equality checks remain.

**Rationale:** an extra message already makes evaluation invalid. Earlier typed
rejection adds a real resource bound without changing successful inputs.

### Decision: keep public compatibility

**Choice:** use named records only for private helpers. Keep
`build_registered_derivations`, `parse_drv_key`, and result schemas unchanged.

**Rationale:** internal interface cleanup must not create downstream migration.

### Decision: validate in ascending cost

**Choice:** run package tests and focused Tiger checks after each coherent edit.
Then run strict Clippy, callers, repository Tiger, and full Nix checks.

**Rationale:** small checks isolate semantic regressions and hidden findings.

## Risks / Trade-offs

- Capacity arithmetic must fail through `Error`, not panic or wrap.
- Decomposition can accidentally move builder configuration after execution.
- Registration assertions may state only plan facts established before I/O.
- The repository check can expose a later independent blocker.
