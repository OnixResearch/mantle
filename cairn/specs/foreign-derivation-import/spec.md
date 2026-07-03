# Foreign Derivation Import Specification

## Purpose

Defines an adapter-neutral contract for importing foreign store-based derivation graphs and package indexes without coupling the translation core to Mantle, Guix, Nix evaluator or flake semantics, or a specific package frontend.

## Requirements

### Requirement: Adapter-neutral foreign derivation IR

r[foreign_derivation_import.adapter_neutral_ir] Foreign derivation import MUST define a versioned, backend-neutral IR for store-based derivation graphs. `foreign-derivation-graph-v1` MUST model schema version, producer/provenance summary, source store prefixes, target prefix when already selected, root derivation identities, bounded derivation nodes, source payload descriptors, and unsupported feature records. Each node MUST model stable node ID, original derivation identity, name, system, builder, args, environment, output declarations, input derivation edges, source/input refs, fixed-output metadata, builtin operation identifiers, declared references, sandbox capability needs, and unsupported feature classes without requiring Mantle-specific `.ncl`, Guix package modules, Nix expressions, or Nix flake outputs.

#### Scenario: Guix-like graph is represented as data

GIVEN a raw foreign graph produced from a pinned Guix-like package frontend
WHEN the graph is converted into `foreign-derivation-graph-v1`
THEN the IR MUST preserve graph nodes, declared dependencies, fixed-output source inputs, builtin operation identifiers, and package aliases as data
AND consumers MUST NOT need the foreign frontend runtime to inspect the IR.

#### Scenario: Nix-like graph is represented as data

GIVEN a raw foreign graph produced from Nix `.drv` files or derivation-JSON export
WHEN the graph is converted into `foreign-derivation-graph-v1`
THEN the IR MUST preserve concrete derivation graph facts, output declarations, input derivation edges, fixed-output facts, builtin operation identifiers, and package aliases as data
AND consumers MUST NOT need Nix expression evaluation, flake output evaluation, overlay application, or `nix-store` execution to inspect the IR.

#### Scenario: Required IR fields are explicit and canonical

GIVEN a producer emits graph facts with nodes, edges, environment maps, output maps, source payload descriptors, and unsupported feature records
WHEN the importer canonicalizes `foreign-derivation-graph-v1`
THEN it MUST order records deterministically, enforce named field and collection limits, and reject duplicate node IDs, duplicate outputs, dangling edges, missing root identities, missing source payload refs, or unknown mandatory fields
AND it MUST NOT preserve host traversal order, temporary generator paths, or frontend-specific object layouts as canonical identity.

#### Scenario: Frontend composition metadata is not the IR

GIVEN a producer supplies Nix flake outputs, Guix package module names, overlays, package-set replacement policy, or Onix module-layer selection data
WHEN the importer validates the foreign derivation IR
THEN it MUST either map those facts into explicit derivation graph or package-index data or reject them as unsupported frontend composition metadata
AND it MUST NOT silently treat them as core import semantics.

#### Scenario: Translation policy stays separate from graph facts

GIVEN a graph IR is paired with store-prefix rewrite, builtin mapping, fetch, cache, package-index, or sandbox compatibility policy
WHEN the importer computes graph identity and translated artifact identity
THEN the raw graph digest MUST be computed from graph facts only and each policy MUST have its own digest bound by the import receipt
AND policy changes MUST NOT be hidden as unrecorded edits to frontend graph facts.

### Requirement: Pure deterministic translation core

r[foreign_derivation_import.pure_translation_core] Foreign derivation translation MUST be implemented as a pure deterministic core over in-memory graph facts and explicit policy. The core MUST NOT read files, inspect environment variables, access stores, contact networks, execute processes, use clocks, or invoke Mantle, Guix, Nix, flake evaluators, or package-manager CLIs. It MUST return canonical translated graph facts, deterministic diagnostics, and receipt inputs.

#### Scenario: Equivalent graph order translates identically

GIVEN two equivalent foreign graph inputs with different node, edge, map, or package-index traversal order
WHEN the translation core canonicalizes and translates them with the same policy
THEN it MUST produce the same translated graph digest and diagnostics ordering
AND host temp paths, generator work directories, and map iteration order MUST NOT affect the result.

#### Scenario: Unsupported builtin fails closed

GIVEN a foreign derivation node uses a builtin operation not declared in the builtin mapping policy
WHEN the translation core validates the node
THEN it MUST reject the graph with a deterministic unsupported-builtin diagnostic
AND it MUST NOT invoke an external tool or silently approximate the builtin.

### Requirement: Explicit store-prefix rewrite policy

r[foreign_derivation_import.store_prefix_rewrite_policy] Foreign derivation import MUST require an explicit store-prefix rewrite policy before translating store paths. The policy MUST declare source prefixes, target prefix, fields eligible for rewrite, embedded source-payload rewrite permissions, builtin mapping, output-path recomputation mode, and unsupported reference behavior. Unknown foreign store references, target-prefix mismatches, or undeclared embedded rewrites MUST fail closed.

