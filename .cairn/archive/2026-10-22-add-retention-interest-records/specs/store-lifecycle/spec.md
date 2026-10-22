# Specification: Retention interest records

## ADDED Requirements

### Requirement: Retention is one canonical record per interest

r[mantle.store_lifecycle.retention_interest_records] Mantle MUST represent each
retention of a store path as one canonical record naming the owner, the logical
store path, a reason, and declaration facts. The record MUST be persisted under
the state directory, named by the BLAKE3 identity of its canonical bytes, and
validated before use. The GC planner MUST consume the deterministically merged
record set.

The merged set MUST NOT change GC decision rules, ordering, or candidate
classification. Records MUST be bounded in size and count, and an oversized,
malformed, duplicate, or unknown-version record MUST fail closed.

#### Scenario: Two owners retain one path

- GIVEN two owners each declare retention for one logical store path
- WHEN both records are published and merged
- THEN the merged retained set MUST contain the path
- AND both records MUST remain present with their owner and reason facts

#### Scenario: Merged records feed the planner unchanged

- GIVEN a merged record set equivalent to a legacy root set
- WHEN the GC planner runs on each input
- THEN the plan MUST contain the same candidates and the same ordering

#### Scenario: Automatic lease renewal replaces its previous declaration

- GIVEN two owners retain one path and another owner's interest would be chosen as the merged path representative
- WHEN the lease owner renews with a changed transition reason
- THEN the lease owner's old record MUST be replaced without removing the other owner's record
- AND its renewal count MUST advance monotonically within the configured bound
- AND a renewal rejected at that bound MUST NOT alter persisted records

### Requirement: Retention release is owner scoped

r[mantle.store_lifecycle.retention_owner_scope] A release operation MUST remove
only records owned by the requesting owner. It MUST NOT modify or remove a
record of another owner. Store reporting MUST expose owner, reason, and record
counts per retained path, and legacy migration MUST remain explicit.

#### Scenario: Foreign release is rejected

- GIVEN a retention record owned by owner `a`
- WHEN owner `b` requests release of that path
- THEN the request MUST fail closed or leave the record unchanged
- AND the path MUST remain retained

#### Scenario: Migration preserves legacy provenance

- GIVEN a legacy roots file with retained paths
- WHEN an operator runs the explicit migration command
- THEN each converted path MUST carry owner and reason facts
- AND every previously `legacy-unmanaged` path MUST remain visible
