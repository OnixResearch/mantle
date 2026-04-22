# functional-core Specification

## Purpose
Define the adopted no-std functional-core boundaries for crunch so pure
attestation, project-management, shell-activation, and release-evidence logic
stay in dedicated `#![no_std]` crates while std crates remain thin
translation and effect shells.
## Requirements
### Requirement: Dedicated no-std core crates

The system MUST put selected functional-core logic in dedicated `#![no_std]`
crates that use `alloc` and expose pure transforms over owned data.

ID: functional.core.dedicated.nostd.crates

The first wave MUST continue to cover these concrete scopes:

- `crunch-attestation-core`: logic currently living in
  `crates/crunch-attestation/src/{canonical.rs,digest.rs,error.rs,policy.rs,release.rs,schema.rs,version.rs}`,
  including `evaluate_policy(...)` and `binary_digests_match(...)`
- `crunch-project-core`: logic currently living in
  `crates/crunch-project/src/{manifest.rs,lock.rs,merge.rs,drift.rs,upgrade.rs,version.rs,generate.rs,refresh.rs}`
  after the refresh path is split so core owns `ResolvedInput`,
  `HashResolutionMode`, `RefreshFailure`, `StaleReport`, `RefreshOutcome`,
  `ApplyResult`, `refresh_inputs(...)`, `apply_outcomes(...)`, and
  `list_stale(...)`, while std adapters keep `RefreshResolver`
  implementations and all resolver I/O

The second wave MUST cover these concrete scopes:

- `crunch-shell-core`: shell sidecar JSON validation plus activation
  env/path/hook planning over owned UTF-8 `String`, `Vec<String>`, and
  `BTreeMap<String, String>` data, while `crunch-shell` keeps `PathBuf`,
  `OsString`, `split_paths(...)`, non-UTF-8/path rejection, and
  `ExecTarget` reconstruction around the core call
- `crunch-release-core`: release-evidence manifest canonicalization,
  bundle-member/path validation, proof-linkage validation, and full
  self-hosting proof identity parsing over owned bytes/strings, while
  `src/release_evidence.rs` and `src/release_cmd.rs` keep file copying,
  directory hashing, manifest I/O, proof-bundle loading, digesting, and CLI
  formatting in the std shell

Additional domains MAY follow later, but they MUST use the same core/shell
pattern once adopted.

Compliance with this requirement MUST be proven by
`functional.core.nostd.boundary.continuously.verified`.

#### Scenario: First wave creates dedicated core crates
ID: functional.core.dedicated.nostd.crates.first.wave

- GIVEN the first no-std extraction wave lands
- WHEN the workspace is inspected
- THEN `crunch-attestation-core` and `crunch-project-core` exist as dedicated
  no-std crates
- AND the extracted logic no longer depends on std shell crates for its pure
  transforms

#### Scenario: Second wave adds shell and release core crates
ID: functional.core.dedicated.nostd.crates.second.wave

- GIVEN the second no-std extraction wave lands
- WHEN the workspace is inspected
- THEN `crunch-shell-core` and `crunch-release-core` exist as dedicated no-std
  crates
- AND `crunch-shell` plus the root `crunch` release-evidence path remain std
  shell/adaptor layers around those cores

### Requirement: Core APIs stay on plain data and typed results

Functional-core APIs MUST accept plain owned data and return plans, normalized
values, validation results, or typed errors. They MUST NOT perform effects.

For the adopted first- and second-wave core crates, every public free-function
signature, public inherent method signature, public struct field, public enum
payload, and public type alias in the core crates MUST use only recursively
allowed boundary types:

- crate-local named structs/enums
- scalars: `bool`, `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`
- owned bytes: `Vec<u8>` and `[u8; N]`
- owned containers instantiated only with recursively allowed types:
  `String`, `Box<T>`, `Vec<T>`, `Option<T>`, `Result<T, E>`,
  `BTreeMap<K, V>`, and `BTreeSet<T>`

Those checked boundary surfaces MUST NOT use references, slices, tuples,
`impl Trait`, trait objects, `Rc`, `Arc`, `Cow`, or any
path/process/time/env/os boundary type.

The adopted first- and second-wave core crates MUST NOT define public traits.
Public trait-impl methods are outside this no-references rule. The API-shape
checker MUST ignore trait-required receiver/formatter signatures and instead
inspect only the checked boundary surfaces named above.

