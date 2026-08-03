# Mantlepkgs Catalog Structure Specification

## Purpose

Define bounded package domains, deterministic catalog composition, explicit variants, separate validation roots, and external corpus evidence for Mantlepkgs.

## ADDED Requirements

### Requirement: Mantlepkgs catalogs partition packages into typed domains

r[mantlepkgs_domains.domain_partition] Mantle MUST bind every catalog shard to one typed identity, class, source lock, owner label, package set, and named limits.

#### Scenario: Core and ecosystem shards are complete

GIVEN one bounded `core` shard and one named `ecosystem` shard contain complete typed facts
WHEN Mantle normalizes the shard manifests
THEN it MUST produce stable shard identities and ordered package records
AND the shard class MUST NOT grant package trust, correctness, or release authority.

#### Scenario: Shard metadata is invalid

GIVEN a shard has a floating source, unknown class, duplicate identity, missing owner label, unsafe path, or exceeded limit
WHEN Mantle normalizes the shard manifests
THEN it MUST reject the shard with ordered diagnostics
AND it MUST NOT read package artifacts or publish a partial catalog.

### Requirement: Domain composition is deterministic and conflict-free

r[mantlepkgs_domains.deterministic_composition] Mantle MUST compose normalized domain shards through a pure deterministic index and MUST reject ambiguous public selectors or conflicting identities.

#### Scenario: Equivalent shards arrive in different orders

GIVEN equivalent normalized shards and package records arrive in different input orders
WHEN the composition core builds the public index
THEN the ordered index, diagnostics, and BLAKE3 identity MUST match
AND filesystem order, hash-map order, and host paths MUST NOT affect the result.

#### Scenario: Two shards claim one selector

GIVEN two shards expose the same public selector with different package, root, policy, or artifact facts
WHEN the composition core evaluates the catalog
THEN it MUST reject the conflict with stable reason codes
AND it MUST NOT select a winner by precedence, input order, or last writer.

### Requirement: Package variants remain explicit

r[mantlepkgs_domains.explicit_variants] Mantle MUST represent each package variant as a typed record that binds its base package, variant name, changed policy, root identity, and provenance.

#### Scenario: Variant has a valid base

GIVEN a variant names one package in the same composed generation and contains complete bounded policy facts
WHEN Mantle validates the variant
THEN it MUST retain the base relationship in the machine record
AND a display selector MUST NOT replace that relationship as the only variant fact.

#### Scenario: Variant relationship is invalid

GIVEN a variant has a missing base, duplicate name, cycle, unsafe override, stale root, or exceeded limit
WHEN Mantle validates the variant
THEN it MUST reject the variant with an ordered diagnostic
AND it MUST NOT expose the variant as buildable.

### Requirement: Package validation uses separate roots

r[mantlepkgs_domains.separate_validation_roots] Mantle MUST model package validation as separate roots that consume package outputs and bind their own sources, tools, dependencies, policies, limits, and expected outcomes.

#### Scenario: Test-only inputs change

GIVEN a package recipe is unchanged and one validation source or test-only dependency changes
WHEN Mantle replans the package and validation root
THEN the package recipe and output identities MUST remain unchanged
AND the validation-root identity MUST change.

#### Scenario: Validation fails

GIVEN the package output exists and its separate validation root returns failure, timeout, or malformed evidence
WHEN Mantle records the validation result
THEN it MUST retain the package output as a distinct observed artifact
AND release policy MAY reject that package without rewriting its output identity.

### Requirement: External package corpora use pinned provenance

r[mantlepkgs_domains.reference_corpus] Mantle MUST bind every external package corpus to an exact repository, revision, observed license, selection, producer policy, and artifact identities.

#### Scenario: Pinned corepkgs corpus is evaluated

GIVEN a reviewed `corepkgs` revision and bounded package selection
WHEN the foreign producer evaluates the corpus
THEN Mantle MUST retain complete graph, source, catalog, blocker, and producer evidence
AND later catalog consumption MUST NOT require copied Ekala product code or an ambient Nix evaluator.

#### Scenario: Corpus identity is incomplete

GIVEN the corpus revision floats, the license record is absent, a selected artifact is missing, or an artifact digest differs
WHEN Mantle admits the corpus evidence
THEN it MUST reject the corpus with a stable reason code
AND it MUST NOT publish a success catalog from the incomplete evidence.

### Requirement: Catalog structure decisions use a functional core

r[mantlepkgs_domains.functional_core] Mantle MUST keep shard normalization, composition, conflict detection, variant validation, validation-root planning, diagnostic ordering, and identity construction pure and deterministic.

#### Scenario: Shell state differs

GIVEN the shell observes files, environment values, clocks, networks, or producer results that differ from explicit core inputs
WHEN Mantle compares shell observations with the core plan
THEN it MUST reject the mismatch before publication
AND the core MUST NOT repair its decision from ambient state.

#### Scenario: Input exceeds a named bound

GIVEN shard, package, alias, variant, validation-root, diagnostic, or artifact counts exceed typed policy
WHEN the core validates the input
THEN it MUST return a typed limit failure before unbounded work
AND the shell MUST NOT continue with a truncated success catalog.

### Requirement: Package-domain claims stay bounded

r[mantlepkgs_domains.claim_boundary] Mantle MUST limit package-domain evidence to exact locks, selections, systems, policies, artifacts, validation roots, and receipts.

#### Scenario: External corpus validation succeeds

GIVEN the selected corpus packages compose and their selected validation roots pass
WHEN Mantle writes the evidence summary
THEN it MAY report success for that exact corpus and policy
AND it MUST NOT claim package correctness, broad ecosystem coverage, evaluator parity, reproducibility, or release eligibility.
