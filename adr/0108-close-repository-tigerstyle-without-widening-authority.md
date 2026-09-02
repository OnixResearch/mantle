# ADR 0108: Close repository Tiger Style without widening authority

- **Status:** Accepted
- **Date:** 2026-09-01

## Context

After package-level repairs, the complete Tiger Style gate reaches 36 findings
in the Mantle root library. The affected modules validate operator contracts,
classify remediation, fetch bootstrap seeds, authorize protected execution,
supervise kernel execution, and render stable errors.

A repository-wide cleanup must not change accepted operator data, first-match
remediation, bootstrap output identity, executable authority, kernel response
handling, audit identity, or public interfaces. Lint allowances and assertions
over untrusted data are not structural repairs.

## Decision

Keep wire compatibility through named explicit serde default functions that
produce the same empty values as the previous implicit defaults. Use named
private inputs for same-type parameter groups. Assert only bounds, uniqueness,
and postconditions established by successful validation.

Split remediation classification into ordered policy-family helpers. Evaluate
those helpers in the existing source order and return the first match.

Use checked conversions and fixed-width public counts at protected-execution
boundaries. The public audit-quiescence result changes from `usize` to `u32`;
all repository callers use checked conversion where they need an index width.
Decompose admission predicates without widening them. Treat kernel response,
tracee-memory, and descendant-reaping failures as explicit fail-closed results.
Do not discard a failed deny response.

Split bootstrap fetching at the existing service-construction, request,
ingest, and export phases. Replace production serialization panics with explicit
typed or deterministic bounded fallback handling.

Repeat the complete pinned repository Tiger Style command after each repair
family. Completion requires that exact command to exit successfully with zero
findings across all first-party targets.

## Consequences

- Legacy omitted operator fields retain their previous empty values.
- Remediation priority and output fields remain stable.
- Protected execution keeps exact authority and observable kernel failures.
- Public counters are portable and reject overflow instead of truncating.
- Downstream source that names the former `usize` result must migrate to `u32`.
- Bootstrap and error-rendering success bytes remain stable.
- A green package check alone cannot complete this repair.
- This decision does not prove compiler correctness, sandbox correctness,
  external service availability, or release eligibility.
