# Specification: Lock-driven vendor fetches

## ADDED Requirements

### Requirement: Bounded producer derivation

r[mantle.lock_vendor_fetch.producer_derivation] Mantle MUST acquire locked
dependencies through a producer derivation that reads the lock file from the
already-fetched source and emits one fixed-output fetch derivation per
artifact plus one assembling derivation laying out the vendor layout.

The producer MUST be bounded in input size, artifact count, and output size.
It MUST run inside the normal build path with no evaluation-time lock
reading, no network of its own, and no nested build tool invocation. Emitted
derivations MUST go through the accepted dynamic-derivation admission with
complete parent identity.

#### Scenario: Lock produces fetches

- GIVEN a fetched source with a Cargo lock naming N artifacts with hashes
- WHEN the producer runs
- THEN it MUST emit exactly N fixed-output fetch derivations with the lock's
  hashes and one assembling derivation, all admitted with parent identity.

#### Scenario: Oversized input

- GIVEN a lock file beyond the declared size or artifact-count bound
- WHEN the producer runs
- THEN it MUST fail closed with the bound named and emit nothing.

### Requirement: Upstream lock hashes are reused

r[mantle.lock_vendor_fetch.lock_hash_reuse] Artifact hashes MUST come from
the upstream lock file. The producer MUST NOT invent, guess, or recompute
hashes, and MUST NOT copy the lock file into the repository.

Dependencies whose lock entries carry no usable hash MUST be rejected with
the dependency named, unless the ecosystem routes them through a shared lock
table.

#### Scenario: Tampered lock entry

- GIVEN a lock entry whose recorded hash does not match the fetched artifact
- WHEN the fetch derivation realizes
- THEN the fixed-output verification MUST fail and the build MUST stop with
  the dependency named.

#### Scenario: Hash-less dependency without a table

- GIVEN an ecosystem with no shared lock table and a dependency without a
  usable hash
- WHEN the producer runs
- THEN it MUST reject the dependency by name and MUST NOT emit an unverified
  fetch.

### Requirement: Shared lock tables

r[mantle.lock_vendor_fetch.shared_lock_tables] Ecosystems whose locks lack
usable hashes MUST use one shared, sorted, version-pinned table mapping
dependency identities to content hashes, with union-merge semantics for
parallel edits.

The table MUST be the only additional repository state. A package's producer
MUST read only its own subset; adding entries for one package MUST NOT
invalidate another package's assembled output.

#### Scenario: Parallel additions merge

- GIVEN two branches each adding disjoint sorted entries to one table
- WHEN the branches merge textually
- THEN the result MUST equal the sorted union with no manual resolution.

#### Scenario: Unrelated package unaffected

- GIVEN an assembled vendor output cached for package A and a new table entry
  used only by package B
- WHEN package A rebuilds
- THEN its producer's input identity MUST be unchanged and its output MUST
  not rebuild.

### Requirement: Bundles exclude vendored trees

r[mantle.lock_vendor_fetch.bundle_payload_reduction] Source-bundle profile
modes MUST be able to exclude vendored dependency trees and record lock-driven
fetch as the acquisition path.

The profile receipt MUST record the excluded payload and the producer
identity. Hydration and fixed-point modes that exclude vendored trees MUST
materialize them through the producer with per-artifact verification before
self-build preflight.

#### Scenario: Fresh clone without vendored trees

- GIVEN a fresh-clone profile excluding vendored trees
- WHEN hydration materializes dependencies
- THEN the vendor layout MUST match the previous vendored tree's content
  hashes and the receipt MUST record the producer identity.

#### Scenario: Excluded but never materialized

- GIVEN a self-build preflight over a profile that excludes vendored trees
  without materialization
- WHEN preflight runs
- THEN it MUST fail closed naming the missing materialization step.

### Requirement: Fail-closed denials

r[mantle.lock_vendor_fetch.negative_controls] Every malformed or hostile
input MUST produce a typed denial: wrong-hash artifact, tampered or
contradictory lock entries, missing artifact after fetch, unpacked-tree hash
mismatch, path escape in layout entries, and duplicate artifact identities.

Denials MUST name the offending artifact and reason. Partial materialization
MUST NOT be published.

#### Scenario: Path escape in layout

- GIVEN an assembling layout entry that escapes the vendor root
- WHEN the assembling derivation runs
- THEN it MUST reject the entry and fail closed.

#### Scenario: Duplicate identity

- GIVEN two artifacts claiming the same identity with different hashes
- WHEN the producer evaluates the lock
- THEN it MUST fail closed naming both entries.
