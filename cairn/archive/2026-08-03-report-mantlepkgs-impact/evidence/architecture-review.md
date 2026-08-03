# Architecture review: Mantlepkgs impact reports

## Question

Which EkaCI comparison ideas fit Mantle without importing hosted CI authority or weakening action-result admission?

## Inspected evidence

- Mantle ADR 0024 separates action results from CAS and execution.
- Mantle ADR 0056 defines generated package catalogs and explicit blockers.
- Mantle build reports and PathInfo state already expose bounded outcomes and closure facts.
- EkaCI documents base and head derivation comparison, result classifications, closure differences, and dedicated CI presentation.
- The inspected EkaCI architecture document references at least one source path absent from the current repository tree.
- The EkaCI repository reported an AGPL-3.0 license during this planning review.

External sources:

- <https://github.com/ekala-project/eka-ci>
- <https://github.com/ekala-project/eka-ci/blob/main/docs/architecture.md>

## Decision

Adapt the base-to-head report model. Keep Mantle comparison pure and evidence-bound. Keep forge credentials, webhooks, comments, approvals, and dashboards in an external adapter.

Do not adopt EkaCI service code or derivation-path-only memoization.

## Owner

Mantle owns snapshot compatibility, package dispositions, admitted outcome transitions, closure deltas, report identity, and non-claims. External CI adapters own forge interaction.

## Next action

Implement the report contracts and pure snapshot comparison after the versioned Mantlepkgs catalog record is stable.

## Non-claim

This review is planning evidence. It does not validate EkaCI runtime behavior, package results, closure completeness, or forge integration.
