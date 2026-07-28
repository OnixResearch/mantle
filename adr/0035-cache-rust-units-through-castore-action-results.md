# ADR 0035: Cache Rust units through castore action results

## Status

Proposed (2026-07-27)

## Context

Mantle has three relevant execution surfaces.

First, ordinary derivation builds already use PathInfo, castore, substitution, and shared action-result admission.

Second, `mantle rust-plan` executes explicit Rust units and records detailed source, toolchain, argument, environment, dependency, host-artifact, and output facts.

Third, external Cargo builds can invoke a custom `RUSTC_WRAPPER` process for each compiler invocation.

Current `rust-plan` reuse depends on artifacts that remain in the prior execution output directory. It does not restore deleted unit outputs from castore.

A direct wrapper implementation would duplicate identity and store access in many short-lived processes. Argument and path normalization alone would not identify all compiler inputs.

Snix castore FUSE is read-only. It cannot serve as Cargo's writable target directory.

ADR 0024 already separates immutable objects, action-result discovery, and execution. Rust unit caching must preserve that separation.

## Decision Drivers

- Reuse supported Rust unit outputs after execution directories are removed.
- Share physical BLAKE3 chunks with other Mantle content.
- Keep Rust action identity, result authority, content presence, and execution separate.
- Preserve deterministic receipts and explicit non-claims.
- Avoid writable target-directory assumptions on read-only FUSE.
- Support external Cargo later without weakening Mantle proof lanes.

## Decision

Mantle will implement Rust unit caching in three ordered changes.

### 1. Add local castore-backed reuse to `rust-plan`

Mantle will define a canonical Rust unit action identity and a separate immutable Rust unit result record.

The action identity will bind all declared execution inputs. It will not use compiler arguments or normalized source paths as the complete key.

Current execution-directory reuse remains first. Local castore lookup follows it. Compiler execution follows an admitted miss.

Successful compiler artifacts will enter castore before the result record becomes discoverable.

### 2. Add signed shared Rust unit action results

Remote lookup will use provider-neutral action-result and object-store boundaries. Discovery remains advisory.

Mantle will verify record authority, action identity, policy, complete content, artifact manifests, and materialization before skipping compiler execution.

Remote publication will make immutable objects visible first, signed records second, and no-clobber index candidates last.

### 3. Add an optional daemon-backed Cargo wrapper

The wrapper will remain a thin local client. One daemon will own stores, remote clients, policy, compiler execution, and receipts.

Strong cache use will require a verified declared-input invocation manifest. Unsupported or undeclared invocations will pass through or fail under explicit policy.

Mantle-native `rust-plan` will not call through this wrapper. Strict proof lanes will continue to scrub ambient wrapper state unless another accepted change admits it.

### Materialization boundary

Mantle will restore cached compiler artifacts into verified private staging. It will commit ordinary files to approved output paths only after complete verification.

Mantle will not mount castore FUSE or virtiofs over writable Rust or Cargo output roots for these changes.

Read-only artifact exposure can be evaluated later. It requires separate performance and compatibility evidence.

### Semantic boundary

Final derivation outputs and Rust unit artifacts can share physical castore chunks.

They will keep separate action schemas, result records, trust admission, retention, garbage-collection roots, receipts, and claims.

Object presence will never authorize Rust unit reuse by itself.

## Alternatives Considered

### Add an ambient `RUSTC_WRAPPER` first

Rejected as the first route. Mantle already controls native Rust unit execution, while wrapper arguments do not prove a complete input closure.

### Use compiler flags and normalized paths as the cache key

Rejected. Compiler, sysroot, environment, dependency, proc-macro, build-script, native-link, policy, and effect facts also affect outputs.

### Treat castore object presence as a cache hit

Rejected. A content identity does not state which action produced the content or whether current policy admits it.

### Mount FUSE or virtiofs over Cargo `target/`

Rejected for this change set. The current castore FUSE surface is read-only, while Cargo and rustc require writable output paths.

### Store Rust unit outputs as ordinary derivation PathInfo

Rejected as the default semantic model. Rust unit output trees do not automatically have derivation store-path, reference-scan, or PathInfo authority.

## Consequences

- `rust-plan` gains the shortest and strongest path to micro-level reuse.
- Remote sharing reuses Mantle's action-result architecture without changing derivation result semantics.
- A later Cargo adapter shares the same Rust cache core instead of defining another identity model.
- Garbage collection must retain Rust unit result roots or remove stale result references first.
- Cache hits require explicit materialization cost unless a later read-only exposure change proves a better route.
- Strong wrapper eligibility remains narrower than arbitrary Cargo compatibility.
- Cache evidence does not prove compiler correctness, determinism, hermeticity, or release eligibility.
