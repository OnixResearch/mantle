# Defaults Specification

## Purpose

Defines features that are experimental or opt-in in Nix but built in as
defaults in mantle. We have no backwards compatibility constraint and no
reason to gate good ideas behind feature flags.

## Context

There are three levels of hashing in the system:

1. **Castore level** — how blobs and directories are identified in storage.
   snix-castore uses BLAKE3 (`B3Digest`). We inherit this.

2. **Derivation level** — how derivation paths and output paths are computed.
   Nix uses SHA-256. mantle uses BLAKE3. This means mantle-built store paths
   differ from Nix-built store paths for the same derivation parameters.
   This is intentional — we don't need Nix compatibility.

3. **Output addressing** — how a derivation's output path relates to its
   inputs or its content:
   - **Input-addressed** (v0 default): output path is derived from the
     derivation's inputs (ATerm hash). Same inputs → same output path,
     even if the build is non-deterministic.
   - **Content-addressed** (future): output path is derived from the actual
     build output content. Identical outputs get identical paths regardless
     of how they were built.

## Requirements

### Requirement: BLAKE3 as default derivation hash

mantle MUST use BLAKE3 for derivation-level hashing. Specifically:

- `hash_derivation_modulo`: BLAKE3 hash of the ATerm representation
  (replacing `Sha256::digest(aterm)` with `blake3::hash(aterm)` in vendored
  nix-compat)
- `calculate_derivation_path`: BLAKE3 of the derivation ATerm for the
  `.drv` store path
- `build_output_path`: BLAKE3-based output path computation
- `build_text_path`: BLAKE3-based text path computation

The store path format MUST remain: `/nix/store/<nixbase32(hash[0:20])>-<name>`.
Only the hash function changes (SHA-256 → BLAKE3). Path length is unchanged.

#### Scenario: Mantle store paths differ from Nix

- GIVEN the same derivation parameters (name, builder, system, etc.)
- WHEN mantle computes the output path
- THEN the path differs from what Nix would produce (different hash function)

#### Scenario: Seed paths are unaffected

- GIVEN seed tool paths from an existing Nix store (`/nix/store/xxx-bash-5.2`)
- WHEN used as `Input::Source` in a mantle derivation
- THEN they are referenced by their original Nix-produced paths as-is.
  mantle does not re-hash source inputs.

### Requirement: BLAKE3 consistency across all levels

The system MUST use BLAKE3 consistently at every applicable hashing level:

| Level | Hash | Notes |
|---|---|---|
| Castore (blobs, directories) | BLAKE3 | Inherited from snix-castore |
| Derivation (ATerm → drv path) | BLAKE3 | Changed from Nix's SHA-256 |
| Output path computation | BLAKE3 | Changed from Nix's SHA-256 |
| Fixed-output derivations | User-specified | FOD hashes use whatever algo the user declares (sha256, sha512, etc.) |

