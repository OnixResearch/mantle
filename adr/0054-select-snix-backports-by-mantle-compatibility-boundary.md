# ADR 0054: Select Snix backports by Mantle compatibility boundary

## Status

Accepted (2026-08-01)

## Context

Mantle vendors `nix-compat`, `snix-build`, `snix-castore`, `snix-store`, and `snix-tracing`. The vendor tree does not record one upstream Snix revision.

The local crates contain intentional changes for BLAKE3 identities, postcard directory encoding, configurable store prefixes, iRPC, local substitution, scheduler behavior, and Mantle evidence. Some local files already contain selected later upstream changes. A timestamp or dependency version therefore cannot identify a safe synchronization base.

An upstream Gerrit review found small correctness and security fixes that apply to Mantle. The same review found broad refactors and serialization changes that conflict with local boundaries.

## Decision Drivers

- Correct trust and data-integrity defects quickly.
- Preserve Mantle-owned identity, evidence, and compatibility semantics.
- Keep each imported behavior reviewable and testable.
- Avoid a false claim that the vendor tree matches one upstream revision.
- Keep upstream code review evidence separate from Mantle release evidence.

## Decision

Mantle treats its vendored Snix crates as an adapted source fork.

Mantle selects upstream changes by behavior. It does not bulk-sync the vendor tree by date, branch head, or an inferred revision.

Each selected backport must record:

- the upstream Gerrit change and review status;
- the local files and semantics affected;
- the Mantle adaptations that must remain;
- positive and negative tests for the selected behavior;
- a disposition of adopted, adapted, deferred, or rejected;
- the evidence that supports the local result.

Security and correctness fixes can be adapted from an open upstream change when the defect and repair are independently demonstrated in Mantle. The local change must not claim upstream acceptance.

A remote trust-boundary fix must fail before persistence, castore mutation, output export, sidecar creation, root registration, or successful cache reporting.

Mantle will not import upstream protobuf directory identity. Postcard directory encoding remains a deliberate incompatible boundary. Mantle will also preserve BLAKE3, configurable store prefixes, iRPC, substitution, and scheduler adaptations unless a separate ADR replaces them.

The review ledger lives with the Cairn change that implements a selected group of backports. Completed changes preserve that ledger in the archive.

## Alternatives Considered

### Bulk-sync the latest Snix revision

Rejected because no recorded base exists and a broad sync can overwrite Mantle-owned semantics.

### Freeze the vendor tree

Rejected because known trust and correctness defects would remain active.

### Import upstream commits without local adaptation

Rejected because upstream tests do not cover Mantle’s local store prefixes, postcard identities, substitution shell, or evidence side effects.

### Record only a future upstream revision marker

Rejected as a repair for current history. A future marker can improve later reviews, but it cannot identify the source of existing mixed adaptations.

## Consequences

- Vendor updates are smaller and easier to audit.
- Every selected behavior needs local positive and negative tests.
- Mantle can adopt clear fixes before an upstream change merges, but it must label that source as open.
- Full upstream parity remains unproven.
- Maintenance needs an explicit ledger instead of a single revision comparison.
- Future vendor imports can add source revision metadata without changing this compatibility policy.
