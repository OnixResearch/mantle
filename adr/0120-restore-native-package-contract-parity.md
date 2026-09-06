# ADR 0120: Restore native package contract parity

## Status

Proposed. The change remains active until its selected gates pass.
See `.cairn/changes/repair-native-package-contract-drift/`.

## Context

The complete no-fail-fast package run exposed nine failing integration targets.
Focused baselines separated stale fixtures from store enumeration and diagnostic decoding defects.
A passing assertion cannot justify a changed authority boundary.

## Decision

Mantle owns composed store enumeration. The generic Snix cache retains its near-only listing behavior.
The store shell enumerates the writable layer and declared bases under one aggregate scan bound.
Ordinary composed lookup selects each unique candidate and retains layer trust, precedence, and shadow checks.
The scan counts shadowed rows before deduplication. It does not backfill data or expose new services.
Base generation checks surround the operation. Existing capability types remain unchanged.

The unchanged global guard also finds six gateway authority violations at the published prerequisite.
A new store-owned `GatewayStore` value replaces its broad handle and raw service access.
It reuses existing bounded transfer operations and exposes only object reads, NAR ingest, and observed imported-output persistence.
Imported metadata must match the observed NAR. Persistence cannot register roots or attach invented provenance.
Imports retain existing castore-only, non-root behavior; this repair does not add physical output export.
The initial positive control incorrectly expected an exported link. The corrected control requires exact reopened PathInfo and node identity plus absent export and root registration.
An opaque actor-local token binds the ingest observation to the same open store instance.
Another store or a reopened instance cannot use that observation to publish metadata for absent content.
The gateway retains existing signature checks and repeats current authority admission before final persistence.
Compile-time denials and the unchanged global guard check that this does not become build, GC, source, or raw-service authority.

The worker diagnostic adapter treats semicolons and line endings as field boundaries.
It rejects duplicate fields and reports over 65,536 bytes. Multiline remediation cannot become part of a capture status code.
These decoded diagnostic fields do not grant execution or output-admission authority.
Worker capture policy and original build failure remain unchanged.

Inventories use existing owners and generators. The portable profile generator supplies the missing operator projection.
Machine cohort tests compare exact unique membership instead of a stale count.
Example entries describe bounded probes and supplied-evidence reports, not provider admission.

The module-boundary scan covers all declared roots. Its previous 512-file cap cannot cover the baseline's 236 `src` and 463 `crates` files.
The new budget permits 1,024 files, 2,048 entries, 16 directory levels, 2 MiB per file, and 32 MiB total content.
Actual reads enforce byte limits. The scan rejects symlinks, special files, missing roots, and excess input instead of truncating coverage.
Positive and negative tests cover exact and exceeded limits plus injected module coupling.
One exact `repository` field in the resource-policy fixture generator records the reviewed external source URL, not a module dependency.
The guard binds that data occurrence to its owner file and full line. Repeated references, runtime owners, and additional coupling still fail.

Remote fixture commands inherit the descriptor of their actual owned ticket file.
A child-only flag change replaces the fixed descriptor-slot remap. The parent retains close-on-exec flags and ownership.
Two-file and absent-inheritance controls check the mechanism. Historical intermittent descriptor aborts remain observations, not a proven complete causal diagnosis.

The NAR guard also contained a stale direct-writer fragment after the Rust-cache capability split.
It now checks both caller delegation and the existing store-owned Snix writer. Missing-edge controls reject broken delegation.
Its vendored source commit and package checksum remain unchanged. Sparse, locked Cargo vendoring supplies only this guard's dependency source cohort.
This is not a complete source-bootstrap vendor tree.

## Rejected alternatives

- Changing generic cache enumeration would alter unrelated cache consumers.
- Changing counts or diagnostics without examining owners would hide missing coverage.
- Broad store handles or writable base access would violate the capability boundary.
- Treating the build fixture's runtime error as a skip would fabricate execution success.
- An unbounded source scan or weaker capture policy would trade one failure for an authority or resource defect.

## Consequences

No core receives host I/O or framework state. Store and protocol effects remain shell-owned.
No format migration, dependency update, signing-key change, global promotion, or downstream admission follows.
The full package gate and scoped consumer qualification remain separate requirements.
