## ADDED Requirements

### Requirement: Provider fixed-point release artifact binding

r[rust_package_planning.provider_fixed_point_release_artifact_binding] Mantle MUST bind provider-backed Cargo-free fixed-point proof evidence to the packaged release artifact digest before reporting provider-backed release artifact evidence.

#### Scenario: release creation packages only matching provider proof evidence

GIVEN an operator runs `mantle release create --provider-fixed-point-proof <proof-dir>` with one or more `--binary` release artifacts
WHEN the provider fixed-point proof validates successfully
THEN release creation MUST require the proof's fixed-point stage binary BLAKE3 digest to match at least one bundled release binary artifact digest.
AND release creation MUST fail closed before writing trusted manifest evidence when no bundled release binary matches the provider proof stage binary digest.

#### Scenario: release verification rejects mismatched provider proof evidence

GIVEN a release evidence bundle records a provider fixed-point proof artifact or an operator supplies `--provider-fixed-point-proof <proof-dir>`
WHEN `mantle release verify --require-provider-fixed-point-proof` validates that proof
THEN release verification MUST require the proof's fixed-point stage binary BLAKE3 digest to match at least one release binary recorded in the manifest.
AND release verification MUST fail closed with a deterministic blocker when the proof is otherwise valid but does not match the release artifact set.

#### Scenario: matched provider proof reports release artifact identity

GIVEN provider fixed-point proof evidence matches a packaged release binary artifact
WHEN Mantle renders human or JSON release verification output
THEN the output MUST identify the matched release artifact relative path and BLAKE3 digest alongside the provider proof status.
AND the output MUST keep deterministic-release, reproducibility, bootstrap, compiler-correctness, and full-Cargo-compatibility claims separate unless separate evidence proves them.