ID: functional.core.apis.plain.data.typed.results

Core APIs MUST NOT read files, spawn subprocesses, fetch from the network, read
ambient environment variables, consult the wall clock, or write logs directly.

Compliance with this requirement MUST be proven by
`functional.core.nostd.boundary.continuously.verified`.

#### Scenario: Core handles normalized request without ambient reads
ID: functional.core.apis.plain.data.typed.results.normalized.request.no.ambient.reads

- GIVEN a shell layer has already read files, resolved paths, or downloaded
  bytes
- WHEN it calls a functional-core API
- THEN the API operates only on the supplied request data
- AND the API does not perform additional ambient reads on its own

### Requirement: Shell adapters own effect translation

Std shell/adaptor crates MUST gather effectful host inputs, translate them into
plain core requests, call the no-std core, and then perform any follow-up
writes or user-facing formatting.

ID: functional.core.shell.adapters.effect.translation

Compliance with this requirement MUST be proven by
`functional.core.nostd.boundary.continuously.verified`.

The required shell-adapter tests MUST prove these assertions:

- `cargo test -p crunch-project shell_adapter_keeps_refresh_io_outside_core`
  proves git/file/download resolution and `RefreshResolver` implementations stay
  in the std adapter before the core call, and that the core-facing boundary
  uses only `ProjectManifest`, `Lockfile`, `ResolvedInput`, `RefreshOutcome`,
  and related core types rather than `PathBuf`, `Command`, or other std effect
  types
- `cargo test -p crunch-attestation shell_adapter_keeps_discovery_outside_core`
  proves directory/file discovery happens in the std adapter before the core
  call, and that the core-facing boundary uses parsed attestation values or
  normalized witness inputs rather than `Path`/`PathBuf` handles
- `cargo test -p crunch-shell adapter_preserves_path_order_and_appends_bin`
  proves `crunch-shell` keeps `split_paths(...)`, `/bin` appending for
  `--with`, and `ExecTarget` reconstruction in the std adapter while
  `crunch-shell-core` only sees owned UTF-8 path strings
- `cargo test -p crunch-shell non_utf8_with_path_is_rejected` proves non-UTF-8
  `--with` or `PATH`-derived entries fail in the std adapter before the core
  receives normalized strings
- `cargo test -p crunch --bin crunch create_and_verify_release_bundle_round_trip`
  and `cargo test -p crunch --bin crunch load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact`
  prove `src/release_evidence.rs` keeps proof-bundle loading, file copying,
  directory hashing, and manifest I/O in the std shell while
  `crunch-release-core` only validates or normalizes owned bytes and manifest
  data
- `cargo test -p crunch --test release_cli release_verify_rejects_manifest_schema_mismatch`,
  `cargo test -p crunch --test release_cli release_verify_rejects_missing_workflow_provenance`,
  `cargo test -p crunch --test release_cli release_verify_rejects_claim_boundary_violation`,
  and `cargo test -p crunch --test release_cli release_verify_rejects_proof_linkage_source_digest_mismatch`
  prove malformed release evidence is rejected without pushing file I/O or CLI
  formatting into the no-std core boundary

#### Scenario: Project refresh keeps resolver I/O in shell
ID: functional.core.shell.adapters.effect.translation.project.refresh.io.in.shell

- GIVEN project refresh needs git, local files, or downloaded URL content
- WHEN crunch evaluates refresh state
- THEN the std shell/adaptor layer performs that I/O first
- AND `crunch-project-core` only receives normalized resolver results and plain
  project state

#### Scenario: Attestation verification keeps file discovery in shell
ID: functional.core.shell.adapters.effect.translation.attestation.file.discovery.in.shell

- GIVEN attestation verification needs to scan a witness directory on disk
- WHEN crunch loads that verification material
- THEN the std shell/adaptor layer finds and reads those files
- AND `crunch-attestation-core` only receives plain attestation values and
  verification inputs

#### Scenario: Shell activation keeps path and exec translation in shell
ID: functional.core.shell.adapters.effect.translation.shell.activation.path.translation.in.shell

- GIVEN shell activation needs host PATH splitting, `PathBuf` handling, or
  exec-target reconstruction
