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

### Requirement: Operator CLI surface for foreign imports

r[foreign_derivation_import.operator_cli_surface] Mantle MUST provide a thin operator CLI for validating and planning from foreign derivation import artifacts without requiring the foreign frontend at consumption time.

#### Scenario: Operator validates lowered artifacts

GIVEN a foreign derivation graph artifact, optional package index, and translation policy file
WHEN an operator runs the validation command
THEN Mantle MUST validate the artifacts through the import core and report deterministic diagnostics
AND it MUST NOT require Guix, Nix, flake evaluation, Nix expression evaluation, overlays, or package-module evaluation.

#### Scenario: Operator emits an adapter plan

GIVEN valid lowered artifacts and a target package/root selection
WHEN an operator runs the planning command
THEN Mantle MUST emit a receipt-bound adapter plan JSON document
AND the plan MUST identify translated roots, policy digests, source acquisition hints, sandbox audit events, and explicit non-claims.

### Requirement: CLI file boundary stays outside the core

r[foreign_derivation_import.cli_file_boundary] Foreign import CLI integration MUST keep filesystem reads, JSON decoding, stdout/stderr rendering, and process exit behavior in the imperative shell while preserving the translation core as pure in-memory logic.

#### Scenario: Core remains file-system independent

GIVEN the CLI reads artifact files from disk
WHEN it invokes translation or admission logic
THEN it MUST pass owned in-memory data into the core
AND the core MUST NOT read files, inspect environment variables, execute processes, or render CLI JSON directly.

#### Scenario: CLI failure reports core diagnostics

GIVEN an input artifact is invalid
WHEN the CLI reports the failure
THEN it MUST preserve the core diagnostic class and path in machine-readable output
AND it SHOULD bound human-readable summaries.

### Requirement: Checked fixtures exercise the CLI contract

r[foreign_derivation_import.checked_fixtures] Mantle MUST ship small checked-in Guix-like and Nix-like foreign import fixtures that can validate the CLI contract without live foreign frontends.

#### Scenario: Guix-like fixture plans without Guix

GIVEN a checked-in Guix-like hello fixture lowered into the foreign import IR
WHEN the CLI validates and plans the fixture
THEN the command MUST succeed without executing `guix`
AND the emitted receipt MUST bind the fixture graph and policy digests.

#### Scenario: Nix-like fixture plans without Nix

GIVEN a checked-in Nix-like hello fixture lowered from `.drv` or derivation-JSON facts
WHEN the CLI validates and plans the fixture
THEN the command MUST succeed without executing `nix`, `nix-store`, flake evaluation, Nix expression evaluation, or overlay application
AND the emitted receipt MUST bind the fixture graph and policy digests.

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

### Requirement: Nixpkgs producer adapter emits foreign import artifacts

r[foreign_derivation_import.nixpkgs_producer_adapter] Nixpkgs support MUST be implemented as a producer adapter that emits `foreign-derivation-graph-v1`, `foreign-package-index-v1`, and import receipt inputs from concrete Nix derivation facts. The adapter MUST NOT make Nix expressions, flakes, overlays, or nixpkgs package-set semantics part of Mantle's stable import ABI.

#### Scenario: concrete nixpkgs derivation closure becomes graph data

GIVEN a nixpkgs package has been selected outside the import core
AND the producer has concrete `.drv` files or derivation-JSON closure facts for that package
WHEN the nixpkgs adapter emits foreign import artifacts
THEN the graph MUST preserve derivation nodes, outputs, input derivation edges, fixed-output source facts, environment fields, builtin operation identifiers, and declared references as data
AND the package index MUST map the selected package name and system to the lowered root derivation identity.

#### Scenario: package index is bounded adapter output

GIVEN the producer discovers nixpkgs package names through a flake output, attribute path, or package-index query
WHEN it writes `foreign-package-index-v1`
THEN the index MUST contain only lowered package names, systems, aliases, root derivation identities, provenance refs, metadata digests, and unsupported metadata classes
AND consumers MUST NOT need to re-run the package discovery mechanism to look up those roots.

### Requirement: Nixpkgs evaluation stays outside consumption

