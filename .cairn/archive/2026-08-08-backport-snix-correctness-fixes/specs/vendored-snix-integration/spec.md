## ADDED Requirements

### Requirement: Selected Snix backports preserve Mantle compatibility boundaries

r[vendored_snix.selective_backport_policy] Mantle MUST select Snix updates by reviewed behavior and MUST preserve Mantle-owned identity, store-prefix, transport, substitution, scheduler, and evidence semantics. A selected backport MUST record its upstream source, review status, local adaptation, tests, and disposition.

#### Scenario: Selected fix has local evidence

GIVEN an upstream Snix change repairs behavior that exists in Mantle’s vendor tree
WHEN Mantle adopts or adapts the repair
THEN the lifecycle evidence MUST identify the upstream change, affected local files, retained Mantle differences, and positive and negative tests
AND it MUST distinguish an open upstream review from a merged upstream change.

#### Scenario: Broad sync cannot replace local semantics silently

GIVEN an upstream refactor changes serialization, store identity, transport, scheduler, or public service interfaces
WHEN the refactor conflicts with a documented Mantle boundary
THEN Mantle MUST defer or reject it unless a separate reviewed change replaces that boundary
AND this change MUST NOT claim complete parity with an inferred upstream revision.

#### Scenario: Conditional fix has a concrete reopen trigger

GIVEN an upstream fix affects a backend or configuration that Mantle does not activate
WHEN the review defers that fix
THEN the disposition MUST name the inactive condition and the trigger for another review
AND deferred status MUST NOT be reported as implemented support.

### Requirement: Castore size and FUSE metadata are exact

r[vendored_snix.castore_metadata] Mantle MUST compute castore directory size without duplicate node contributions and MUST expose FUSE directory-entry and inode metadata with the constants required by each kernel interface.

#### Scenario: Directory size counts each entry contribution once

GIVEN a bounded castore directory contains directory, regular-file, or symlink nodes
WHEN Mantle computes the directory size
THEN each encoded entry and node contribution MUST be counted exactly once
AND a node contribution MUST NOT be added again after its entry size already includes it.

#### Scenario: FUSE readdir emits directory-entry types

GIVEN FUSE lists a castore directory containing a directory, regular file, and symlink
WHEN Mantle emits each `readdir` entry
THEN it MUST use the matching `DT_*` directory-entry value
AND it MUST NOT use an inode mode `S_IF*` value as the entry type.

#### Scenario: FUSE attributes have a valid link count

GIVEN FUSE returns attributes for a supported castore node
WHEN Mantle constructs the attribute record
THEN `nlink` MUST contain the adopted valid nonzero value for that node behavior
AND the record MUST NOT expose a zero link count for a present node.

### Requirement: Blocking store services preserve local mutation authority

r[vendored_snix.store_service_behavior] Mantle MUST keep blocking redb transaction work off the async executor and MUST scope cache listing to the writable near PathInfo service.

#### Scenario: Redb write transaction starts in the blocking worker

GIVEN an async directory-service caller requests a redb write
WHEN Mantle schedules the blocking database operation
THEN transaction creation, mutation, and commit MUST occur inside the blocking worker with owned database state
AND no live redb write transaction MUST cross the async scheduling boundary.

#### Scenario: PathInfo cache lists writable local state

GIVEN a PathInfo cache has a writable near service and a read-through far service
WHEN a caller lists the cache
THEN Mantle MUST list the near service only
AND it MUST NOT invoke the far service list operation or expose far-only records as local mutation candidates.

#### Scenario: Near writes become listable

GIVEN a caller writes PathInfo through the cache
WHEN the caller lists the cache after the write succeeds
THEN the near-service entry MUST appear according to the near service’s bounded listing semantics
AND the result MUST remain suitable for local store operations such as explicit signing.

### Requirement: Selected operational backports keep bounded behavior

r[vendored_snix.operational_alignment] Mantle MUST adopt selected ingestion and tracing fixes without importing unrelated Snix API refactors. The implementation MUST keep ingestion memory bounded and MUST apply configured tracing filters to the combined subscriber behavior.

#### Scenario: Filesystem ingestion uses the maintained bounded copy path

GIVEN Mantle ingests a regular file through the vendored castore filesystem adapter
WHEN bytes move into the blob writer
THEN the adapter MUST use the maintained bounded buffering and copy path selected by the adopted upstream fix
AND it MUST NOT retain a fixed oversized buffer without current evidence that the size is required.

#### Scenario: Tracing filter governs combined layers

GIVEN Mantle configures formatting and non-format tracing layers under one environment filter
WHEN an event is enabled or disabled by that filter
THEN the combined subscriber behavior MUST apply the decision consistently
AND attaching the filter to formatting alone MUST NOT suppress or bypass required non-format layer behavior.