- WHEN crunch computes an activation plan
- THEN `crunch-shell` performs those std/path/OS translations first
- AND `crunch-shell-core` only receives owned UTF-8 strings and owned maps

#### Scenario: Release evidence keeps bundle I/O in shell
ID: functional.core.shell.adapters.effect.translation.release.evidence.bundle.io.in.shell

- GIVEN release evidence needs file copying, directory hashing, or proof-bundle
  loading from disk
- WHEN crunch creates or verifies a release-evidence bundle
- THEN `src/release_evidence.rs` performs those filesystem and hashing steps in
  the std shell before or after the core call
- AND `crunch-release-core` only receives manifest bytes or owned manifest data

### Requirement: No-std boundary stays continuously verified

The no-std core boundary MUST stay covered by repeatable validation.

ID: functional.core.nostd.boundary.continuously.verified

Validation for this change MUST run in the repo's rustup-managed toolchain
environment. It MUST first ensure the `wasm32-unknown-unknown` target is
installed via `rustup target add wasm32-unknown-unknown` or fail with a clear
prerequisite error.

Validation MUST then include these exact commands:

- `cargo check -p crunch-attestation-core`
- `cargo check -p crunch-project-core`
- `cargo check -p crunch-shell-core`
- `cargo check -p crunch-release-core`
- `cargo check -p crunch-attestation-core --target wasm32-unknown-unknown`
- `cargo check -p crunch-project-core --target wasm32-unknown-unknown`
- `cargo check -p crunch-shell-core --target wasm32-unknown-unknown`
- `cargo check -p crunch-release-core --target wasm32-unknown-unknown`
- `cargo test -p crunch-attestation-core`
- `cargo test -p crunch-project-core`
- `cargo test -p crunch-shell-core`
- `cargo test -p crunch-release-core`
- `cargo test -p crunch-attestation shell_adapter_keeps_discovery_outside_core`
- `cargo test -p crunch-project shell_adapter_keeps_refresh_io_outside_core`
- `cargo test -p crunch-shell adapter_preserves_path_order_and_appends_bin`
- `cargo test -p crunch-shell non_utf8_with_path_is_rejected`
- `cargo test -p crunch --bin crunch create_and_verify_release_bundle_round_trip`
- `cargo test -p crunch --bin crunch load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact`
- `cargo test -p crunch --test release_cli release_verify_rejects_manifest_schema_mismatch`
- `cargo test -p crunch --test release_cli release_verify_rejects_missing_workflow_provenance`
- `cargo test -p crunch --test release_cli release_verify_rejects_claim_boundary_violation`
- `cargo test -p crunch --test release_cli release_verify_rejects_proof_linkage_source_digest_mismatch`
- `scripts/check-no-std-core-deps.sh`
- `scripts/check-no-std-core-purity.sh`
- `scripts/check-no-std-core-scope.sh`
- `scripts/check-no-std-core-api-shape.sh`
- `scripts/check-no-std-core-ownership.sh`

The dependency-boundary checker MUST operate as an allowlist, not a denylist.
It MUST fail if the dependency closure of any adopted first- or second-wave
core crate includes any crate not named in the checked-in allowlist
`openspec/specs/functional-core/validation/deps-allowlist.txt`.

The checker MUST also verify enabled features with `cargo tree -e features` or
an equivalent metadata source. It MUST fail if an allowlisted crate enables a
`std` feature or a default-feature set that requires `std`.

The backing adopted-core inventory that drives
`scripts/check-no-std-core-scope.sh`, `scripts/check-no-std-core-api-shape.sh`,
and `scripts/check-no-std-core-ownership.sh` MUST live as one checked-in source
under `openspec/specs/functional-core/validation/`. That inventory MUST
enumerate every adopted first- and second-wave core crate, its required
exports, and its std adapter files. `scripts/no_std_core_checks.py` MAY cache
parsed inventory data during one run, but it MUST fail when `crunch-shell-core`,
`crunch-release-core`, or their named exported surfaces are absent from that
checked-in backing inventory.

The purity checker MUST fail if adopted first- or second-wave core source uses
any banned direct ambient-effect or non-determinism pattern, including these
minimum patterns:

- `std::`
- `println!`
- `eprintln!`
- `dbg!`
- `log::`
- `tracing::`
- `rand::`
- `getrandom::`
- `fastrand::`
- `thread_rng`
- `OnceLock`
- `LazyLock`
- `lazy_static!`
- `thread_local!`
- `static mut`