#### Scenario: Declared prefix rewrite succeeds

GIVEN a graph contains only store references under a declared source prefix
AND the rewrite policy declares a target prefix and output-path recomputation mode
WHEN translation runs
THEN every eligible builder, argument, environment, input, source, and reference field MUST be rewritten according to policy
AND the translated graph receipt MUST bind the policy digest and target prefix.

#### Scenario: Undeclared embedded store path is rejected

GIVEN a source payload or derivation field contains a foreign store path outside the declared rewrite fields or source prefixes
WHEN translation validates embedded references
THEN it MUST reject the graph with a deterministic undeclared-foreign-reference diagnostic
AND it MUST NOT rewrite the path silently or leave it as a hidden runtime dependency.

### Requirement: Import receipts bind provenance and non-claims

r[foreign_derivation_import.import_receipt] Foreign derivation import MUST emit deterministic receipts that bind producer identity, generator identity when known, channel or source revision facts, root derivation identities, raw graph BLAKE3 digest, translation policy BLAKE3 digest, translated graph BLAKE3 digest, package-index digest when present, fetch/cache policy digest, sandbox compatibility policy, disabled-test policy, and explicit non-claims. The receipt MUST NOT by itself claim build success, package correctness, bootstrap parity, output trust, or reproducibility.

#### Scenario: Receipt is reviewable without foreign frontend

GIVEN a translated foreign graph has an import receipt
WHEN a consumer or reviewer inspects the receipt
THEN the receipt MUST identify the graph roots, provenance facts, policy digests, translated graph digest, package-index digest when present, and non-claims
AND the reviewer MUST NOT need to run the foreign frontend to understand what was imported.

#### Scenario: Stale receipt is rejected

GIVEN a graph, translation policy, package index, fetch/cache policy, or sandbox compatibility policy differs from the facts bound in an existing receipt
WHEN the translated graph is admitted for planning or realization
THEN admission MUST reject the stale receipt with deterministic diagnostics
AND it MUST NOT treat the old receipt as proof for the changed import artifact.

### Requirement: Operator trust model documentation

r[foreign_derivation_import.operator_trust_model_docs] Mantle documentation MUST explain the trust boundaries for foreign derivation import receipts, including graph provenance, policy digests, source verification, cache/substitution trust, sandbox capabilities, realization, and output verification.

#### Scenario: Operator can identify trust boundaries

GIVEN an operator reads the foreign import trust-model guide
WHEN they review a foreign import receipt
THEN the guide MUST explain which facts the receipt binds and which trust decisions remain separate
AND it MUST distinguish import admission from build success and output trust.

#### Scenario: Guide links from proof documentation

GIVEN operator proof documentation mentions foreign import evidence
WHEN a reader follows related documentation
THEN the trust-model guide MUST be reachable from the proof guide or README
AND the linked text MUST preserve Mantle naming while allowing exact schema/command identifiers.

### Requirement: Receipt non-claims are documented

r[foreign_derivation_import.receipt_non_claims_documentation] Foreign import documentation MUST state that an import receipt alone does not claim build success, package correctness, bootstrap parity, output trust, reproducibility, or foreign-frontend availability.

#### Scenario: Receipt is not mistaken for proof success

GIVEN a valid foreign import receipt exists for a translated graph
WHEN an operator reads the documentation
THEN the documentation MUST state that additional realization and verification evidence is required before claiming trusted outputs
AND it MUST NOT present receipt existence as proof of correctness.

#### Scenario: Cache hints are not output trust

GIVEN a receipt records cache or substitution metadata
WHEN the documentation explains cache reuse
THEN it MUST state that cache hints remain subject to store/substitution trust policy
AND they MUST NOT bypass output admission or signature verification.

### Requirement: Trust model documentation guard

r[foreign_derivation_import.trust_model_guard] Mantle SHOULD include a lightweight guard that fails when required foreign import trust-model headings or non-claim language disappear from operator documentation.

#### Scenario: Guard accepts complete docs

GIVEN the trust-model guide includes required trust-boundary and non-claim sections
WHEN the guard runs
THEN it MUST pass and report the checked sections.

#### Scenario: Guard rejects missing non-claims

GIVEN the trust-model guide omits receipt non-claim language
WHEN the guard runs in negative or self-test mode
THEN it MUST fail with a deterministic diagnostic
AND it MUST identify the missing section or phrase class.

### Requirement: Package index stays format-neutral

r[foreign_derivation_import.package_index_boundary] Foreign package-set import MUST expose package indexes as versioned, format-neutral by-name data. The index MUST bind package names, systems, root derivation identities, optional aliases, provenance refs, package metadata digests, and unsupported metadata classes. Nix flakes, overlays, Guix module paths, and package replacement policy MAY be adapter metadata, but they MUST NOT be the stable import ABI.

#### Scenario: By-name package lookup selects a root

