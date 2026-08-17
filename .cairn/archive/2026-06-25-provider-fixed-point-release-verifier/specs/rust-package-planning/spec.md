## ADDED Requirements

### Requirement: Provider fixed-point release verifier

r[rust_package_planning.provider_fixed_point_release_verifier] Mantle MUST let release verification validate a provider-backed Cargo-free fixed-point proof bundle as bounded release-adjacent evidence without upgrading the release reproducibility claim.

#### Scenario: valid provider fixed-point proof is reported separately

GIVEN an operator runs `mantle release verify <bundle-dir> --provider-fixed-point-proof <proof-dir>`
WHEN the supplied proof bundle contains successful stage1 and stage2 summaries with matching Mantle binary BLAKE3 digests, absent Cargo markers, successful smoke checks, successful stage receipts, and enforced source-built closure policy digests
THEN release verification MUST report the provider fixed-point proof status separately from reproducibility and deterministic release eligibility.
AND it MUST report the closure policy digest and matching stage binary digest.

#### Scenario: required provider fixed-point proof fails closed

GIVEN an operator runs `mantle release verify <bundle-dir> --require-provider-fixed-point-proof`
WHEN the provider fixed-point proof path is missing, malformed, has failed stages, has mismatched stage binary digests, lacks enforced source-built closure status, or omits bounded non-claims
THEN release verification MUST fail closed with deterministic blockers that name the invalid proof condition.

#### Scenario: fixed-point proof stays bounded

GIVEN a provider fixed-point proof verifies successfully during release verification
WHEN Mantle renders human or JSON release verification output
THEN the result MUST keep release reproducibility and full Cargo compatibility non-claims visible.
AND it MUST NOT set deterministic release eligibility or reproducibility status from fixed-point proof evidence alone.