r[foreign_derivation_import.nixpkgs_eval_boundary] Nixpkgs integration MUST keep Nix expression evaluation, flake output lookup, overlay application, and optional `snix-eval` execution in a producer shell before artifact emission. Mantle validation, planning, substitution, and realization from accepted artifacts MUST NOT invoke Nix, `nix-store`, flakes, overlays, Nix expression evaluation, package-set replacement logic, or `snix-eval`.

#### Scenario: host Nix producer is allowed before artifact emission

GIVEN an operator chooses a host-Nix producer mode to export `nixpkgs#hello`
WHEN the producer emits graph and package-index artifacts
THEN the producer receipt MUST record the producer command class, nixpkgs revision or lock identity when known, selected attribute or package name, and non-claims
AND later Mantle consumption MUST use only the emitted artifacts.

#### Scenario: optional snix-eval remains a producer backend

GIVEN a future adapter uses `snix-eval` to evaluate nixpkgs or flake outputs
WHEN the adapter emits foreign import artifacts
THEN `snix-eval` MUST be treated as a producer backend with recorded provenance
AND the import core and consuming Mantle build path MUST remain independent of `snix-eval` availability.

#### Scenario: frontend composition metadata blocks consumption

GIVEN a graph or package index contains overlay-order dependence, flake follows rewriting, package replacement policy, or other nixpkgs composition semantics that were not lowered into explicit derivation graph facts
WHEN Mantle validates the foreign import artifacts
THEN validation MUST reject the artifacts or mark the affected package entries unsupported with deterministic diagnostics
AND it MUST NOT silently approximate those frontend semantics.

### Requirement: Nix hash domain stays separate from Mantle hash domain

r[foreign_derivation_import.nix_hash_domain_boundary] The nixpkgs adapter MUST preserve Nix-compatible `.drv`, store-path, NAR, NARInfo, and binary-cache identity using the hash algorithms required by those formats while separately using BLAKE3 for Mantle-owned import receipts, policy digests, and translated artifact identities. The adapter MUST fail closed if a Mantle BLAKE3 digest is supplied as a Nix-compatible derivation or cache identity, or if a Nix-compatible digest is supplied where a Mantle receipt digest is required.

#### Scenario: `.drv` identity uses Nix-compatible hashing

GIVEN a nixpkgs `.drv` or derivation-JSON closure is parsed by the adapter
WHEN the adapter records original derivation identity and cache lookup facts
THEN those facts MUST use Nix-compatible hashing and store-path rules required by the source format
AND Mantle BLAKE3 derivation hashes MUST NOT be substituted for original Nix identities.

#### Scenario: receipt identity uses BLAKE3

GIVEN the adapter emits a raw graph, translation policy, package index, and import receipt
WHEN Mantle computes admission identity
THEN the raw graph digest, policy digest, translated graph digest, and receipt-bound artifact digests MUST use Mantle's BLAKE3 receipt domain
AND those BLAKE3 digests MUST NOT be presented as Nix cache keys or `.drv` hashes.

#### Scenario: modified format crates are audited before use

GIVEN Mantle uses vendored `nix-compat` or Snix crates for nixpkgs adapter work
WHEN a code path participates in Nix-compatible identity, cache lookup, or `.drv` parsing
THEN the implementation MUST prove that path uses upstream-compatible Nix hash semantics or an explicit adapter-local Nix hash mode
AND it MUST NOT rely on Mantle's BLAKE3-modified derivation hashing for Nix cache compatibility.

### Requirement: Nixpkgs import supports substitution-first planning

r[foreign_derivation_import.nixpkgs_substitution_first] Nixpkgs import MUST support a substitution-first compatibility level where binary-cache hints are admitted as explicit trust-scoped policy and outputs are accepted only through Mantle's normal PathInfo, NAR hash, signature, store-prefix, and attestation admission rules. Substitution-first receipts MUST NOT claim local rebuild support.

#### Scenario: trusted cache hint remains policy data

GIVEN a nixpkgs graph includes a cache hint for `cache.nixos.org` or another binary cache
AND the operator policy declares matching trusted public keys and cache scope
WHEN Mantle plans the imported root
THEN the plan MAY report the root as substitutable
AND the eventual output MUST still pass Mantle store admission before any output trust claim is made.

#### Scenario: untrusted cache hint is ineligible

