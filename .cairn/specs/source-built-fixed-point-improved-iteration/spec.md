# Source Built Fixed Point Improved Iteration Specification

## Purpose

Defines the `source-built-fixed-point-improved-iteration` capability.

## Requirements

### Requirement: Dev-only provider-output cache adopts only receipt-validated entries

r[source_built_fixed_point_improved_iteration.dev_provider_cache] The source-built fixed-point command MUST provide an opt-in dev-only provider-output cache. Membership MUST be keyed by the source-authority digest plus the closure, hermeticity, protected-execution, effect, and normalization policy digests from the current plan. On lookup the shell MUST recompute the source-authority digest from the current materialized sources and MUST adopt a cached StageX provider or full-source native provider only when its stored receipt re-validates against those exact digests. When the cache flag is unset, or on a missing or stale receipt, or on any digest mismatch, the attempt MUST take the normal cold construction path and MUST NOT use the cached provider.

#### Scenario: Matching receipt-validated provider is adopted

GIVEN a dev run passes an explicit provider-cache directory AND that directory contains a StageX provider and full-source native provider whose stored receipts re-validate against the current source-authority and policy digests
WHEN the shell runs the dev attempt
THEN it MUST adopt the cached providers by reference instead of reconstructing them
AND the adoption MUST be recorded as a dev-cache hit in the attempt transcript.

#### Scenario: Stale or mismatched receipt is a hard miss

GIVEN a dev run passes an explicit provider-cache directory AND a stored provider's receipt does not re-validate against the current source-authority or policy digests
WHEN the shell runs the dev attempt
THEN it MUST treat that provider as a cache miss
AND it MUST take the normal cold construction path.

#### Scenario: No cache flag keeps the cold path unchanged

GIVEN a run does not pass the dev provider-cache flag
WHEN the shell runs the attempt
THEN it MUST construct the StageX provider and full-source native provider from scratch
AND it MUST never read or adopt any cached provider output.

### Requirement: Content-addressed store snapshot seeds a dev run

r[source_built_fixed_point_improved_iteration.dev_store_snapshot] After a completed dev run the shell MUST snapshot the native store and state into the cache keyed by plan digest, and on the next dev run for the same plan digest it MUST seed the fresh staging store from that snapshot so unchanged content-addressed store paths are reused rather than rebuilt. The snapshot MUST only be used on dev runs and MUST be bounded by the existing disk-bytes proof preflight.

#### Scenario: Same plan digest seeds from snapshot

GIVEN a previous dev run completed and wrote a store snapshot keyed by its plan digest AND a later dev run uses the same plan digest
WHEN the later run seeds its staging store
THEN it MUST reuse unchanged content-addressed store paths from the snapshot
AND it MUST NOT re-import unchanged source records from scratch.

#### Scenario: Disabled seeding preserves cold behavior

GIVEN a run does not enable dev store-snapshot seeding
WHEN the shell prepares the staging store
THEN it MUST build the store from empty authority as today
AND it MUST NOT read any prior store snapshot.

### Requirement: Fast-fail baseline reports an unchanged source profile

r[source_built_fixed_point_improved_iteration.dev_fast_fail_baseline] Before launching a full dev run the shell MUST hash the current source profile and compare it to the last published fixed-point receipt bindings. When the source profile is unchanged from a prior successful fixed point, the shell MUST report that prior success with a dev notice rather than launching a full rebuild. The fast-fail path MUST report the exact prior digest it matched and MUST NOT fabricate a fresh proof. The promoted cold run MUST NOT use the fast-fail path.

#### Scenario: Unchanged source profile short-circuits

GIVEN the current source profile digest equals the last published successful fixed-point receipt binding AND a dev run is requested
WHEN the shell runs the fast-fail check
THEN it MUST report the prior success and its exact source/digest binding
AND it MUST NOT launch a full rebuild.

#### Scenario: Changed source profile proceeds

GIVEN the current source profile digest differs from the last published fixed-point receipt binding
WHEN the shell runs the fast-fail check
THEN it MUST proceed with a full dev run
AND it MUST NOT report the prior success as current.