The scope checker MUST fail unless all of these statements are true:

- `crates/crunch-attestation-core/src/lib.rs`,
  `crates/crunch-project-core/src/lib.rs`,
  `crates/crunch-shell-core/src/lib.rs`, and
  `crates/crunch-release-core/src/lib.rs` each declare `#![no_std]` and
  `extern crate alloc`
- the default feature set of each adopted first- or second-wave core crate
  remains no-std
- none of the adopted first- or second-wave core crates defines a `std`
  feature at all
- `crunch-attestation-core` exports the first-wave attestation modules and
  functions named in `functional.core.dedicated.nostd.crates`
- `crunch-project-core` exports the first-wave project modules and functions
  named in `functional.core.dedicated.nostd.crates`
- `crunch-shell-core` exports `ShellError`, `ShellSidecar`, `HostEnv`,
  `ShellWarning`, `ActivationPlan`, and `compute_activation(...)`
- `crunch-release-core` exports `ReleaseEvidenceError`,
  `ReleaseEvidenceManifest`, `BundledArtifact`, `BundledArtifactKind`,
  `ReleaseWorkflowIdentity`, `ReleaseProofLinkage`,
  `FullSelfHostingProofIdentityFields`, and
  `extract_full_self_hosting_proof_identity_fields(...)`

The API-shape checker MUST operate as an allowlist over the checked boundary
surfaces defined by `functional.core.apis.plain.data.typed.results`. It MUST
recursively inspect generic arguments and fail if any checked boundary surface
in the adopted first- or second-wave core crates exposes a type outside that
allowed boundary set. It MUST also fail if any adopted first- or second-wave
core crate defines any `pub trait`.

The ownership checker MUST continue to deterministically inspect the first-wave
legacy std source paths:

- `crates/crunch-attestation/src/{canonical.rs,digest.rs,error.rs,policy.rs,release.rs,schema.rs,version.rs}`
- `crates/crunch-project/src/{manifest.rs,lock.rs,merge.rs,drift.rs,upgrade.rs,version.rs,generate.rs,refresh.rs}`

It MUST fail unless the first-wave module/function scope listed in
`functional.core.dedicated.nostd.crates` is implemented by the new core crates
and those first-wave legacy std paths are either removed or reduced to allowed
adapter forms.

First-wave legacy std paths MAY keep only these deterministic adapter forms:

- comments and blank lines
- `use` imports
- `pub use` re-exports
- `type` aliases

The ownership checker MUST fail if any first-wave legacy std path contains code
outside those allowed forms, including these minimum forbidden patterns:

- `fn `
- `impl `
- `struct `
- `enum `
- loops: `for `, `while `, `loop `
- `match `
- hashing calls: `blake3`, `sha1`, `sha2`, `md5`, `digest::`

Because semantic duplicate-logic detection outside the first-wave legacy paths
is not fully automatable, final validation MUST also update the checked-in
review artifact `openspec/specs/functional-core/evidence/ownership-review.md`.
That artifact MUST still list every touched std workspace source file outside
those first-wave legacy paths derived from the union of git history for the
active path `openspec/changes/no-std-functional-core/` and the archived path
`openspec/changes/archive/*-no-std-functional-core/`, when present, through
`HEAD`.

That artifact MUST also explicitly list these second-wave std adapter files even
if the current implementation change does not edit them:

- `crates/crunch-shell/src/lib.rs`
- `crates/crunch-shell/src/adapter.rs`
- `crates/crunch-shell/src/types.rs`
- `src/release_evidence.rs`
- `src/release_cmd.rs`

The ownership review artifact MUST classify each listed second-wave std adapter
file as `adapter-only` or `unrelated`, and it MUST record a review verdict that
shell/release business logic remains in `crunch-shell-core` and
`crunch-release-core` rather than drifting back into those std files.

#### Scenario: Regression introduces std leak
ID: functional.core.nostd.boundary.continuously.verified.regression.introduces.std.leak

- GIVEN a later edit adds a std-only dependency or ambient effect to an
  adopted no-std core crate
- WHEN the required no-std target checks, core tests, and dependency checker
  run
- THEN no-std target compilation or dependency-boundary validation fails
- AND the regression is caught before the boundary claim is trusted

