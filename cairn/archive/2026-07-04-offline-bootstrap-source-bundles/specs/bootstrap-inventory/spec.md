## ADDED Requirements

### Requirement: Bootstrap inputs are source-bundle admissible

r[bootstrap_inventory.offline_bootstrap_source_bundles] Mantle MUST provide an offline bootstrap source-bundle profile that can describe, import, pin, and preflight the source/input material required by selected bootstrap and self-build workflows. The profile MUST bind provider archive identity, provider metadata, bootstrap source archives, Mantle source tree identity, vendored Cargo input identity when applicable, toolchain/source-root records, logical store prefix, fixed-output hashes, and proof-mode requirements before an offline bootstrap command can consume those inputs.

#### Scenario: complete bootstrap bundle permits offline source acquisition

GIVEN an operator imports and pins a bootstrap source bundle whose records match the selected bootstrap profile
AND the profile includes every source, provider, toolchain, source tree, and vendored input required by that mode
WHEN Mantle runs bootstrap preflight or an offline bootstrap command
THEN Mantle MAY use the imported source state instead of live network source fetches
AND the report MUST bind the profile digest and source-state digest used for the decision.

#### Scenario: incomplete bootstrap bundle fails closed

GIVEN a bootstrap source bundle is missing a required provider archive, provider manifest, source archive, source tree, vendored Cargo input, toolchain source-root record, or proof input for the selected mode
WHEN Mantle evaluates offline bootstrap readiness
THEN Mantle MUST reject the profile before bootstrap execution
AND diagnostics MUST name the missing or stale record class.

#### Scenario: provider metadata mismatch blocks offline bootstrap

GIVEN imported source state contains provider material with a wrong provider kind, unsupported metadata schema, stale fixed-output hash, mismatched logical store prefix, missing reduced-provider provenance, or normalized seed contract mismatch
WHEN Mantle validates the offline bootstrap profile
THEN Mantle MUST fail closed before consuming the provider
AND it MUST NOT downgrade to an online fetch or a broader bootstrap claim silently.

#### Scenario: source-bundle readiness is not bootstrap proof

GIVEN Mantle reports an offline bootstrap source bundle as ready
WHEN an evidence file, task, documentation page, or status reply cites that readiness
THEN the claim MUST be limited to bootstrap source/input material being locally available and identity-matched
AND it MUST NOT claim provider trust removal, compiler correctness, self-build success, release reproducibility, or full bootstrap correctness without separate proof evidence.
