# Mantlepkgs Version Resolution Specification

## Purpose

Define bounded resolution from reported package versions to exact Nixpkgs revisions before Mantlepkgs production.

## ADDED Requirements

### Requirement: Revision cohorts and indexes are typed and system-specific

r[mantlepkgs_versions.typed_index] Mantle MUST define typed Nickel contracts for revision cohorts, observations, compact indexes, version requests, selection policy, and named limits.

Each index MUST bind one target system, exact cohort identity, generator identity, observation-set identity, and deterministic entry set.

#### Scenario: Complete per-system index is accepted

GIVEN an index binds one system, exact sampled revisions, declared attributes, observation methods, statuses, and bounded limits
WHEN Mantle validates and normalizes the index
THEN it MUST produce one deterministic BLAKE3 index identity
AND record order, file order, host paths, and ambient state MUST NOT change that identity.

#### Scenario: Index system or cohort is invalid

GIVEN an index has a floating cohort, wrong system, duplicate key, unknown method, missing observation identity, unsafe path, or exceeded limit
WHEN Mantle validates the index
THEN it MUST reject the index with ordered diagnostics
AND it MUST NOT run Nix or produce a resolution receipt.

### Requirement: Version observations preserve unavailable and failed states

r[mantlepkgs_versions.observation_status] The producer MUST record each revision and attribute observation as success, unavailable, or failed with bounded facts and stable reasons.

The initial successful method MUST read the package `version` attribute. It MUST NOT infer a version from the derivation name.

#### Scenario: Package version is observed

GIVEN one sampled revision exposes the declared attribute and a valid `version` value for the target system
WHEN the producer records the observation
THEN it MUST bind the revision, Nix `narHash`, attribute, system, method, reported version, and response identity
AND saved observation replay MUST NOT require network or Nix access.

#### Scenario: Version cannot be observed

GIVEN evaluation fails, the attribute is absent, `version` is absent, the value is malformed, or a bound is exceeded
WHEN the producer records the observation
THEN it MUST retain unavailable or failed status with a stable reason
AND it MUST NOT create a successful entry from package-name parsing or another revision.

### Requirement: Revision selection is deterministic and policy-bound

r[mantlepkgs_versions.deterministic_resolution] Mantle MUST resolve requests through a pure deterministic core over validated policy and saved per-system indexes.

The initial policy MUST select the newest sampled published revision that reports the requested version.

#### Scenario: Several revisions report the requested version

GIVEN several accepted observations report the requested attribute and version for one system
WHEN the core applies `newest-published-revision-for-reported-version`
THEN it MUST select the newest cohort-ordered revision
AND observation input order MUST NOT affect the result.

#### Scenario: Requested version has no accepted observation

GIVEN no successful index entry matches the requested system, attribute, and reported version
WHEN the core resolves the request
THEN it MUST return a typed unresolved result with ordered blockers
AND it MUST NOT select a nearby version, another system, or current network state.

### Requirement: Resolution receipts bind exact source and version facts

r[mantlepkgs_versions.resolution_receipt] Mantle MUST emit a versioned resolution receipt for each resolved or blocked request.

A successful receipt MUST bind request, policy, index, observation-set, system, attribute, reported version, selector, revision, Nix `narHash`, and observation method.

The receipt identity MUST use BLAKE3. The Nix `narHash` MUST retain its required SHA-256 algorithm tag.

#### Scenario: Successful resolution receipt is complete

GIVEN one request resolves under an accepted index and policy
WHEN Mantle emits the receipt
THEN the receipt MUST contain every exact selection and source fact
AND it MUST distinguish BLAKE3 receipt identity from Nix SHA-256 source identity.

#### Scenario: Receipt fact is stale or mismatched

GIVEN the request, policy, index, system, revision, `narHash`, method, or reported version differs from the receipt
WHEN Mantle validates the receipt
THEN validation MUST fail with a deterministic mismatch code
AND no Mantlepkgs generation MUST use that receipt.

### Requirement: Selected revisions are rechecked before catalog production

r[mantlepkgs_versions.producer_recheck] The producer MUST re-evaluate each selected attribute at its exact revision before it emits a Mantlepkgs generation manifest.

It MUST compare the observed version and Nix `narHash` with the accepted receipt. It MUST compute the source-tree BLAKE3 for the existing source lock.

#### Scenario: Selected revision recheck passes

GIVEN the exact source materializes and reports the requested version for the target system
WHEN the producer performs the recheck
THEN it MUST bind the observed version, source-tree BLAKE3, exact revision, and `narHash`
AND it MAY emit an existing Mantlepkgs manifest for that revision group.

#### Scenario: Selected revision recheck fails

GIVEN source materialization fails or any revision, hash, attribute, system, or version fact differs
WHEN the producer performs the recheck
THEN it MUST block that request with an ordered reason
AND it MUST NOT emit a success manifest or silently choose another revision.

### Requirement: Requests are grouped by exact revision for domain composition

r[mantlepkgs_versions.revision_grouping] Mantle MUST group accepted requests by exact selected revision before existing Mantlepkgs generation.

Each group MUST use one exact source lock. Resulting generations MUST compose through typed domain shards with unambiguous versioned public selectors.

#### Scenario: Requests share one revision

GIVEN several accepted requests resolve to one exact revision
WHEN Mantle creates the production plan
THEN it MUST create one source group containing all compatible selectors
AND it MUST NOT repeat source evaluation for each request without an explicit reason.

#### Scenario: Two versions require different revisions

GIVEN two requests for one attribute resolve to different exact revisions
WHEN Mantle creates and composes their generations
THEN it MUST place them in separate source groups and domain shards
AND their versioned public selectors MUST remain distinct.

### Requirement: Resolution logic uses a functional core

r[mantlepkgs_versions.functional_core] Index normalization, observation compaction, request resolution, revision grouping, diagnostics, and identity construction MUST be pure deterministic functions.

#### Scenario: Equivalent saved inputs replay

GIVEN equivalent cohorts, observations, indexes, requests, and policies arrive in different orders
WHEN the core resolves and groups them
THEN plans, receipts, blockers, and BLAKE3 identities MUST match
AND the core MUST NOT read files, run processes, access networks, inspect clocks, or mutate state.

#### Scenario: Input exceeds a named limit

GIVEN revision, attribute, observation, request, group, diagnostic, or artifact input exceeds policy
WHEN the core validates the input
THEN it MUST return a typed limit failure before unbounded work
AND the shell MUST NOT continue with truncated success evidence.

### Requirement: Version-resolution claims stay bounded

r[mantlepkgs_versions.claim_boundary] Mantle MUST limit version-resolution claims to exact cohorts, systems, observations, policies, revisions, hashes, rechecks, generations, and receipts.

#### Scenario: Versioned catalog pilot passes

GIVEN two reported versions resolve, recheck, generate, and compose for one exact pilot cohort
WHEN Mantle writes the evidence summary
THEN it MAY report successful resolution and catalog production for that cohort
AND it MUST NOT claim package correctness, compatibility, cache retention, evaluator parity, reproducibility, deployment safety, or release eligibility.
