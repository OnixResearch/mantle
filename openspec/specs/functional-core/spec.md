# functional-core Specification

## Purpose
TBD - created by archiving change no-std-functional-core. Update Purpose after archive.
## Requirements
### Requirement: Dedicated no-std core crates

The system MUST put selected functional-core logic in dedicated `#![no_std]`
crates that use `alloc` and expose pure transforms over owned data.

ID: functional.core.dedicated.nostd.crates

The first wave MUST cover these concrete scopes:

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

### Requirement: Core APIs stay on plain data and typed results

Functional-core APIs MUST accept plain owned data and return plans, normalized
values, validation results, or typed errors. They MUST NOT perform effects.

For the first wave, every public free-function signature, public inherent
method signature, public struct field, public enum payload, and public type
alias in the core crates MUST use only recursively allowed boundary types:

- crate-local named structs/enums
- scalars: `bool`, `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`
- owned bytes: `Vec<u8>` and `[u8; N]`
- owned containers instantiated only with recursively allowed types:
  `String`, `Box<T>`, `Vec<T>`, `Option<T>`, `Result<T, E>`,
  `BTreeMap<K, V>`, and `BTreeSet<T>`

Those checked boundary surfaces MUST NOT use references, slices, tuples,
`impl Trait`, trait objects, `Rc`, `Arc`, `Cow`, or any
path/process/time/env/os boundary type.

First-wave core crates MUST NOT define public traits. Public trait-impl methods
are outside this no-references rule. The API-shape checker MUST ignore
trait-required receiver/formatter signatures and instead inspect only the
checked boundary surfaces named above.

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
- `cargo check -p crunch-attestation-core --target wasm32-unknown-unknown`
- `cargo check -p crunch-project-core --target wasm32-unknown-unknown`
- `cargo test -p crunch-attestation-core`
- `cargo test -p crunch-project-core`
- `cargo test -p crunch-attestation shell_adapter_keeps_discovery_outside_core`
- `cargo test -p crunch-project shell_adapter_keeps_refresh_io_outside_core`
- `scripts/check-no-std-core-deps.sh`
- `scripts/check-no-std-core-purity.sh`
- `scripts/check-no-std-core-scope.sh`
- `scripts/check-no-std-core-api-shape.sh`
- `scripts/check-no-std-core-ownership.sh`

The dependency-boundary checker MUST operate as an allowlist, not a denylist.
It MUST fail if the dependency closure of either first-wave core crate includes
any crate not named in the checked-in allowlist
`openspec/changes/no-std-functional-core/validation/deps-allowlist.txt`.

The checker MUST also verify enabled features with `cargo tree -e features` or
an equivalent metadata source. It MUST fail if an allowlisted crate enables a
`std` feature or a default-feature set that requires `std`.

The purity checker MUST fail if first-wave core source uses any banned direct
ambient-effect or non-determinism pattern, including these minimum patterns:

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

- `crates/crunch-attestation-core/src/lib.rs` and
  `crates/crunch-project-core/src/lib.rs` each declare `#![no_std]` and
  `extern crate alloc`
- the default feature set of each first-wave core crate remains no-std
- neither first-wave core crate defines a `std` feature at all
- `crunch-attestation-core` exports the first-wave attestation modules and
  functions named in `functional.core.dedicated.nostd.crates`
- `crunch-project-core` exports the first-wave project modules and functions
  named in `functional.core.dedicated.nostd.crates`

The API-shape checker MUST operate as an allowlist over the checked boundary
surfaces defined by `functional.core.apis.plain.data.typed.results`. It MUST
recursively inspect generic arguments and fail if any checked boundary surface
in the first-wave core crates exposes a type outside that allowed boundary set.
It MUST also fail if either first-wave core crate defines any `pub trait`.

The ownership checker MUST deterministically inspect these legacy std source
paths:

- `crates/crunch-attestation/src/{canonical.rs,digest.rs,error.rs,policy.rs,release.rs,schema.rs,version.rs}`
- `crates/crunch-project/src/{manifest.rs,lock.rs,merge.rs,drift.rs,upgrade.rs,version.rs,generate.rs,refresh.rs}`

It MUST fail unless the first-wave module/function scope listed in
`functional.core.dedicated.nostd.crates` is implemented by the new core crates
and those legacy std paths are either removed or reduced to allowed adapter
forms.

Legacy std paths MAY keep only these deterministic adapter forms:

- comments and blank lines
- `use` imports
- `pub use` re-exports
- `type` aliases

The ownership checker MUST fail if any legacy std path contains code outside
those allowed forms, including these minimum forbidden patterns:

- `fn `
- `impl `
- `struct `
- `enum `
- loops: `for `, `while `, `loop `
- `match `
- hashing calls: `blake3`, `sha1`, `sha2`, `md5`, `digest::`

Because semantic duplicate-logic detection outside the legacy paths is not
fully automatable, final validation MUST also update the checked-in review
artifact `openspec/changes/no-std-functional-core/evidence/ownership-review.md`.
That artifact MUST list every touched std workspace source file outside the
legacy paths, classify each one as `adapter-only` or `unrelated`, and record a
review verdict that no first-wave business logic was reintroduced there.

#### Scenario: Regression introduces std leak
ID: functional.core.nostd.boundary.continuously.verified.regression.introduces.std.leak

- GIVEN a later edit adds a std-only dependency or ambient effect to a no-std
  core crate
- WHEN the required no-std target checks, core tests, and dependency checker
  run
- THEN no-std target compilation or dependency-boundary validation fails
- AND the regression is caught before the boundary claim is trusted

