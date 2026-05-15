## ADDED Requirements

### Requirement: Deterministic build proof receipt

The build pipeline MUST support a canonical deterministic-build proof receipt
for a single derivation output set. The receipt MUST record the derivation
identity, selected hermeticity mode, workflow version, toolchain/provider
identity, logical store prefix, physical store isolation strategy, normalized
execution envelope, ambient host perturbation matrix, typed hermeticity audit
events, per-run output store paths, and per-output canonical BLAKE3 NAR digests.

The receipt MUST have a closed verdict enum with at least `not-attempted`,
`deterministic-match`, `mismatch`, `missing-evidence`, `impure-mode`, and
`unsupported-workflow`. Only `deterministic-match` permits a deterministic build
claim.

#### Scenario: Canonical receipt bytes are stable

- GIVEN the same deterministic-build proof facts in any insertion order
- WHEN Mantle serializes the proof receipt
- THEN the canonical bytes are identical
- AND the receipt digest is identical

#### Scenario: Deterministic claim requires matching BLAKE3 digest sets

- GIVEN two or more clean build runs for the same derivation identity
- AND each run completed in strict hermetic mode without proof-blocking audit
  events
- WHEN every required output has the same canonical BLAKE3 NAR digest in every
  run
- THEN the proof receipt verdict is `deterministic-match`

#### Scenario: Digest drift fails the proof

- GIVEN repeated clean build runs for the same derivation identity
- WHEN any required output has a different canonical BLAKE3 NAR digest between
  runs
- THEN the proof receipt verdict is `mismatch`
- AND Mantle MUST NOT claim the derivation is deterministic

### Requirement: Determinism proof runs use isolated clean stores

A deterministic-build proof attempt MUST run each comparison build in a clean
store namespace or fresh physical store directory that cannot reuse prior output
artifacts for the derivation under test. Dependency substitution MAY be allowed
only when the substituted dependency identities are declared in the receipt and
are held constant across all runs.

#### Scenario: Prior output reuse is rejected

- GIVEN a requested deterministic proof for derivation A
- AND A's output already exists in the default store
- WHEN Mantle schedules proof comparison runs
- THEN it uses fresh proof stores or namespaces for A
- AND it does not satisfy the proof by reusing the existing output for A

#### Scenario: Declared dependency substitution is stable

- GIVEN proof runs use substituted dependencies
- WHEN the proof receipt is emitted
- THEN every substituted dependency identity is recorded
- AND the dependency set is identical across all comparison runs

### Requirement: Determinism proof perturbs ambient host state

The deterministic-build proof harness MUST perturb ambient host state across
comparison runs to catch accidental host leakage. The first required matrix MUST
vary at least `HOME`, `PATH`, `USER`, `LOGNAME`, `TZ`, `LANG`, `LC_ALL`, temp
directories, current working directory, umask, and host process environment
noise while preserving the canonical sandbox envelope.

#### Scenario: Host perturbation does not affect strict build output

- GIVEN a derivation eligible for deterministic proof
- WHEN Mantle reruns it under the required ambient host perturbation matrix
- THEN successful runs produce the same BLAKE3 output digest set
- AND the proof receipt records the perturbation cases used

#### Scenario: Host leakage is proof-blocking

- GIVEN a proof run records a hermeticity audit event that indicates host state
  leaked into the build
- WHEN the proof receipt is finalized
- THEN the verdict is not `deterministic-match`
- AND the receipt names the blocking event kind

### Requirement: Deterministic proof requires strict hermetic mode

A deterministic-build proof attempt MUST run in strict hermetic mode. If the
operator selects practical or impure execution, Mantle MUST emit a receipt with
`impure-mode` or `missing-evidence` rather than promoting the result to a
deterministic claim.

#### Scenario: Practical build cannot be promoted silently

- GIVEN a derivation was built successfully in practical mode
- WHEN an operator asks whether it is deterministically proven
- THEN Mantle reports that deterministic proof evidence is missing
- AND it does not infer determinism from the successful practical build

#### Scenario: Impure build blocks determinism proof

- GIVEN a derivation run used impure mode
- WHEN deterministic proof classification is requested
- THEN the proof receipt verdict is `impure-mode`
- AND no deterministic claim is emitted
