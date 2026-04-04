# Portability Specification

## Purpose

Defines how crunch avoids platform lock-in. The core (evaluation, derivation
construction, store) MUST be OS-agnostic. Platform-specific code (sandbox,
filesystem) MUST be behind trait abstractions so new platforms can be added
without modifying the core.

## Requirements

### Requirement: Layered platform abstraction

The system MUST be structured in three layers:

1. **Core (OS-agnostic):** Nickel evaluation, derivation construction,
   KnownPaths, store path computation, BuildRequest generation. No
   `#[cfg(target_os)]` in this code.

2. **Platform traits:** Abstract interfaces for sandbox execution, filesystem
   operations, and store backend. Defined in terms of capabilities, not
   OS primitives.

3. **Platform implementations:** Concrete implementations of the traits for
   each supported platform. Behind `#[cfg(target_os)]` or cargo features.

```
crunch-eval          (OS-agnostic — Nickel is pure Rust, cross-platform)
crunch-glue          (OS-agnostic — serde + nix-compat data structures)
nix-compat           (OS-agnostic — pure computation, no I/O)
snix-castore         (mostly OS-agnostic — blob/dir storage)
snix-store           (mostly OS-agnostic — pathinfo, NAR)
snix-build           (platform-specific sandbox behind BuildService trait)
  ├── wasm           (DEFAULT: wasmtime + WASI — runs everywhere)
  ├── bwrap          (Linux: bubblewrap + namespaces + seccomp)
  ├── oci            (Linux: OCI container runtime)
  ├── grpc           (OS-agnostic: delegate to remote builder)
  └── (future)       (darwin, pledge, capsicum — as needed)
```

#### Scenario: Build on OpenBSD

- GIVEN crunch compiled on OpenBSD
- WHEN `crunch build hello.ncl` runs with the WASM sandbox (default)
- THEN evaluation, derivation construction, and store operations work
  identically to Linux; the build executes in wasmtime with WASI

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

### Requirement: Native sandbox as default, WASM as future goal

v0 MUST use native platform sandboxes as the default:

- Linux: bwrap (bubblewrap) with namespaces + seccomp
- Other platforms: gRPC to a remote builder, or unsandboxed with warning

The reason WASM cannot be the default today: **standard WASI has no
process spawning** (no `fork`, `exec`, `spawn`). A bash build script
that calls `gcc`, then `cp`, then `mkdir` cannot run in WASI. Build
scripts need to orchestrate multiple tools as subprocesses — this is
fundamental to how builds work.

WASIX (Wasmer's extension) adds `proc_fork`/`proc_exec`/`proc_spawn`,
but it is not standardized, Wasmer-only, and the ecosystem of tools
compiled to WASIX is thin.

Once WASI gains subprocess support (it is on the standards roadmap),
WASM becomes the right default. The architecture MUST support this
transition.

A Nickel derivation MAY declare its sandbox preference:

```nickel
{
  name = "my-tool",
  sandbox | default = 'native,  # or 'wasm, 'oci
  ...
}
```

### Requirement: WASM sandbox for single-process builds

Even without subprocess support, WASM is viable for builds that are a
single tool invocation (no shell script orchestration). crunch SHOULD
support a `WasmBuildService` for these cases:

- A Rust program compiled to `wasm32-wasip1` that reads inputs and
  writes outputs
- A custom build tool that doesn't shell out
- Fetch operations (download + hash)

This is opt-in via `sandbox = 'wasm` in the derivation.

#### Scenario: Single-process WASM build

- GIVEN a derivation whose builder is a WASM module that reads input
  files and writes output files without spawning subprocesses
- WHEN `crunch build` runs with `sandbox = 'wasm`
- THEN wasmtime loads the module, pre-opens directories via WASI,
  executes, collects outputs. Works on any OS.

#### Scenario: Shell script cannot use WASM sandbox

- GIVEN a derivation whose builder is bash with a script that calls gcc
- WHEN `sandbox = 'wasm` is set
- THEN crunch rejects the build with an error explaining that WASM
  sandbox does not support subprocess spawning. User must use 'native.

### Requirement: BuildService trait as the portability boundary

The `BuildService` trait from snix-build is the portability boundary for
build execution. All sandbox code MUST live behind this trait.

```rust
#[async_trait]
pub trait BuildService: Send + Sync {
    async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult>;
}
```

Implementations:

| Implementation | Platform | When to use |
|---|---|---|
| `WasmBuildService` | Any (wasmtime) | Default. Portable, deterministic. |
| `BubblewrapBuildService` | Linux | Native builds needing Linux specifics. |
| `OCIBuildService` | Linux | Container-based builds. |
| `GRPCBuildService` | Any | Delegate to remote builder. |

Adding a new sandbox means implementing this trait. The rest of crunch
is unchanged.

#### Scenario: Cross-platform via remote builder

- GIVEN crunch running on a platform with no native sandbox
- WHEN `crunch build --sandbox grpc://linux-builder:8080 hello.ncl`
- THEN evaluation and derivation construction run locally; the build is
  sent to the remote builder via gRPC

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

### Requirement: v0 targets Linux, architecture supports all

v0 MUST work on Linux (x86_64 and aarch64). The architecture MUST NOT
contain Linux-only assumptions outside of the sandbox implementation.
Adding macOS, BSD, or Redox support SHOULD require only:

1. A new `BuildService` implementation for that platform's sandbox
2. Adding system variants to the Nickel `System` enum
3. Platform-specific store considerations (if any)

No changes to evaluation, derivation construction, or the CLI.