GIVEN a nixpkgs graph includes cache metadata without matching trust policy
WHEN Mantle evaluates substitution-first planning
THEN the cache hint MUST be rejected or reported as ineligible with a deterministic diagnostic
AND it MUST NOT satisfy import admission, build success, or output trust claims.

#### Scenario: rebuild support is a separate level

GIVEN a nixpkgs import receipt was admitted through substitution-first planning
WHEN an operator reports the result
THEN the receipt MUST state that local rebuild compatibility is not proven unless a later Mantle build report proves it
AND rebuild blockers such as sandbox capability gaps, unsupported builtins, or missing source acquisition policy MUST remain visible.

### Requirement: Nixpkgs receipts document non-claims

r[foreign_derivation_import.nixpkgs_receipt_non_claims] Nixpkgs foreign import documentation and receipts MUST distinguish admitted, planned, substituted, rebuilt, and verified states. Admission-only or planning-only receipts MUST NOT claim nixpkgs package correctness, local rebuild success, output trust, bootstrap parity, reproducibility, or future availability of the producer frontend.

#### Scenario: operator sees strongest proven state

GIVEN an operator validates or plans an imported nixpkgs package
WHEN Mantle emits a report or receipt
THEN the report MUST name the strongest proven state among admitted, planned, substituted, rebuilt, and verified
AND it MUST list missing evidence classes as explicit non-claims.

#### Scenario: substitution is not package correctness

GIVEN Mantle successfully substitutes an imported nixpkgs output from a trusted cache
WHEN the operator trust-model documentation explains the result
THEN it MUST state that cache admission is output trust under the configured store policy, not proof of nixpkgs package correctness or reproducibility
AND package correctness or reproducibility claims MUST require separate evidence.

### Requirement: Live Nixpkgs export-to-plan proof is captured

r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof] Mantle SHOULD maintain current evidence that a real Nixpkgs package can be exported through host Nix into concrete derivation facts, lowered into foreign import artifacts, and consumed by Mantle validation/planning without Nix available during consumption. The proof MUST name the strongest proven state and MUST NOT claim substitution, local rebuild compatibility, output trust, package correctness, or reproducibility.

#### Scenario: live nixpkgs hello reaches planning without Nix during consumption

GIVEN host Nix resolves a live `nixpkgs#hello` derivation path
AND host Nix exports the recursive concrete derivation JSON before artifact emission
WHEN Mantle lowers that JSON with `foreign-import produce-nix`
THEN the evidence MUST include emitted `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifact paths
AND later `foreign-import validate` and `foreign-import plan` proof commands MUST run with a PATH that lacks `nix` and `nix-store`.

#### Scenario: live proof stays admission/planning scoped

GIVEN the live proof validates and plans the lowered artifacts
WHEN the evidence summarizes the result
THEN it MUST report the strongest proven state as admitted/planned
AND it MUST list substitution, local rebuild compatibility, output trust, package correctness, and reproducibility as non-claims.

### Requirement: Direct `.drv` producer parses concrete derivation closures

r[foreign_derivation_import.direct_drv_producer] The Nix producer adapter MUST accept an explicit concrete `.drv` closure input mode that parses Nix ATerm derivation files with Rust crate code and emits the same `foreign-derivation-graph-v1` plus `foreign-package-index-v1` artifacts as derivation-JSON input. The producer MUST NOT invoke `nix`, `nix-store`, flake evaluation, Nix expression evaluation, overlay logic, or host store closure discovery while parsing those explicit files.

#### Scenario: explicit drv closure becomes graph data

GIVEN an operator supplies a root derivation identity and explicit logical `.drv` path-to-file mappings for every derivation in the closure
WHEN the Nix producer parses the `.drv` files
THEN the emitted graph MUST preserve derivation nodes, outputs, input derivation edges, fixed-output metadata, source refs, environment fields, and declared references as data
AND the package index MUST map the selected package name and system to the lowered root derivation identity.

#### Scenario: malformed drv input fails closed

GIVEN an explicit `.drv` input file is malformed, unreadable, or does not parse as a Nix derivation ATerm
WHEN the Nix producer parses the explicit closure
THEN artifact production MUST fail with a deterministic diagnostic
AND it MUST NOT emit partial graph or package-index artifacts.
