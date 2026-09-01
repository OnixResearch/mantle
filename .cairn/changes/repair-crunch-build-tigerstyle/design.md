## Context

The focused command
`nix run .#tigerstyle -- check -- -p crunch-build -p crunch-rustc-wrapper`
reports 20 findings across six source files. The accepted store repair moved the
repository gate to this boundary without suppressing it.

`crunch-build` owns build-request normalization, content-addressed planning,
execution-profile admission, registry state, and worker orchestration.
`crunch-rustc-wrapper` owns bounded compiler-output publication. A lint repair
can change authority if it replaces typed rejection with panic, changes
canonical bytes, weakens bounds, reorders scheduler effects, or changes
no-replace publication.

## Portfolio Search Contract

- **Goal:** make focused and repository Tiger Style checks report zero findings
  for `crunch-build` and `crunch-rustc-wrapper` with accepted behavior intact.
- **Completion evidence:** zero focused findings, a repository Tiger result that
  advances beyond this build boundary, passing positive and negative package
  tests, strict Clippy, formatting, and full-check evidence.
- **False completion:** lint allowances, reduced package or target scope,
  warning budgets, finding baselines, deleted negative coverage, hidden panic
  paths, changed canonical identity, or a check that stops before this layer.
- **Audit risks:** canonical JSON drift, content-addressed output drift,
  assertion-triggered aborts on untrusted data, scheduler effect reordering,
  registry API breakage, and weakened no-replace publication.
- **Budget:** current repository and pinned Tiger input only; six source files;
  no new dependency or runtime authority; at most ten focused Tiger rounds.
- **Allowed outcomes:** validated, exact later blocker, exhausted round budget,
  or a user decision for an incompatible public contract.

## Approach Registry

### Family: invariant-preserving local repair

**Mechanism:** use checked constants, predicate names, decomposed branches, and
assertions over facts already established by validation or successful effects.

**Claim:** quantity, arithmetic, condition, and assertion findings can close
without changing data flow or rejection behavior.

**Artifact:** narrow edits plus unchanged positive and negative test results.

**State:** active.

### Family: typed interface and error propagation

**Mechanism:** collect related private inputs into named records and propagate
existing typed errors instead of `panic`, `expect`, or untyped fallbacks. Keep
public compatibility wrappers when callers depend on the current shape.

**Claim:** explicit-interface and panic findings can close while making authority
and failure data more visible.

**Artifact:** named request types, compatible wrappers, and negative tests for
invalid values.

**State:** active.

### Family: bounded iterative canonicalization

**Mechanism:** replace recursive JSON traversal with an explicit bounded stack
that rebuilds arrays and sorted objects without changing scalar or object-key
semantics.

**Claim:** recursion can close without changing canonical structured-attribute
bytes or permitting unbounded growth.

**Artifact:** positive canonical-byte fixtures plus negative depth and size
fixtures.

**State:** active if current serialization does not already provide the exact
required canonical order.

### Family: policy suppression

**Mechanism:** add `allow` attributes, budgets, baselines, or narrower check
scope.

**Claim:** the check can report success without source repair.

**Blocker:** this violates the task, Octet policy, and the accepted strict-gate
requirement.

**State:** falsified and prohibited.

## Decisions

### Decision: preserve build identity and effect order

**Choice:** keep derivation hashing, structured-attribute canonical bytes,
content-addressed output selection, registry mutation, waiter notification, and
publication order unchanged.

**Rationale:** lint cleanup does not authorize a build-protocol change.

### Decision: assertions state internal facts only

**Choice:** assert facts established by constructors, successful validation,
checked bounds, or completed filesystem effects. Return typed errors for
malformed profiles, output names, paths, and structured attributes.

**Rationale:** untrusted input must not gain a process-abort path.

### Decision: preserve public compatibility

**Choice:** use named records for private helpers. If a public method exceeds the
parameter limit, add a named public request type and retain a compatibility
wrapper when practical. Update all repository callers and tests together.

**Rationale:** explicit interfaces must not silently alter build meaning.

### Decision: validate in ascending cost

**Choice:** run leaf tests and formatting after each file family, then focused
Tiger and strict Clippy. Run repository and full Nix checks after focused
findings reach zero.

**Rationale:** small feedback loops isolate semantic regressions.

## Risks / Trade-offs

- Iterative canonicalization is more code than recursion and needs exact byte
  parity tests.
- Removing panic paths can require a fallible boundary to move outward through
  callers.
- Named request types can change source compatibility if no wrapper remains.
- The full check can expose a later independent blocker after this layer passes.
