## ADDED Requirements

### Requirement: Deterministic proof unit records exact rebuild inputs

The build pipeline MUST define a deterministic proof unit before executing proof rebuilds. The proof unit MUST select one target artifact/output set and MUST record workflow identity/version, selected provider kind, source tree BLAKE3, vendor/input bundle BLAKE3, toolchain/stage roots, logical store prefix, sandbox profile identity, and selected output identities.

The proof unit MAY target the Mantle self-build/release artifact or a smaller release artifact, but the receipt MUST name the selected target explicitly. The proof unit MUST NOT imply that unselected bootstrap stages, packages, platforms, or social witnesses were proven.

#### Scenario: Proof unit identifies one selected artifact

- GIVEN an operator requests a deterministic proof
- WHEN Mantle creates the proof plan
- THEN the plan names exactly one selected proof-unit target/output set
- AND records workflow/version, selected provider kind, source/vendor BLAKE3, toolchain/stage roots, and sandbox profile identity before rebuilds start

#### Scenario: Missing exact input identity blocks planning

- GIVEN a requested deterministic proof lacks source tree BLAKE3, vendor/input BLAKE3, selected provider kind, or workflow version
- WHEN Mantle plans the proof
- THEN planning fails closed before any proof claim can be emitted

### Requirement: Deterministic proof receipt compares two clean BLAKE3 rebuild sets

The build pipeline MUST support a canonical deterministic proof receipt schema `mantle-deterministic-proof-receipt-v1`. The receipt MUST be serialized as canonical compact JSON and identified by a BLAKE3 digest over the canonical bytes excluding the self-digest field.

The receipt MUST record rebuild A and rebuild B artifact digest sets using BLAKE3. The receipt verdict MUST be a closed value. `self-rebuild-match` MUST be emitted only when all proof-unit input identities validate, provider kind linkage matches, sandbox evidence is supported, proof-store/output anti-reuse checks pass, and rebuild A/B artifact digest sets match exactly.

Closed non-promoting verdicts MUST cover at least `not-attempted`, `mismatch`, `missing-evidence`, `reused-store`, `impure-mode`, `unsupported-workflow`, `unsupported-sandbox`, `provider-kind-mismatch`, and `malformed-receipt`.

#### Scenario: Canonical receipt bytes are stable

- GIVEN the same deterministic proof facts in any insertion order
- WHEN Mantle serializes the proof receipt
- THEN the canonical bytes are identical
- AND the receipt BLAKE3 digest is identical

#### Scenario: Matching clean rebuilds produce self-rebuild match

- GIVEN rebuild A and rebuild B were executed for the same proof unit
- AND each run used a distinct clean proof store and output root
- AND each run records supported `mantle-proof-sandbox-v1:*` sandbox evidence
- AND every selected output has the same canonical BLAKE3 digest in both runs
- WHEN Mantle finalizes the receipt
- THEN the verdict is `self-rebuild-match`

#### Scenario: Digest drift fails the proof

- GIVEN rebuild A and rebuild B completed for the same proof unit
- WHEN any selected output has a different BLAKE3 digest between runs
- THEN the receipt verdict is `mismatch`
- AND Mantle MUST NOT claim `self-rebuild-match`

#### Scenario: Provider kind mismatch fails the proof

- GIVEN the proof unit records one selected provider kind
- BUT rebuild evidence, proof linkage, or prerequisites record a different provider kind
- WHEN Mantle validates the receipt
- THEN the receipt verdict is `provider-kind-mismatch` or validation fails closed
- AND Mantle MUST NOT claim `self-rebuild-match`

### Requirement: Determinism proof runs use isolated clean stores and supported sandbox evidence

A deterministic proof attempt MUST run each comparison build in a clean store namespace or fresh physical store directory that cannot reuse prior output artifacts for the proof-unit target. The first implementation MUST schedule at least rebuild A and rebuild B and MUST allocate distinct proof-store and output-root identities for each run.

Every proof run MUST execute through a supported sandbox envelope whose profile identity starts with `mantle-proof-sandbox-v1:`. Direct-host execution, missing sandbox evidence, unsupported profile identity, bypassed sandbox execution, or malformed sandbox evidence MUST fail closed.

Dependency substitution MAY be allowed only when substituted dependency identities are declared in the receipt and held constant across all proof runs. The selected proof-unit output itself MUST NOT be satisfied from the default store, the main release-reproduce output, or a previous proof run.

#### Scenario: Proof run roots are distinct and fresh

- GIVEN Mantle plans rebuild A and rebuild B for the same proof unit
- WHEN it allocates proof-store and output-root paths
- THEN every run receives distinct proof-store and output-root identities
- AND any root that already contains the proof-unit output is rejected before the proof run starts

#### Scenario: Main reproduce output cannot satisfy proof runs

- GIVEN `mantle release reproduce` has already produced a main rebuild output
- WHEN deterministic proof comparison runs are requested
- THEN the proof-run sandbox does not bind the main rebuild output or main rebuild store as writable proof inputs
- AND the receipt records separate proof-store and output-root identities for each comparison run

#### Scenario: Unsupported sandbox evidence fails closed

- GIVEN a proof run is recorded as direct-host or with a sandbox profile not starting with `mantle-proof-sandbox-v1:`
- WHEN Mantle validates the proof receipt
- THEN the receipt cannot produce `self-rebuild-match`
- AND the report identifies unsupported sandbox evidence

#### Scenario: Declared dependency substitution is stable

- GIVEN proof runs use substituted dependencies
- WHEN the proof receipt is emitted
- THEN every substituted dependency identity is recorded
- AND the dependency set is identical across all comparison runs
