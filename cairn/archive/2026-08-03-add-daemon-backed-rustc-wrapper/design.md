# Design: Thin Rust compiler wrapper with one Mantle cache daemon

## Context

Cargo starts one wrapper process for many compiler invocations. Each process receives the real compiler path followed by compiler arguments.

A wrapper sees arguments and selected environment values. Those facts do not prove a complete compiler input closure. Proc macros, build-script outputs, native link inputs, sysroot files, and included files can affect outputs.

The wrapper must therefore remain a thin protocol adapter. The daemon owns cache policy, storage services, compiler execution, and receipts.

## Decisions

### Decision 1: Keep this adapter outside Mantle-native Rust execution

**Choice:** `rust-plan` will call the Rust cache library directly. The wrapper exists only for explicit external Cargo developer workflows.

Strict self-build, witness, and release proof lanes will continue to scrub ambient wrapper and cache-daemon variables unless a later proof change admits them.

**Rationale:** Mantle already controls native unit execution. Adding a wrapper there would add another hidden process and identity boundary.

### Decision 2: Use a thin wrapper and one per-user daemon

**Choice:** Add `mantle-rustc-wrapper` as a small client. Add `mantle rust-cache serve` as the store-owning shell.

The daemon will own local indexes, castore services, remote clients, signing keys, policy, compiler execution, and receipt writing. The wrapper will not open store databases or read remote credentials.

**Rationale:** One daemon avoids per-invocation startup, lock contention, and credential duplication.

### Decision 3: Use a bounded local Unix-socket protocol

**Choice:** Use versioned length-bounded request and response messages over a Unix socket. The daemon will require an authorized peer user and restrictive socket permissions.

Requests will contain an invocation manifest reference, expected manifest BLAKE3, real compiler path, normalized argument data, admitted environment data, working-root identity, and output contract.

Responses will contain disposition, bounded diagnostics, compiler status, captured stream locations or bounded bytes, artifact commit status, and receipt identity.

**Rationale:** The wrapper protocol is a local authority boundary, not an unbounded serialization channel.

### Decision 4: Require an explicit declared-input manifest for strong caching

**Choice:** Strong lookup and publication require a versioned Mantle-generated invocation manifest. The manifest will bind:

- package, unit, host, target, profile, mode, feature, and source identities;
- allowed source, generated-output, sysroot, toolchain, native-link, and dependency roots;
- dependency, proc-macro, build-script output, and native artifact digests;
- compiler and linker closure identities;
- normalized compiler arguments and admitted environment;
- output paths and artifact classes;
- filesystem, network, clock, randomness, and compiler-policy dispositions;
- schema and policy versions.

The daemon will verify the manifest digest and every current declared input before cache lookup. Missing, unreadable, changed, out-of-root, or unclassified inputs will cause pass-through or a policy blocker.

**Rationale:** Arguments alone do not identify all compiler inputs.

### Decision 5: Run eligible misses under declared execution policy

**Choice:** For cache-eligible invocations, the daemon will execute the real compiler with explicit argv, environment, working directory, readable roots, writable output staging, resource limits, and effect policy.

If the host cannot enforce the declared boundary, the adapter will bypass caching and invoke the real compiler through the wrapper process.

**Rationale:** A declared manifest without enforcement does not confine undeclared file access.

### Decision 6: Preserve Cargo behavior and define bypass classes

**Choice:** Cargo remains responsible for planning, build-script runtime, fingerprints, and invocation order. The wrapper will preserve compiler exit status and stream behavior.

The adapter will bypass these initial classes:

- compiler queries and capability probes;
- incremental compilation;
- incomplete or unsupported output contracts;
- unsupported response-file or encoded-argument forms;
- missing declared-input manifests;
- unsupported compiler, linker, or proc-macro effect policy;
- daemon unavailability when fail-open developer policy is selected.

Fail-closed policy remains available for reviewed workflows.

**Rationale:** Transparent pass-through is safer than fabricating partial Cargo compatibility.

### Decision 7: Restore exact compiler artifacts atomically

**Choice:** The daemon will restore a complete admitted artifact set into a private staging area. It will verify all artifacts before committing them to Cargo's expected output paths.

The wrapper will return success only after the output commit and receipt write succeed. Restoration will not mount FUSE or virtiofs over Cargo's writable target directory.

**Rationale:** Cargo expects ordinary files and compiler process completion semantics.

### Decision 8: Reuse common local and remote Rust result semantics

**Choice:** The daemon will use the same action identity, result record, admission, castore, materialization, conflict, and receipt core as `rust-plan`.

Remote lookup and publication will remain disabled until `share-rust-unit-action-results` is available and explicitly configured.

**Rationale:** The wrapper is a transport and orchestration adapter. It must not create a second cache truth model.

### Decision 9: Author daemon policy in typed Nickel

**Choice:** Human-authored daemon policy will use a typed Nickel contract. A deterministic export will provide runtime data.

Policy will define cache mode, fail-open or fail-closed behavior, allowed roots, socket path policy, peer policy, size limits, time limits, effect policy, result sources, publication policy, and redaction.

**Rationale:** Cache eligibility and authority need reviewable typed configuration.

## Failure Semantics

- A missing daemon uses the selected fail-open pass-through or fail-closed error policy.
- A malformed, oversized, unauthorized, or replay-inconsistent request is rejected before compiler execution.
- An unsupported invocation passes through without cache publication.
- A compiler failure preserves the compiler status and bounded diagnostics and publishes no result.
- A restore failure returns failure and leaves no successful partial artifact set.
- A remote cache failure does not expose credentials or fabricate local reuse.

## Risks / Trade-offs

- Declared-input manifest generation limits general Cargo transparency.
- Compiler sandboxing can differ from normal Cargo developer behavior.
- Per-invocation socket traffic adds latency on misses and very small compilations.
- Multi-file output commit cannot be one filesystem rename when Cargo expects several independent destination paths.
- Proc-macro and native-link effect policy remains a major compatibility boundary.
