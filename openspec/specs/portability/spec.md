# Portability Specification

## Purpose

Defines how crunch avoids platform lock-in. The core (evaluation, derivation
construction, store) MUST be OS-agnostic. Platform-specific code (sandbox,
filesystem) MUST be behind trait abstractions so new platforms can be added
without modifying the core.
## Requirements
### Requirement: Layered platform abstraction

The system MUST stay structured in three layers:

1. **Core (OS-agnostic):** Nickel evaluation, derivation construction,
   project-manifest logic, store path computation, and other pure data/model
   code. This layer MUST avoid platform-specific execution logic.
2. **Platform traits:** abstract interfaces for build execution, filesystem
   access, and store backends. These interfaces describe capabilities, not OS
   primitives.
3. **Platform implementations:** concrete implementations behind those traits.
   These are the only layers allowed to depend on platform-specific behavior.

For the current implementation, build execution is only shipped on Linux.
Evaluation, manifest handling, and other pure logic remain portable Rust code.

```
crunch-eval          (OS-agnostic — Nickel evaluation)
crunch-glue          (OS-agnostic — derivation conversion)
crunch-project       (OS-agnostic — manifest + lockfile logic)
nix-compat           (OS-agnostic — store path computation)
snix-castore         (mostly OS-agnostic — blob/dir storage)
snix-store           (mostly OS-agnostic — pathinfo + NAR)
snix-build           (platform-specific build execution behind BuildService)
  └── bwrap          (shipped on Linux today)
```

#### Scenario: Non-Linux host keeps portable core but not build execution

- GIVEN crunch compiled on a non-Linux host
- WHEN a user evaluates Nickel or runs project-management commands
- THEN the pure core behavior still works
- AND a build attempt is handled by the platform-specific build boundary

### Requirement: Configurable store prefix

The store path prefix MUST be configurable. The default SHOULD be
`/crunch/store` (not `/nix/store`) to avoid conflicts with existing Nix
installations and to work on platforms where `/nix` is awkward.

The prefix MUST be set at crunch initialization time and embedded in all
store path computations. Changing the prefix invalidates all existing
store paths (they are derived from the prefix).

In vendored nix-compat, the hardcoded `STORE_DIR: &str = "/nix/store"`
MUST be replaced with a configurable value.

#### Scenario: Custom store prefix

- GIVEN `crunch --store /opt/crunch/store build hello.ncl`
- WHEN the derivation's output path is computed
- THEN it starts with `/opt/crunch/store/` instead of `/nix/store/`

#### Scenario: Default prefix

- GIVEN no `--store` flag
- WHEN store paths are computed
- THEN they use the default prefix

#### Scenario: Seed paths with different prefix

- GIVEN seed paths from a Nix store at `/nix/store/...-bash-5.2`
- WHEN referenced as `Input::Source` with crunch using `/crunch/store`
- THEN the seed paths are used as-is (their prefix is not rewritten).
  The sandbox mounts them at their original location.

### Requirement: No hardcoded Unix paths in core

The core crates (crunch-eval, crunch-glue, nix-compat) MUST NOT contain
hardcoded references to Unix-specific paths (`/dev`, `/proc`, `/tmp`,
`/bin/sh`). These belong in the platform sandbox implementations.

The `SandboxSpec` in snix-build already abstracts this correctly — it
specifies what to mount, not how. Platform implementations translate
the spec to OS-specific mounts.

#### Scenario: No /proc on Redox

- GIVEN a Redox OS sandbox implementation that provides `scheme:`-based
  isolation instead of `/proc` and `/dev`
- WHEN a build runs
- THEN the sandbox implementation translates `SandboxSpec` to Redox
  schemes; the core is unaware of the difference

### Requirement: Native sandbox as default, Linux-only build support in v0

v0 build execution MUST use the native Linux bubblewrap path.

On non-Linux hosts, `crunch build` and other build-entry commands MUST fail with
an explicit error that building is only supported on Linux and requires bwrap.

#### Scenario: Non-Linux build fails clearly

- GIVEN a non-Linux host
- WHEN `crunch build hello.ncl` is attempted
- THEN crunch returns a clear error explaining that building is only supported on Linux and requires bwrap

### Requirement: Future backends stay labeled as future work

WASM, OCI, and remote builders MUST stay labeled as future work in the main
spec and repo docs until code for them ships in the runtime path.

#### Scenario: Docs do not advertise unimplemented builders as available

