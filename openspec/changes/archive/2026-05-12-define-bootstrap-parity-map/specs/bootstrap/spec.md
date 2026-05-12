## ADDED Requirements

### Requirement: Whole-bootstrap parity map

Crunch MUST maintain a canonical whole-bootstrap parity map that covers the complete bootstrap ladder required to claim parity with the reference source-bootstrap ecosystems being used as inputs: live-bootstrap for the concrete seed-to-modern-toolchain stage order, Guix for full-source bootstrap claim semantics and trust-root disclosure, and StageX for the no-quorum audited-seed lineage profile.
ID: bootstrap.parity.map

The map MUST enumerate every planned or implemented bootstrap stage from the audited seed through the normalized seed provider and final self-build proof, including stage0/hex0 material, M0/M1/hex2/kaem-style transition tools, Mes, TinyCC, musl, make, patch, grep, sed, bzip2, gzip, tar, coreutils, diffutils, gawk, bison, flex, m4, libtool, autoconf/automake versions, Perl versions, GMP, MPFR, MPC, binutils generations, GCC 4.0, GCC 4.7, GCC 10, full musl/binutils handoff, seed-full, selftest, integration-test, and the Crunch self-build stages. Each entry MUST name the reference source lineage (`live-bootstrap`, `guix`, `stagex`, or a documented Crunch-specific bridge), source artifact identity, patch set, provider inputs and outputs, implementation derivation, placeholder status, runtime-smoke evidence, proof transcript path or digest, and known deviations.

#### Scenario: Parity map names all reference axes

- GIVEN the bootstrap parity map exists
- WHEN it is validated
- THEN it lists live-bootstrap, Guix full-source bootstrap, and StageX no-quorum axes separately
- AND each axis has explicit completion criteria and evidence fields

#### Scenario: Missing mapped stage blocks parity claim

- GIVEN any mapped stage lacks an implementation derivation, replacement rationale, or proof transcript
- WHEN bootstrap parity is reported
- THEN parity status is `incomplete`
- AND the report names the missing stage and reference axis

#### Scenario: Placeholder stage blocks parity claim

- GIVEN a mapped stage still emits placeholder, bridge-only, or stub output
- WHEN bootstrap parity is reported
- THEN the stage is counted as incomplete unless the map records an accepted replacement rationale and equivalent proof evidence
- AND full live-bootstrap/Guix/StageX parity claims remain disabled

### Requirement: Bootstrap parity gap report

Crunch MUST produce a deterministic bootstrap parity gap report from the canonical map and current repository state.
ID: bootstrap.parity.gap-report

The report MUST classify each stage as `complete`, `partial`, `placeholder`, `blocked`, `out-of-scope-replaced`, or `not-started`. It MUST distinguish graph-completion evidence from semantic correctness evidence, name runtime smokes and host-linked semantic smokes separately, and record whether evidence was produced by the legacy fetched provider, source-root provider, or StageX-class lineage provider. It MUST preserve the current distinction that the GCC 4.0 pass1 bridge and bounded libgcc semantic members are progress toward native correctness, not full native GCC correctness.

#### Scenario: Report separates graph completion from correctness

- GIVEN GCC 4.0 builds through a late generator/executable graph using stubs or bridge behavior
- AND only selected libgcc members have semantic implementations
- WHEN the parity gap report is generated
- THEN the graph-completion row is marked separately from native compiler correctness rows
- AND the report does not claim full GCC correctness

#### Scenario: Provider kind is visible in every evidence row

- GIVEN a stage has runtime-smoke evidence
- WHEN the parity gap report is generated
- THEN the row records whether the evidence came from `legacy-fetch`, `source-root`, or `stagex-lineage`
- AND StageX parity is incomplete unless the row is backed by StageX-class lineage evidence where required

### Requirement: Bootstrap parity claim gating

Crunch MUST fail closed on any operator-facing claim that Crunch has reached full live-bootstrap, Guix full-source bootstrap, or StageX no-quorum parity unless the parity map and gap report show every required stage complete with the required provider/proof evidence.
ID: bootstrap.parity.claim-gating

A parity claim MUST be scoped to the exact axis satisfied. Live-bootstrap parity MUST require the complete mapped stage ladder or accepted Crunch-specific replacements. Guix full-source parity MUST require source-built inputs, trust-root documentation, and final source proof comparable to Guix's full-source bootstrap claim semantics. StageX parity MUST require the audited hex0 seed lineage, no prebuilt compiler/tool root, protected execution audit when used, and `stagex-lineage` self-build proof metadata. Partial drains, archived scaffolds, placeholder derivations, seed-assisted legacy fallback, and broad documentation links MUST NOT satisfy these claims.

#### Scenario: Axis-specific claim succeeds only for completed axis

- GIVEN live-bootstrap parity is complete
- BUT Guix trust-root disclosure or StageX lineage proof remains incomplete
- WHEN release evidence is generated
- THEN the release may claim only the completed live-bootstrap-scoped parity
- AND it must explicitly list Guix and StageX gaps

#### Scenario: Legacy fallback blocks no-quorum parity

- GIVEN any required parity evidence row was produced by the legacy fetched provider
- WHEN a StageX no-quorum claim is requested
- THEN the claim fails closed
- AND the diagnostic names the legacy-backed rows that must be rebuilt through `stagex-lineage`
