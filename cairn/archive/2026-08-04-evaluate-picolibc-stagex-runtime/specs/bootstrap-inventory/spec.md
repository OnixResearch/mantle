## ADDED Requirements

### Requirement: Picolibc StageX comparison remains diagnostic and evidence-gated

r[bootstrap_inventory.picolibc_stagex_comparison] Mantle MUST classify a Picolibc StageX comparison from deterministic evidence. The comparison MUST NOT change provider or parity authority.

#### Scenario: source and tool authority is complete

GIVEN a pinned Picolibc release is selected for comparison
WHEN Mantle plans the x86_64 Linux static diagnostic
THEN the plan MUST bind source URLs, release identity, fixed-output hash, source BLAKE3, configuration, and license manifests
AND it MUST bind Meson, Ninja, compiler, linker, archiver, and generated-input BLAKE3 identities
AND realization MUST deny live network access and undeclared tool discovery.

#### Scenario: diagnostic construction stays outside StageX authority

GIVEN the complete source and tool authority is available
WHEN Mantle builds the Picolibc x86_64 Linux static profile
THEN the output MUST record compiled sources, generated files, local source rewrites, tool roles, startup objects, libraries, and output bytes
AND the build MUST reject host-libc linkage, ambient startup libraries, missing tools, source tamper, or output outside the declared diagnostic root
AND Meson and Ninja observations MUST remain diagnostic-only and MUST NOT satisfy protected StageX execution evidence.

#### Scenario: behavior evidence covers positive and negative paths

GIVEN a Picolibc diagnostic output was built
WHEN Mantle compares it with the current native-musl StageX baseline
THEN Picolibc MUST pass the shared positive libc behavior contract and malformed-source rejection contract
AND missing behavior, crash, timeout, unexpected success, host-libc dependence, or undeclared fallback MUST prevent a `candidate` outcome.

#### Scenario: isolated builds produce stable identities

GIVEN two builds use the same declared source and tool authority
WHEN Mantle runs them with fresh state, output, and scratch roots
THEN every compared runtime artifact and normalized report MUST have equal BLAKE3 identities
AND absolute scratch paths, timestamps, ambient state, or generated-file drift MUST produce `rejected` or `blocked` evidence.

#### Scenario: pure comparison logic selects one bounded outcome

GIVEN complete Picolibc and native-musl comparison facts
WHEN the pure comparison core evaluates the facts
THEN it MUST emit exactly one outcome from `candidate`, `rejected`, or `blocked` with deterministic reasons
AND `candidate` MUST require fewer compiled translation units, fewer source-rewrite operations, complete behavior parity, stable identities, and no new protected executable roles
AND proven behavior failure, nondeterminism, host-libc dependence, or no surface reduction MUST produce `rejected`
AND missing or inconclusive source, tool, license, behavior, identity, baseline, or protected-route evidence MUST produce `blocked`.

#### Scenario: comparison outcome cannot promote a provider

GIVEN the comparison emits any outcome
WHEN Mantle records the report, ADR, oracle checkpoint, or lifecycle evidence
THEN provider selection, StageX lineage, bootstrap parity, accepted provider digests, and release status MUST remain unchanged
AND a `candidate` outcome MUST authorize only a later Cairn change with separate construction and admission evidence.

#### Scenario: comparison claims remain bounded

GIVEN the diagnostic and comparison report are complete
WHEN operators or maintainers cite the result
THEN they MUST identify the Picolibc source, tools, configuration, licenses, baseline, behavior, artifacts, outcome, and evidence location
AND they MUST NOT claim final musl replacement, protected StageX admission, compiler correctness, libc correctness, kernel correctness, full-bootstrap completion, or release eligibility.
