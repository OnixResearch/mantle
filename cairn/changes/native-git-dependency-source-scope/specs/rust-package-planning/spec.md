# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native git dependency source scope

r[rust_package_planning.native_git_dependency_source_scope] Mantle MUST represent supported locked git dependency sources as explicit native source facts before those packages participate in Cargo-free native package facts, unit graphs, or topology execution.

#### Scenario: Locked git dependency source facts are derived from captured source closure

GIVEN a Cargo lockfile contains a git package entry with a URL and resolved revision
AND the captured Rust source closure contains the same package identity with a readable manifest path and source root
WHEN Mantle computes native Rust source-planning facts for the current invocation
THEN Mantle MUST record a native git source fact containing package name, version, package identity, git URL, resolved revision, lockfile digest, manifest path, source root, and BLAKE3 source-tree digest
AND the source fact MUST identify that it is bounded to captured source-closure material, not a network fetch or version-solving result.

#### Scenario: Git dependency edges resolve only through ready git source facts

GIVEN a native package has a selected dependency whose Cargo lockfile source is git-backed
AND native git source planning has a ready fact for that dependency package identity
WHEN Mantle computes native package dependency facts
THEN Mantle MUST resolve the dependency edge to the git source fact instead of emitting `unsupported-non-path-dependency`
AND the resolved dependency MUST be eligible for downstream native unit graph and topology planning under the same explicit-source rules as supported registry/path dependencies.

#### Scenario: Git source scope fails closed for undeclared or ambiguous material

GIVEN a selected git dependency lacks captured source-closure material, has an unreadable manifest path, has no resolved revision, has multiple ambiguous package roots for one package identity, or would require network or `$CARGO_HOME` discovery outside the captured source closure
WHEN Mantle computes native source or package dependency facts
THEN Mantle MUST emit a deterministic native git source blocker before rustc execution
AND Mantle MUST NOT claim native package-target, unit graph, host-unit graph, or topology execution readiness for packages depending on that unresolved git source.

#### Scenario: Git source facts are receipt-bound and auditable

GIVEN Mantle emits `rust-plan` JSON evidence for a workspace with supported locked git dependency sources
WHEN native git source planning finishes
THEN the receipt MUST include ready status, ordered git source facts, lockfile/source digests, oracle/source-closure comparison evidence, blockers when present, and a stable receipt hash
AND the receipt MUST keep the bounded claim explicit: locked git packages with captured local source-closure material only, with no general Cargo git compatibility claim.

#### Scenario: Native git source scope covers the self-probe blocker

GIVEN Mantle's self `rust-plan --execute-topology` probe currently reports `snix-castore` missing native package facts because dependency `wu-manber` is outside the bounded path-or-declared-registry fragment
WHEN this change is implemented for locked git source facts
THEN focused verification MUST show that `wu-manber` no longer produces `unsupported-non-path-dependency` solely because it is git-backed
AND remaining blockers, if any, MUST be recorded as new deterministic classes with baseline/current evidence.
