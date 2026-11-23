## ADDED Requirements

### Requirement: A store handle verifies each castore node's completeness once

r[cache_substitution.completeness_memo] Within one `StoreHandle`, Mantle MUST verify a castore
node's complete tree at most once per finalized node identity, keyed by kind, digest and size for
files and directories. A repeat check of a node already verified complete on that handle MUST NOT
read, hash or probe the active services. The set of verified identities MUST be bounded by a named
constant, for example `MAX_VERIFIED_COMPLETE_NODES`; at that bound the handle MUST stop inserting
and MUST keep performing full checks, and it MUST NOT evict an entry to make room. Only a
successful check MAY be recorded, so an incomplete or absent node MUST be re-checked in full on
every call. Symlinks are inline and need no entry. The set is per handle and in memory; it MUST be
cleared by every handle operation that deletes or replaces castore blobs or directories, including
garbage collection and guarded garbage-collection recovery. The uncached recursive completeness
check MUST remain available and MUST be used by callers that require a fresh check, in particular
action-result admission.

Boundary: the memo covers this handle's lifetime only. A separate process deleting castore content
while the handle is live was already outside the completeness check's guarantee, which observes the
services at the moment of the check and holds no lease.

#### Scenario: A repeat check of a complete node skips the services

- GIVEN a `StoreHandle` that verified a complete file or directory node and recorded its finalized
  identity
- WHEN a later lookup or cache check on the same handle verifies that same node identity
- THEN it MUST return the same result without reading, hashing or probing any blob or directory
- AND the returned `PathInfo` MUST be unchanged.

#### Scenario: An incomplete node is re-checked on every call

- GIVEN a node whose blob or child content is missing or does not match its declared size
- WHEN the handle verifies it repeatedly
- THEN every call MUST perform the full recursive check
- AND no identity MUST be recorded.

#### Scenario: A node that was never verified is checked in full

- GIVEN a handle with one verified complete node identity
- WHEN it verifies a different node identity
- THEN it MUST perform the full recursive check against the services.

#### Scenario: Deletion clears the memo

- GIVEN a handle that recorded a verified complete node identity
- WHEN a handle operation that deletes or replaces castore content runs, including garbage
  collection or guarded garbage-collection recovery
- THEN the set MUST be empty afterwards
- AND the next check of that node MUST read the services again.

#### Scenario: The bound retains full checking

- GIVEN a handle whose verified-node set has reached its named bound
- WHEN it verifies another complete node identity
- THEN it MUST perform the full check and return the honest result
- AND it MUST NOT evict an existing identity to record the new one.
