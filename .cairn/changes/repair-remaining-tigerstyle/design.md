## Context

The command `nix build .#checks.x86_64-linux.tigerstyle --no-link -L
--builders ''` reports 36 root-library findings after the accepted pipeline
repair. The current classes are assertion density, explicit defaults, boolean
names, ambiguous parameters, production panics, function length, sentinel
fallbacks, compound conditions, unchecked arithmetic, portable APIs, and a
discarded result.

The affected modules own operator-contract validation and remediation,
bootstrap acquisition, protected executable authority, seccomp and ptrace
supervision, audit delivery, and stable error envelopes. Structural cleanup
must not change those authority or compatibility boundaries.

## Portfolio Search Contract

- **Goal:** make the complete repository Tiger Style command report zero
  findings while preserving current successful behavior and fail-closed paths.
- **Completion evidence:** a zero-exit repository Tiger Style result, passing
  pre-change and post-change root tests, strict Clippy, formatting, caller
  checks, and full-check evidence.
- **False completion:** lint allowances, warning budgets, finding baselines,
  narrower targets, deleted negative tests, changed wire defaults, reordered
  remediation, widened executable authority, swallowed supervision errors, or
  replaced fail-closed behavior.
- **Audit risks:** legacy operator JSON rejection, remediation priority drift,
  changed protected root admission, response errors lost from audit evidence,
  public count truncation, unchecked public count migration, bootstrap output
  drift, or new panic paths.
- **Budget:** current repository and pinned Tiger input only; at most fifteen
  complete Tiger rounds; no new dependency or runtime authority.
- **Allowed outcomes:** zero repository findings, an exact tool or infrastructure
  blocker, exhausted round budget, or a user decision for incompatible behavior.

## Approach Registry

### Family: explicit compatibility and bounded contracts

**Mechanism:** use named serde default functions, named validation inputs, and
post-validation assertions over established bounds and uniqueness.

**Claim:** contract findings can close without changing accepted wire data,
canonical command names, aliases, or validation errors.

**Artifact:** compatibility fixtures, positive and negative validation tests,
and exact repository Tiger output.

**State:** selected.

### Family: ordered policy decomposition

**Mechanism:** split remediation classification into ordered policy-family
helpers. Return the first matching record in the current order.

**Claim:** function-length and assertion findings can close without changing
classification priority, safe subjects, commands, or documentation links.

**Artifact:** table-driven classifier tests and source-order review.

**State:** selected.

### Family: explicit protected-execution boundaries

**Mechanism:** use checked conversions, fixed-width public counts, named
predicates, decomposed admission guards, and handled response errors.

**Claim:** protected execution remains exact, bounded, and fail-closed while its
invariants become visible.

**Artifact:** serial positive and negative supervisor tests plus audit checks.

**State:** selected.

### Family: phase-aligned bootstrap and serialization

**Mechanism:** split seed fetching at existing service, request, ingest, and
export phases. Replace serialization `expect` calls with explicit error paths or
a deterministic bounded envelope fallback.

**Claim:** bootstrap and error rendering retain their existing successful bytes
and effect order without production panic paths.

**Artifact:** bootstrap tests, error-envelope byte tests, and caller checks.

**State:** selected.

### Family: suppression or assertion injection

**Mechanism:** add lint allowances, reduce checked targets, or add assertions
that do not express established invariants.

**Claim:** the gate can become green without repairing the design.

**Blocker:** this hides debt or creates new abort paths over untrusted input.

**State:** falsified and prohibited.

## Decisions

### Decision: clear the complete repository gate

**Choice:** continue through newly exposed findings until the exact repository
Tiger Style command exits successfully.

**Rationale:** package-local success is insufficient for the requested result.

### Decision: preserve compatibility and authority before style

**Choice:** reject any cleanup that changes accepted operator data, remediation
priority, bootstrap identity, protected executable authority, supervision
failures, audit fields, or public behavior. Normalize only the reported audit
count from `usize` to `u32`, and use checked conversion at existing callers.

**Rationale:** lint cleanup does not authorize semantic migration. The pinned
portable-API rule does require one explicit fixed-width boundary migration.

### Decision: test protected supervision serially

**Choice:** run affected protected-execution tests with one test thread and keep
the known deep-descendant timeout as an independent baseline when applicable.

**Rationale:** installed seccomp listeners are thread-local and irreversible.

### Decision: validate in repeated ascending-cost rounds

**Choice:** run focused tests after each coherent edit. Run the repository Tiger
Style command after each finding family, then run broad validation after zero.

**Rationale:** each round exposes hidden findings without mixing unrelated
regressions.

## Risks / Trade-offs

- Serde default helpers must reproduce the exact previous empty values.
- Classifier extraction can change first-match priority if helper order drifts.
- Fixed-width count conversion must reject overflow rather than truncate.
- Response-send failures must remain observable and fail-closed.
- Assertions must describe facts already established by typed validation.
- The full flake check can retain non-Tiger infrastructure blockers.