GIVEN a `foreign-package-index-v1` contains a package named `hello` for a selected system
WHEN a consumer asks for that package through a supported adapter
THEN lookup MUST return the root derivation identity and provenance ref deterministically
AND it MUST NOT require flake output evaluation, overlay application, or Guix module evaluation.

#### Scenario: Composition semantics are blockers

GIVEN a package index entry depends on overlay order, flake follows rewriting, package replacement policy, module-layer evaluation, or other frontend composition semantics that are not lowered into explicit graph data
WHEN the package index is validated
THEN validation MUST reject or mark the entry unsupported with deterministic diagnostics
AND it MUST NOT expose the entry as a buildable package root.

#### Scenario: Nix flake output snapshot is adapter metadata only

GIVEN a Nix adapter used flake evaluation outside the translation core to discover package names
WHEN it emits `foreign-package-index-v1`
THEN the index MUST bind only lowered package names, systems, root derivation identities, provenance refs, and unsupported metadata classes
AND later consumers MUST NOT need flake evaluation or overlay application to look up those package roots.

### Requirement: Fetch and cache policy remains explicit

r[foreign_derivation_import.fetch_and_cache_policy] Foreign derivation import MUST model fixed-output source acquisition and cache reuse as explicit policy data. Fixed-output fetch nodes MUST carry content refs and MAY carry ordered mirror candidates. Binary cache or substitution hints MUST be trust-scoped metadata and MUST NOT bypass store admission. Test disabling, mirror substitution, or source URL replacement MUST be explicit and receipt-bound.

#### Scenario: Ordered mirrors are source policy

GIVEN a fixed-output source input has multiple mirror candidates and a content ref
WHEN the source acquisition plan is derived from the translated graph
THEN the plan MUST preserve deterministic mirror order and content-ref verification
AND a failed mirror MUST NOT change the accepted source identity.

#### Scenario: Untrusted cache hint is not output trust

GIVEN a translated package root includes cache or substituter metadata without a matching trust policy
WHEN realization planning evaluates reuse
THEN the cache hint MUST be rejected or reported as ineligible
AND it MUST NOT satisfy output trust, receipt admission, or build success claims.

### Requirement: Sandbox compatibility is audited per derivation

r[foreign_derivation_import.sandbox_capability_audit] Foreign derivation compatibility behavior MUST be modeled as explicit per-derivation sandbox capability data. Compatibility exceptions such as setuid or setgid chmod behavior, syscall-filter changes, network allowances, writable-prefix needs, or builder identity requirements MUST be declared, policy-checked, and audit-reported. They MUST NOT be hidden behind global impurity.

#### Scenario: Declared compatibility capability is reported

GIVEN a foreign derivation requires a compatibility capability allowed by policy
WHEN the graph is admitted for planning
THEN the resulting plan or receipt MUST identify the derivation, capability, policy basis, and audit classification
AND the capability MUST be scoped to that derivation or declared closure segment.

#### Scenario: Undeclared compatibility need blocks realization

GIVEN a foreign derivation would require a sandbox exception not declared in its compatibility policy
WHEN planning or execution policy validation reaches that derivation
THEN the route MUST fail closed with a deterministic sandbox-capability diagnostic
AND it MUST NOT silently switch to an impure or globally relaxed sandbox mode.

### Requirement: Host build-tool integration is a thin adapter

r[foreign_derivation_import.integration_boundary] Foreign derivation import integration MUST keep host build-tool coupling in a thin adapter. Mantle integration MUST consume accepted translated graph artifacts through generic build-plan, source-bundle, store-transport, or dynamic-plan APIs, and other consumers MUST be able to consume the same canonical artifacts without linking to Mantle CLI behavior. Realization MUST NOT require Guix, Nix, flake evaluation, Nix expression evaluation, overlay application, package-module evaluation, or foreign frontend runtimes on the consuming side.

#### Scenario: Mantle adapter consumes canonical artifacts

GIVEN a translated graph artifact and receipt have been accepted by the import core
WHEN Mantle plans or realizes the imported root through its adapter
THEN the adapter MUST map canonical graph facts into ordinary build inputs or dynamic-plan units
AND Mantle core MUST NOT gain Guix package semantics, Nix flake output semantics, overlay semantics, or package-set replacement semantics.

#### Scenario: Non-Mantle consumer can inspect the artifact

GIVEN another store-based consumer supports the foreign derivation import IR
WHEN it reads the translated graph and receipt
THEN it MUST be able to validate graph identity, policy identity, package roots, and non-claims from the canonical artifact
AND it MUST NOT need Mantle-specific CLI flags, Mantle project files, or Mantle `.ncl` evaluation to understand the import.

#### Scenario: Nix adapter output is consumed without Nix

GIVEN a Nix adapter has already exported a concrete derivation graph into the foreign derivation IR
WHEN Mantle or another consumer plans from that artifact
THEN planning MUST use the canonical graph and receipt data
AND it MUST NOT invoke `nix`, `nix-store`, Nix expression evaluation, flake output evaluation, or overlay application during consumption.
