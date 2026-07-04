## ADDED Requirements

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
