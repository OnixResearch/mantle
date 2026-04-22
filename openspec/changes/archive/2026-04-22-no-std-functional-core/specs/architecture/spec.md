## ADDED Requirements

### Requirement: No-std core crates are a first-class workspace tier

The workspace MUST distinguish dedicated no-std core crates from std
shell/adaptor crates in the first extraction wave.

ID: architecture.nostd.core.workspace.tier

The first no-std extraction wave MUST add these crates and roles:

| Crate | Role |
|---|---|
| `crunch-attestation-core` | no-std ownership of logic currently implemented in `crates/crunch-attestation/src/{canonical.rs,digest.rs,error.rs,policy.rs,release.rs,schema.rs,version.rs}`, including `evaluate_policy(...)` and `binary_digests_match(...)` |
| `crunch-project-core` | no-std ownership of logic currently implemented in `crates/crunch-project/src/{manifest.rs,lock.rs,merge.rs,drift.rs,upgrade.rs,version.rs,generate.rs,refresh.rs}` after the refresh path is split so core owns `ResolvedInput`, `HashResolutionMode`, `RefreshFailure`, `StaleReport`, `RefreshOutcome`, `ApplyResult`, `refresh_inputs(...)`, `apply_outcomes(...)`, and `list_stale(...)` |
| `crunch-attestation` | std adaptor/re-export layer that retains `crates/crunch-attestation/src/discovery.rs`, file discovery, and release-bundle loading before calling core logic |
| `crunch-project` | std adaptor/re-export layer that retains `RefreshResolver` implementations, resolver I/O, tempdirs, file reads/writes, and shell-facing project APIs around the moved project-core logic |

#### Scenario: Workspace shows explicit no-std core tier
ID: architecture.nostd.core.workspace.tier.visible

- GIVEN the workspace manifest and crate directories
- WHEN the project layout is inspected after this change lands
- THEN the workspace contains `crunch-attestation-core` and
  `crunch-project-core` as dedicated no-std crates
- AND the existing std-facing `crunch-attestation` and `crunch-project` crates
  remain present as shell/adaptor layers

### Requirement: First-wave core crates use crate boundaries for shell separation

The first-wave functional core MUST rely on crate boundaries, not convention
alone, to keep moved business logic separate from imperative shell code.

ID: architecture.nostd.core.crate.boundary

Code that performs filesystem access, subprocess execution, environment reads,
clock access, network I/O, or log writing MUST live outside the first-wave
no-std core crates. Core crates MUST expose deterministic transforms over plain
data and typed errors only.

Compliance with this requirement MUST be proven by
`functional.core.nostd.boundary.continuously.verified`.

For the first wave, deterministic means no ambient randomness and no hidden
mutable global state inside the no-std core crates.

#### Scenario: Effectful dependency stays out of first-wave core crates
ID: architecture.nostd.core.crate.boundary.effectful.dependency.outside

- GIVEN a future change that needs git subprocesses or HTTP downloads for the
  attestation or project-management paths
- WHEN that code is added to crunch
- THEN it lands in a std shell/adaptor crate
- AND the no-std core crate interface accepts only normalized request data and
  returns typed results
