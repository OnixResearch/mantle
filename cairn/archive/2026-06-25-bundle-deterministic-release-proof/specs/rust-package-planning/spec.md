## ADDED Requirements

### Requirement: Bundle-local deterministic release proof

r[rust_package_planning.bundle_deterministic_release_proof] Mantle MUST let release evidence bundles carry deterministic-release proof receipts as bounded, verifiable bundle-local evidence.

#### Scenario: release reproduce attaches deterministic proof artifacts

GIVEN an operator runs `mantle release reproduce <bundle-dir>` with deterministic proof runs enabled
WHEN the repeated rebuild proof succeeds
THEN Mantle MUST write the deterministic build proof receipt and deterministic sandbox isolation evidence into the release evidence bundle and record manifest artifacts with relative paths, BLAKE3 digests, sizes, and bounded evidence roles.
AND Mantle MUST fail closed before recording trusted manifest evidence when a proof artifact is missing, malformed, or inconsistent with the rebuilt artifact digest set.

#### Scenario: release verification uses bundled deterministic proof when required

GIVEN a release evidence bundle records deterministic proof artifacts
WHEN an operator runs `mantle release verify <bundle-dir> --require-deterministic-release` without external deterministic proof paths
THEN release verification MUST validate the bundled deterministic proof receipt and sandbox isolation evidence and report deterministic-release eligibility from that proof.
AND verification MUST fail closed with deterministic blockers when the manifest omits required proof artifacts, any bundled proof artifact is missing, or proof digests do not match manifest evidence.

#### Scenario: external deterministic proof override remains explicit

GIVEN a release evidence bundle records deterministic proof artifacts
WHEN an operator supplies `--deterministic-proof <path>` and `--deterministic-sandbox-isolation-evidence <path>`
THEN release verification MUST validate the explicitly supplied proof paths and report that external deterministic proof evidence was used.
AND the bundle-local proof artifacts MUST remain recorded in manifest output without being confused with the external override result.

#### Scenario: bundled deterministic proof stays bounded

GIVEN release verification validates bundled deterministic proof artifacts
WHEN Mantle renders human or JSON release verification output
THEN the result MAY report deterministic-release eligibility for the packaged artifact set described by the proof.
AND it MUST NOT claim compiler correctness, full bootstrap reproducibility, full Cargo compatibility, deploy success, or physical-target determinism unless separate evidence proves those claims.
