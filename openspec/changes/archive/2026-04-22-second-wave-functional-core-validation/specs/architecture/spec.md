## MODIFIED Requirements

### Requirement: No-std core crates are a first-class workspace tier

The workspace MUST distinguish dedicated no-std core crates from std
shell/adaptor crates across the adopted no-std waves.

ID: architecture.nostd.core.workspace.tier

The adopted no-std waves MUST contain these crates and roles:

| Crate | Role |
|---|---|
| `crunch-attestation-core` | no-std ownership of logic currently implemented in `crates/crunch-attestation/src/{canonical.rs,digest.rs,error.rs,policy.rs,release.rs,schema.rs,version.rs}`, including `evaluate_policy(...)` and `binary_digests_match(...)` |
| `crunch-project-core` | no-std ownership of logic currently implemented in `crates/crunch-project/src/{manifest.rs,lock.rs,merge.rs,drift.rs,upgrade.rs,version.rs,generate.rs,refresh.rs}` after the refresh path is split so core owns `ResolvedInput`, `HashResolutionMode`, `RefreshFailure`, `StaleReport`, `RefreshOutcome`, `ApplyResult`, `refresh_inputs(...)`, `apply_outcomes(...)`, and `list_stale(...)` |
| `crunch-shell-core` | no-std ownership of shell sidecar JSON validation plus activation env/path/hook planning over owned UTF-8 data |
| `crunch-release-core` | no-std ownership of release-evidence manifest canonicalization, bundle-member/path validation, proof-linkage validation, and full self-hosting proof identity parsing over owned bytes/strings |
| `crunch-attestation` | std adaptor/re-export layer that retains `crates/crunch-attestation/src/discovery.rs`, file discovery, and release-bundle loading before calling core logic |
| `crunch-project` | std adaptor/re-export layer that retains `RefreshResolver` implementations, resolver I/O, tempdirs, file reads/writes, and shell-facing project APIs around the moved project-core logic |
| `crunch-shell` | std adaptor layer that retains `PathBuf`, `OsString`, `split_paths(...)`, non-UTF-8 rejection, and `ExecTarget` reconstruction around `crunch-shell-core` |
| `crunch` | std shell for release evidence that retains `src/release_evidence.rs`, `src/release_cmd.rs`, file copying, directory hashing, proof-bundle loading, manifest I/O, and CLI formatting around `crunch-release-core` |

#### Scenario: Workspace shows explicit no-std core tier
ID: architecture.nostd.core.workspace.tier.visible

- GIVEN the workspace manifest and crate directories
- WHEN the project layout is inspected after this change lands
- THEN the workspace contains `crunch-attestation-core`,
  `crunch-project-core`, `crunch-shell-core`, and `crunch-release-core` as
  dedicated no-std crates
- AND the existing std-facing `crunch-attestation`, `crunch-project`,
  `crunch-shell`, and root `crunch` release-evidence path remain present as
  shell/adaptor layers

### Requirement: Adopted no-std core crates use crate boundaries for shell separation

The adopted first- and second-wave functional core MUST rely on crate
boundaries, not convention alone, to keep moved business logic separate from
imperative shell code.

ID: architecture.nostd.core.crate.boundary

Code that performs filesystem access, subprocess execution, environment reads,
clock access, network I/O, or log writing MUST live outside the adopted
no-std core crates. Core crates MUST expose deterministic transforms over plain
data and typed errors only.

Compliance with this requirement MUST be proven by
`functional.core.nostd.boundary.continuously.verified`.

For the adopted waves, deterministic means no ambient randomness and no hidden
mutable global state inside the no-std core crates.

#### Scenario: Effectful dependency stays out of first-wave core crates
ID: architecture.nostd.core.crate.boundary.effectful.dependency.outside

- GIVEN a future change that needs git subprocesses, HTTP downloads, PATH
  splitting, shell-target reconstruction, proof-bundle loading, or release
  bundle hashing for one of the adopted no-std domains
- WHEN that code is added to crunch
- THEN it lands in a std shell/adaptor crate or std root module
- AND the no-std core crate interface accepts only normalized request data and
  returns typed results