FOD output hashes remain user-specified because they verify upstream content
(e.g., a tarball's sha256). The derivation-level hashing of the FOD itself
uses BLAKE3.

#### Scenario: Fixed-output derivation keeps declared content hash

- GIVEN a fixed-output derivation declares `sha256` for upstream content
- WHEN mantle computes both the FOD output hash and the derivation path
- THEN the upstream content is verified with the declared `sha256` hash
- AND the derivation path fingerprint still uses BLAKE3

### Requirement: Modify vendored nix-compat for BLAKE3

The vendored `nix-compat` crate MUST be modified to:

1. Add `blake3` as a dependency
2. Replace `sha2::Sha256` with `blake3::Hasher` in:
   - `Derivation::hash_derivation_modulo`
   - `store_path::build_output_path`
   - `store_path::build_text_path`
   - `store_path::build_store_path_from_fingerprint`
3. Keep SHA-256 available for FOD content hashing (user-specified hashes)

#### Scenario: Vendored crate compiles

- GIVEN the BLAKE3 modifications to nix-compat
- WHEN `cargo check` is run
- THEN the workspace compiles without errors

### Requirement: v0 uses input-addressed derivations

v0 MUST use input-addressed derivations. Output paths are computed from the
derivation ATerm hash (now BLAKE3), before the build runs.

Fixed-output derivations (FODs) are content-addressed by definition — their
output path is computed from the declared hash. This is already handled by
`build_ca_path` in nix-compat. FODs in v0 work as expected.

#### Scenario: Regular v0 output path is known before build

- GIVEN a non-fixed-output v0 derivation
- WHEN mantle computes the derivation output paths
- THEN each output path is derived from the ATerm hash before the builder runs

### Requirement: Content-addressed derivations as default

mantle SHOULD implement content-addressed derivations as the default output addressing mode, and the architecture MUST support both content-addressed and input-addressed modes from the start.
When a build completes, the output path is determined by the content of the
output (BLAKE3 hash of the NAR), not by the derivation inputs.

This means:

- Two derivations with different inputs but identical outputs get the
  same output path
- Changing a comment in a build script that doesn't affect the output
  does not trigger downstream rebuilds
- Output paths are not known before the build completes

The pipeline MUST be structured accordingly:

- Output path computation MUST be a post-build step (hash the output),
  not a pre-build step (hash the inputs)
- The `KnownPaths` structure MUST support entries where the output path
  is resolved after build
- The `BuildRequest` MUST NOT assume that output paths are known before
  the build starts
- Provisional paths (for self-references in environment variables) MUST
  use placeholders that are rewritten after the output path is known

Fixed-output derivations remain pre-computed (the user declares the hash).

#### Scenario: Identical outputs, different inputs

- GIVEN derivation A built with gcc-13 and derivation B built with gcc-14
- WHEN both produce byte-identical outputs
- THEN both get the same output path

#### Scenario: No-op rebuild avoided

- GIVEN a derivation was previously built
- WHEN a non-output-affecting change is made (e.g., build script comment)
- THEN the output path is the same, and downstream consumers are not rebuilt

**Note:** v0 MAY initially implement input-addressed derivations for
simplicity and switch to content-addressed as the default once the
pipeline stabilizes. The architecture MUST support both from the start.

### Requirement: Dynamic derivations as default

mantle SHOULD support dynamic derivations, meaning builds that produce `.drv` files as outputs, and the build orchestration MUST remain able to accept derivations discovered at build time.
Those produced `.drv` files are then built in turn. This enables meta-build
scenarios and is needed for self-hosting.

The build orchestration MUST:

- Inspect build outputs after completion (already required for ref scanning)
- Detect `.drv` files in outputs and schedule them for building
- Support derivation outputs that are themselves derivation inputs
  ("text hashing" derivation outputs)

**Note:** v0 MAY defer dynamic derivation support. The architecture MUST
support it from the start — the build loop must accept new derivations
discovered at build time.

#### Scenario: Build produces a derivation

- GIVEN a derivation whose build script writes a `.drv` file to `$out`
- WHEN the build completes
- THEN mantle detects the `.drv`, builds it, and the final output is
  available

### Requirement: cgroup isolation as default

mantle SHOULD run builds inside cgroups on Linux; v0 MUST be allowed to defer cgroup support because it is not required for correctness.
When cgroup support is available, this enables:

- Resource usage tracking per build (CPU, memory, I/O)
- Resource limits (prevent a single build from consuming all memory)
- Clean process cleanup on build timeout/failure

snix-build's OCI builder already creates namespaces. cgroup integration
SHOULD be added to the bwrap builder as well.

**Note:** v0 MAY defer cgroup support. It is not required for correctness.

#### Scenario: Cgroup-enabled build reports bounded resources

- GIVEN Linux cgroup support is enabled for builds
- WHEN a build runs through the sandbox
- THEN mantle can attribute CPU, memory, and I/O usage to that build

### Requirement: Git-native content addressing

mantle SHOULD support git-native content addressing for source trees, and source-tree addressing MUST remain extensible enough to preserve git tree identity.
When a source input is a git repository, the tree hash from git can be
used directly as the content address instead of re-hashing through NAR.

This avoids the overhead of converting git trees to NAR format and back,
and enables direct verification against git commit hashes.

**Note:** v0 MAY defer this. The `Input` type would gain a `GitTree`
variant in addition to `Source` and `Derivation`.

#### Scenario: Git tree identity can be preserved

- GIVEN a git source input with a known tree hash
- WHEN mantle materializes the source tree
- THEN the source-addressing model can record that git tree identity without
  requiring a NAR-only identity

### Requirement: Verified fetches as default

When mantle fetches git repositories, signature verification SHOULD be enabled by default, and fetcher design MUST support that policy when a signing key is provided.
This means:

- `fetchGit` (or mantle's equivalent) verifies commit signatures when
  a signing key is provided
- Unsigned fetches require explicit opt-out, not opt-in for verification

Security should be the default. Nix gates this behind `verified-fetches`
because of backwards compatibility. We have no such constraint.

**Note:** v0 MAY defer fetcher implementation entirely. When fetchers
are added, verification should be the default.

#### Scenario: Unsigned git fetch requires explicit opt-out

- GIVEN a git fetch has a configured signing policy
- WHEN the fetched revision cannot be verified
- THEN the fetch is rejected unless the user explicitly opts out of verification

### Requirement: Enforced sandbox security (from Lix)

The build sandbox MUST enforce the strongest available isolation on
each platform. There MUST NOT be an option to weaken it.

| Platform | Mechanism |
|---|---|
| Linux | seccomp-bpf syscall filtering + no-new-privileges + namespaces |
| macOS/Darwin | `sandbox-exec` profiles (or equivalent) |
| BSD | `pledge`/`unveil` (OpenBSD), `capsicum` (FreeBSD), or chroot fallback |
| Other | chroot + restricted PATH at minimum |

The security boundary is per-platform but the principle is uniform:
builds cannot escape the sandbox, escalate privileges, or access
undeclared inputs.

#### Scenario: Sandbox weakening is unavailable

- GIVEN a user starts a build
- WHEN mantle selects the platform sandbox
- THEN it enables the strongest available isolation for that platform
- AND no CLI option weakens that isolation boundary

### Requirement: Parallel builds (from Determinate Nix)

mantle SHOULD build independent derivations in the dependency graph concurrently, and the architecture MUST support parallelism from the start.
The `BuildService` trait is already async and supports concurrent `do_build`
calls.

The scheduler identifies derivations whose inputs are all built and
submits them to the build service in parallel, bounded by a
configurable concurrency limit (default: number of CPU cores).

**Note:** v0 MAY start with sequential builds. The architecture MUST
support parallelism from the start.

#### Scenario: Independent builds run concurrently

- GIVEN derivations A and B with no dependency on each other
- WHEN both are ready to build
- THEN both builds start concurrently (up to the concurrency limit)

### Requirement: Managed garbage collection (from Determinate Nix)

mantle SHOULD include automatic garbage collection; manual `mantle store gc` MUST remain sufficient for v0 if automatic GC is deferred.
Long-term garbage-collection control can:

- Run in the background (or before builds when disk is low)
- Maintain a configurable minimum free disk space (default: 10GB)
- Delete unreferenced store paths oldest-first
- Enter urgent mode if disk falls below a critical threshold

Users SHOULD NOT need to manually run a GC command for normal operation.
A `mantle store gc` command MAY be provided for manual control.

**Note:** v0 MAY defer automatic GC. A manual `mantle store gc` command
is sufficient for v0.

#### Scenario: Manual GC keeps free space policy explicit

- GIVEN automatic GC is not yet enabled
- WHEN an operator runs `mantle store gc`
- THEN mantle deletes only unreferenced store paths according to the configured
  retention policy

### Requirement: Auto-fix FOD hash mismatches (from Determinate Nix)

mantle MUST report and optionally fix declared hash mismatches when a
fixed-output derivation build completes with a different output hash:

1. Report the expected hash and the actual hash
2. Display the exact line in the `.ncl` file to update
3. Optionally (with `--fix`) update the `.ncl` file directly

This eliminates the manual cycle of: build → fail → copy hash → paste →
rebuild.

#### Scenario: Hash mismatch with fix suggestion

- GIVEN a FOD with `hash = "sha256-AAAA..."` but the actual content
  hashes to `sha256-BBBB...`
- WHEN the build fails
- THEN mantle prints: `expected: sha256-AAAA...` / `got: sha256-BBBB...` /
  `update hello.ncl:7 to: hash = "sha256-BBBB..."`

#### Scenario: Auto-fix

- GIVEN the same mismatch
- WHEN `mantle build --fix hello.ncl` is run
- THEN mantle updates the hash in `hello.ncl` and exits, instructing
  the user to re-run (the derivation path changed)

### Requirement: Hash algorithm in Nickel contracts

The `HashAlgo` enum in the Nickel stdlib MUST include `'md5`, `'sha1`,
`'sha256`, `'sha512`, and `'blake3`. BLAKE3 is a first-class hash
algorithm, not an experimental addition.

#### Scenario: Nickel accepts blake3 hash declarations

- GIVEN a Nickel fetch declaration sets `algo = 'blake3`
- WHEN the declaration is checked against the stdlib contracts
- THEN the hash algorithm is accepted as a first-class option

### Requirement: Independence from Nix

mantle MUST NOT depend on Nix experimental features being available in
any Nix installation. mantle operates independently. The features listed
here are design decisions for mantle, informed by but not dependent on
Nix's experimental feature process.

#### Scenario: Missing Nix experimental flags do not affect mantle

- GIVEN a host Nix installation has no experimental features enabled
- WHEN mantle evaluates or builds its own Nickel-based inputs
- THEN mantle behavior is unchanged because it does not depend on those flags

## Summary

### From Nix experimental features

| Feature | mantle status |
|---|---|
| `blake3-hashes` | **Default.** BLAKE3 everywhere. |
| `ca-derivations` | **Default.** Content-addressed outputs. (v0 may start input-addressed.) |
| `dynamic-derivations` | **Default.** Builds can produce derivations. (v0 may defer.) |
| `cgroups` | **Default.** Resource isolation. (v0 may defer.) |
| `git-hashing` | **Default.** Git-native content addressing. (v0 may defer.) |
| `verified-fetches` | **Default.** Signature verification on. (v0 may defer fetchers.) |
| `flakes` | **N/A.** Nix CLI concept. |
| `nix-command` | **N/A.** Nix CLI v2. |
| `recursive-nix` | **Subsumed** by dynamic derivations. |
| `impure-derivations` | **Deferred.** Undermines reproducibility. |
| `auto-allocate-uids` | **Deferred.** Multi-user store. v0 is single-user. |
| `fetch-tree` / `fetch-closure` | **N/A.** Nix builtins. |
| `pipe-operators` | **N/A.** Nix language. Nickel has `\|>`. |
| `local-overlay-store` | **Deferred.** Store composition. |
| `external-builders` | **Inherited.** snix-build has the `BuildService` trait. |

### From Lix

| Feature | mantle status |
|---|---|
| Enforced syscall filtering + no-new-privileges | **Default.** Unconditional in sandbox. No opt-out. Security is not optional. |
| Custom subcommands (`lix-custom-sub-commands`) | **Deferred.** Plugin subcommands (`mantle foo` → `crunch-foo`) are useful but not v0. |
| Better error values/positions | **Inherited.** Nickel and serde both have good error reporting. |
| REPL improvements | **N/A.** Nickel has its own REPL. |

### From Determinate Nix

| Feature | mantle status |
|---|---|
| Parallel builds | **Default.** Independent derivations in the DAG SHOULD build concurrently. `BuildService` is async. (v0 may start sequential.) |
| Managed garbage collection | **Default.** Automatic GC with disk space thresholds. No manual `nix-collect-garbage`. |
| Auto-fix FOD hashes | **Default.** When a fixed-output derivation hash mismatches, report the correct hash so the user can update their `.ncl`. |
| Lazy trees | **N/A.** Nix flake input optimization. No flakes. |
| Native Linux builder for macOS | **Deferred.** v0 is Linux-only. Interesting for future macOS support. |
| WebAssembly in expressions | **N/A.** Nix evaluator feature. Nickel could have its own WASM story. |
