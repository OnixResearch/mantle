# Mantlepkgs Specification

## Purpose

Defines the `mantlepkgs` capability.

## Requirements

### Requirement: Mantlepkgs selection uses a typed manifest

r[mantlepkgs.catalog_manifest] Mantle MUST define a typed Nickel manifest for each Mantlepkgs generation. It MUST bind the producer source lock, systems, package selectors, aliases, conversion policy, source policy, output layout, and named limits.

#### Scenario: Complete manifest is accepted

GIVEN a manifest contains exact source identity, supported systems, unique selectors, valid policies, confined output paths, and bounded limits
WHEN Mantle validates the manifest
THEN it MUST produce one deterministic normalized selection plan
AND host paths, traversal order, and Nickel record order MUST NOT change plan identity.

#### Scenario: Invalid selection fails before production

GIVEN a manifest has a floating source, duplicate alias, unsupported system, unsafe output path, missing policy, or invalid limit
WHEN Mantle validates the manifest
THEN it MUST reject the manifest with ordered diagnostics
AND it MUST NOT run Nix or write generated artifacts.

### Requirement: Nixpkgs evaluation remains a producer action

r[mantlepkgs.producer_boundary] Mantle MUST keep Nixpkgs evaluation in an explicit producer shell. Published artifacts MUST bind the Nix implementation, Nixpkgs lock, systems, selectors, command class, graph identities, package indexes, source inventory, and non-claims.

#### Scenario: Locked producer emits complete inputs

GIVEN an accepted manifest selects packages from one locked Nixpkgs source
WHEN the producer evaluates those selections and publishes its result
THEN every selected package MUST have concrete graph and package-index facts or one explicit failure disposition
AND the producer receipt MUST bind all emitted artifact identities.

#### Scenario: Consumer has no Nix command

GIVEN a complete published Mantlepkgs generation
WHEN Mantle validates, selects, plans, or rebuilds a package
THEN it MUST use only the published generation and admitted source records
AND it MUST NOT execute Nix, `nix-store`, flake evaluation, overlay logic, or another Nix evaluator.

### Requirement: Catalog generation is deterministic and complete

r[mantlepkgs.catalog_generation] Mantle MUST generate one deterministic catalog from all selected package graphs. It MUST deduplicate identical foreign nodes, reject conflicting nodes, bind relative artifact paths and BLAKE3 digests, record source requirements, and publish atomically.

#### Scenario: Shared dependencies are emitted once

GIVEN two selected packages contain an identical foreign derivation node
WHEN the converter merges their graphs
THEN the catalog generation MUST contain one canonical node for that foreign identity
AND both package roots MUST resolve through the same compiled dependency.

#### Scenario: One selected package fails

GIVEN one selected package converts and another selected package has a graph conflict or missing root
WHEN catalog generation completes
THEN the batch MUST return failure with a disposition for each selection
AND Mantle MUST NOT publish a success catalog that omits the failed package.

#### Scenario: Catalog publication is atomic

GIVEN every selected package and referenced artifact passes staged validation
WHEN Mantle publishes the generation
THEN readers MUST observe either the prior complete generation or the new complete generation
AND a copy, hash, validation, or commit failure MUST NOT expose a partial final generation.

### Requirement: Mantlepkgs rebuilds use recomputed Mantle identities

r[mantlepkgs.recomputed_rebuild] A buildable Mantlepkgs entry MUST compile through the accepted foreign graph compiler with recomputed target paths. Target derivation, output, plan, and receipt identities MUST use Mantle BLAKE3 domains under the configured target store prefix.

#### Scenario: Selected package rebuilds without substitution

GIVEN a buildable catalog entry has complete admitted sources and a supported execution profile
WHEN Mantle realizes it with substitution disabled
THEN the ordinary Mantle worker MUST build the complete target graph under recomputed identities
AND the realization receipt MUST identify each output as built rather than imported from the original Nix cache.

