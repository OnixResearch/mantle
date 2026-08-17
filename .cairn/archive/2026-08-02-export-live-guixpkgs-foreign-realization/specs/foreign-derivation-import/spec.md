## ADDED Requirements

### Requirement: Live GuixPkgs export and realization proof

r[foreign_derivation_import.live_guixpkgs_export_realization] Mantle MUST retain reproducible evidence for one pinned GuixPkgs package export that reaches receipt-bound realization and a bounded provenance-audit disposition without Guix or Nix during consumption.

#### Scenario: Producer export binds translation identity

GIVEN a pinned GuixPkgs revision records its upstream Guix revision and `guix-transfer` input
WHEN producer-side Nix exports the recursive `hello.unwrapped` derivation graph
THEN the evidence MUST bind those revisions, the selected package attribute, graph, root derivation, output path, policy, and plan identities
AND the producer boundary MUST finish before Mantle consumption starts.

#### Scenario: Producer exports a signed translated closure

GIVEN producer-side Nix realizes the selected translated output and its complete runtime closure
WHEN the producer signs and exports those paths under a dedicated proof key
THEN every exported NARInfo MUST bind the exact translated path, references, NAR hash, NAR size, and exporter signature
AND the secret signing key MUST NOT enter retained evidence or consumer state.

#### Scenario: Signed translated closure reaches realized state

GIVEN the selected translated output and its runtime references have trusted exporter signatures
WHEN Mantle consumes the cache-only plan with Nix and Guix commands absent from `PATH`
THEN the bounded closure MUST reach `realized` through Mantle's signed PathInfo, NAR, castore, scheduler, worker, and store boundaries
AND no local or remote builder MAY execute as fallback.

#### Scenario: Reuse, hydration, and provenance are checked

GIVEN the translated closure completed once
WHEN Mantle repeats realization, hydrates a fresh store from the receipt-selected root, and runs provenance audit
THEN exact reuse MUST avoid builder execution and fresh hydration MUST admit the same complete signed closure
AND provenance evidence MUST retain the exact bounded findings and strongest valid state.

#### Scenario: Invalid evidence fails closed

GIVEN the cache key, closure limits, realization receipt, required member, path, signature, or NAR fact is invalid
WHEN Mantle preflights or consumes the closure
THEN it MUST return a stable bounded failure and MUST NOT report complete realization
AND preflight rejection MUST NOT mutate the target store or create a realization or audit receipt.

#### Scenario: Proof keeps explicit non-claims

GIVEN the live GuixPkgs evidence is complete
WHEN an operator reviews the proof
THEN Guix evaluator parity, translation correctness, direct Guix cache authentication, original `/gnu/store` identity, local rebuild compatibility, package correctness, reproducibility, bootstrap parity, runtime safety, deployment, and release eligibility MUST remain non-claims
AND future Guix, GuixPkgs, `guix-transfer`, Nixpkgs, Cachix, and cache-content revisions MUST remain outside the proof.
