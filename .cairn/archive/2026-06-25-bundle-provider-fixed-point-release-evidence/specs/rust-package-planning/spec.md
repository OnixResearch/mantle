## ADDED Requirements

### Requirement: Bundle-local provider fixed-point release evidence

r[rust_package_planning.bundle_provider_fixed_point_release_evidence] Mantle MUST let release evidence bundles carry a provider-backed Cargo-free fixed-point proof as bounded, verifiable release-adjacent evidence.

#### Scenario: release creation packages validated provider fixed-point proof

GIVEN an operator runs `mantle release create --provider-fixed-point-proof <proof-dir>`
WHEN the proof directory is a valid provider-backed Cargo-free fixed-point proof bundle
THEN release creation MUST copy the proof into the release evidence bundle and record a manifest artifact with its relative path, BLAKE3 digest, size, and bounded evidence role.
AND release creation MUST fail closed before writing trusted manifest evidence when the proof is malformed, mismatched, missing enforced source-built closure status, or missing bounded non-claims.

#### Scenario: release verification uses bundled proof when required

GIVEN a release evidence bundle records a provider fixed-point proof artifact
WHEN an operator runs `mantle release verify <bundle-dir> --require-provider-fixed-point-proof` without an external proof path
THEN release verification MUST validate the bundled proof artifact and report its status, artifact digest, closure policy digest, and matching stage binary digest.
AND release verification MUST fail closed with deterministic blockers when the manifest omits the proof artifact or the bundled proof is invalid.

#### Scenario: external proof override remains explicit

GIVEN a release evidence bundle records a provider fixed-point proof artifact
WHEN an operator also supplies `--provider-fixed-point-proof <proof-dir>`
THEN release verification MUST validate the explicitly supplied proof path and report that the external proof source was used.
AND the bundled proof artifact MUST remain recorded in the manifest output without being confused with the external override result.

#### Scenario: bundled fixed-point proof stays bounded

GIVEN release verification validates a bundled provider fixed-point proof
WHEN Mantle renders human or JSON release verification output
THEN the result MUST keep release reproducibility and full Cargo compatibility non-claims visible.
AND it MUST NOT set reproducibility status, deterministic release eligibility, or release artifact digest matching from provider fixed-point proof evidence alone.