- GIVEN the repo documentation and main specs
- WHEN they describe supported build backends
- THEN bubblewrap on Linux is listed as the shipped implementation
- AND WASM, OCI, and remote builders are labeled as future work

### Requirement: BuildService trait as the portability boundary

The `BuildService` trait MUST remain the abstraction boundary for future
portability work, and the implementation list in the main spec MUST match what
this tree actually ships.

```rust
#[async_trait]
pub trait BuildService: Send + Sync {
    async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult>;
}
```

Current runtime status:

| Implementation | Status | Notes |
|---|---|---|
| `BubblewrapBuildService` | shipped | Current Linux build path |
| other backends behind `BuildService` | future work | Do not describe them as available until crunch wires and ships them |

#### Scenario: Current implementation table is honest

- GIVEN the main portability spec
- WHEN it lists concrete build-service implementations
- THEN it marks bubblewrap as the current shipped build path
- AND it does not present unimplemented services as current runtime options

### Requirement: Filesystem abstraction for store operations

Store operations (reading/writing blobs, directories, PathInfo) MUST
go through trait abstractions (`BlobService`, `DirectoryService`,
`PathInfoService`) already defined in snix-castore and snix-store.

These traits are already OS-agnostic. Platform-specific filesystem
behavior (symlink handling, permissions, case sensitivity) MUST be
handled in the trait implementations, not in callers.

#### Scenario: Case-insensitive filesystem

- GIVEN a macOS filesystem (case-insensitive by default)
- WHEN store paths that differ only in case are used
- THEN the store implementation handles this correctly (e.g., by using
  a case-sensitive disk image, or by rejecting collisions)

### Requirement: Portable Nickel stdlib

The Nickel stdlib MUST NOT contain OS-specific assumptions. The `System`
enum currently lists Linux and Darwin targets. New targets (BSD, Redox)
MUST be addable by extending the enum without changing the contract
structure.

```nickel
let System = [|
  'x86_64-linux,
  'aarch64-linux,
  'x86_64-darwin,
  'aarch64-darwin,
  'x86_64-freebsd,
  'x86_64-openbsd,
  'x86_64-redox,
  # ...
|] in
```

#### Scenario: Adding a new target extends the system enum only

- GIVEN crunch adds support for a new host target such as `x86_64-redox`
- WHEN the Nickel stdlib target contract is updated
- THEN support is added by extending the `System` enum
- AND the record contract structure does not need OS-specific rewrites

### Requirement: v0 targets Linux, architecture supports all

v0 MUST work on Linux (x86_64 and aarch64). The architecture MUST NOT
contain Linux-only assumptions outside of the sandbox implementation.
Adding macOS, BSD, or Redox support SHOULD require only:

1. A new `BuildService` implementation for that platform's sandbox
2. Adding system variants to the Nickel `System` enum
3. Platform-specific store considerations (if any)

No changes to evaluation, derivation construction, or the CLI.

#### Scenario: New platform support stays outside eval and CLI core

- GIVEN crunch adds support for a non-Linux host such as BSD or Redox
- WHEN that platform work is implemented
- THEN the new work is confined to the sandbox/runtime portability boundary
- AND evaluation, derivation construction, and CLI command structure remain unchanged

### Requirement: Eval execution strategy stays outside the portable core

The portable evaluation core MUST NOT require OS process management, daemon
lifecycle, or mandatory global thread-pool state in order to evaluate Nickel or
force roots.

`crunch-eval` MUST remain usable with its required serial inline backend alone.
Threaded and subprocess execution strategies MAY be shipped as host-specific
optimizations, but they MUST stay optional layers above the portable forcing
core.

If a subprocess backend is shipped, it MUST NOT require a resident service and
MUST NOT make library callers or non-process targets spawn another `crunch`
binary in order to use the eval core.

#### Scenario: Portable eval core works without host process helpers

- GIVEN a host or embedding target that does not want subprocess spawning or a
  resident worker service
- WHEN it uses the `crunch-eval` lazy forcing APIs through the required inline
  backend
- THEN evaluation and root forcing still work
- AND the portable core does not require daemon lifecycle management

#### Scenario: Subprocess backend remains optional host policy

- GIVEN a host runtime that adds a local subprocess backend for eval work
- WHEN that backend is available
- THEN it is selected as an optional host policy choice
- AND the portable eval core still remains usable without subprocess support
- AND the architecture does not require a resident daemon or cross-command
  worker service