#### Scenario: Parent uses the recomputed child output

GIVEN a selected parent references one output from a converted child
WHEN Mantle compiles and builds the target graph
THEN the parent MUST receive the exact recomputed child output path
AND no parent recipe field MAY retain the mapped original child store path.

### Requirement: Unsupported packages remain visible and unbuildable

r[mantlepkgs.unsupported_package_diagnostics] Mantle MUST assign deterministic blockers to selected packages with unsupported or incomplete behavior. It MUST NOT silently rewrite opaque source bytes, omit the package, use an ambient foreign store path, or call Nix as fallback.

#### Scenario: Unsupported package receives blockers

GIVEN a selected package has an unsupported builtin, missing source, unresolved store path, graph fault, ambiguous alias, or unsupported hard-coded source-store assumption
WHEN catalog generation evaluates that package
THEN the catalog report MUST retain its selector and ordered blocker codes
AND the buildable package map MUST NOT expose that entry as ready.

#### Scenario: Blocker is repaired in a later generation

GIVEN a later locked generation supplies supported facts for a previously blocked selector
WHEN the converter evaluates the new complete input
THEN it MAY expose the package as buildable under a new catalog identity
AND it MUST retain no success claim for the earlier blocked generation.

### Requirement: Conversion decisions use a functional core

r[mantlepkgs.functional_core] Manifest normalization, graph merging, conflict detection, package disposition, artifact planning, diagnostic ordering, and canonical identity construction MUST be pure deterministic functions over explicit in-memory inputs.

#### Scenario: Equivalent inputs produce equal plans

GIVEN equivalent manifest, graph, index, policy, and source facts arrive in different input orders
WHEN the core plans both catalog generations
THEN their normalized plans, package dispositions, diagnostics, and BLAKE3 identities MUST match
AND the core MUST NOT read files, inspect the environment, run processes, use clocks, access the network, or write output.

#### Scenario: Shell effects cannot change a core decision

GIVEN the shell observes a staged file or producer result that differs from the facts supplied to the core
WHEN pre-publication validation compares the observation with the plan
THEN publication MUST fail with a deterministic mismatch
AND the shell MUST NOT repair the core result from ambient state.

### Requirement: Mantlepkgs claims stay bounded

r[mantlepkgs.claim_boundary] Catalog generation and successful rebuilding MUST remain bounded to the exact locked selection, graphs, sources, policies, profiles, system, and receipts. They MUST NOT claim Nix source translation, evaluator parity, full Nixpkgs coverage, package correctness, reproducibility, bootstrap parity, or release eligibility.

#### Scenario: Rebuilt cohort is reported accurately

GIVEN every package in the selected validation cohort rebuilds under Mantle
WHEN Mantle writes the proof summary
THEN it MAY claim successful conversion and rebuilding for that exact cohort
AND it MUST list all broader package-set and semantic claims as unsupported.

### Requirement: Mantlepkgs validation includes success and failure

r[mantlepkgs.validation] The change MUST retain positive and negative fixtures plus a live pinned rebuild proof before archive. Validation MUST cover catalog identity, source readiness, dependency rewriting, package selection, failure isolation, and the no-Nix consumer boundary.

#### Scenario: Positive cohort passes

GIVEN a pinned cohort includes an executable root, a library root, and a root with a selected library dependency
WHEN the production converter and rebuild path run with substitution disabled
THEN all accepted roots MUST reach built state with exact retained evidence
AND later consumer steps MUST succeed with Nix commands absent from `PATH`.

#### Scenario: Negative fixtures fail closed

GIVEN fixtures contain stale locks, missing sources, digest tampering, alias conflicts, cycles, unsupported builtins, unresolved paths, or partial batch failure
WHEN the production paths process them
THEN each fixture MUST fail with its expected stable blocker
AND no fixture MAY publish a success catalog or successful realization receipt.
