# Design: Mantlepkgs source-update plans

## Context

Ekala `corepkgs` exposes reusable updater combinators. `ekapkgs-update` adds source adapters, version policy, OSV observations, Repology observations, and review output.

The inspected updater code invokes Nix repeatedly for package metadata. Its selected vulnerability path converts fetch failures into empty results.

Mantle needs the useful policy model without those authority and evidence gaps.

## Decisions

### Decision: author update policy in typed Nickel

**Choice:** Each package update policy binds package and variant selectors, source kind, current version, current source identity, version normalization, allowed and ignored constraints, prerelease rules, patch policy, advisory policy, validation policy, and named limits.

Policy has an explicit version and BLAKE3 identity. It does not contain executable shell fragments.

**Rationale:** Reviewers can inspect policy without granting arbitrary process authority.

### Decision: observe sources through bounded adapters

**Choice:** Shell adapters support declared source kinds, such as Git tags, release APIs, directory indexes, and ecosystem registries.

Each observation binds adapter identity, query, source authority, response identity, status, candidates, and collection facts. Status is explicit success, unavailable, or failed.

Adapters use bounded response bytes, candidate counts, redirects, retries, and elapsed time. Ambient credentials and proxies remain disabled unless explicit policy allows them.

**Rationale:** Network acquisition is evidence, not a silent input to pure selection.

### Decision: select candidates in a pure core

**Choice:** The core normalizes versions, applies constraints, rejects ignored or prerelease candidates, orders accepted candidates, and returns one candidate or typed reason codes.

Version rules do not contain unexplained numeric thresholds. Odd-minor or high-patch development rules use named policy fields.

**Rationale:** Candidate selection must replay from saved observations without network access.

### Decision: make mutation dry-run and preimage-bound

**Choice:** The core produces an update effect plan. Each effect binds target path, structured field, exact old value, exact new value, input digest, output digest, and reason.

The shell defaults to no mutation. Explicit execution re-reads every input, verifies the preimage, writes a private stage, validates generated outputs, and publishes with no-clobber semantics.

Plain regular-expression replacement across arbitrary source is not accepted.

**Rationale:** Source updates must not rewrite an unintended field or stale checkout.

### Decision: preserve advisory availability

**Choice:** OSV and Repology observations are separate evidence records. Each record binds service identity, query, package coordinate, version, response digest, schema, status, and bounded result set.

Unavailable or failed queries remain unavailable or failed. They never become an empty finding set.

Advisory policy can block automatic proposal or require review. It cannot prove source trust, package correctness, or release eligibility.

**Rationale:** Missing security data is not evidence of no known issue.

### Decision: validate candidates through existing Mantle boundaries

**Choice:** A candidate update can produce a new catalog generation, package build observations, separate validation-root observations, and a package-impact report.

The update receipt links those artifacts and records missing or failed validation explicitly. It does not bypass ordinary build, cache, source, or release admission.

**Rationale:** Update automation must compose existing evidence instead of inventing a new trust path.

### Decision: preserve a functional core and imperative shell

**Choice:** The core validates policy, normalizes observations, selects candidates, plans effects, classifies advisory status, derives proposal status, and computes BLAKE3 identities.

The shell reads files, evaluates Nickel, performs bounded network requests, invokes producers, realizes validation roots, stages mutations, and writes receipts.

**Rationale:** Update policy remains testable without files, networks, processes, clocks, or stores.

## Rollout

1. Add typed policy and saved-observation fixtures.
2. Add pure candidate selection and effect planning.
3. Add one Git source adapter and one advisory adapter in observe-only mode.
4. Add preimage-bound mutation execution.
5. Link builds, validation roots, and impact reports before automatic proposal status.
6. Add more ecosystem adapters only after positive and negative evidence exists.

## Risks and trade-offs

- Source APIs can change or rate-limit requests.
- Version strings can be ambiguous across ecosystems.
- Structured Nickel mutation needs a stable field-update boundary.
- Advisory services can be unavailable or incomplete.
- Automatic proposals can create review load even when every update is valid.
