# Design: Real-toolchain APK adapter proof

## Context

The adapter's offline tests use stub tools. The real question is whether three authenticated extraction derivations and six SDK stages, driven by the admitted prebuilt toolchain and a signed native runtime cohort, produce a structurally valid signed APK with reproducible bytes. That question is answered only by executing the real path and recording what happened.

The repo already has the pattern for this: execution-gated proof changes that record environment prerequisites, run detached with pueue, and never quote results from memory.

## Architecture

```text
examples/android-minimal.ncl (one Java source, manifest, resource, test-only signing)
  -> original-HTTPS fixed-output source bundle; pinned offline SourceFetchOverridePlan
  -> signed glibc/libgcc NAR cohort plus declared static BusyBox shell/utility
  -> three same-run authenticated extraction derivations
  -> six sandboxed SDK stages via the declared glibc loader
  -> signed APK with independently checked ZIP, binary XML, DEX, APK Signing Block, apksigner
  -> second clean signed rebuild in a fresh store/state and changed-Java control
  -> matching clean BLAKE3 digests, different changed-Java digest, tamper rejection
```

### Example application

`examples/android-minimal.ncl` is deliberately small: one activity, the manifest, one Java source, and one XML string resource to exercise `aapt2 compile` and `aapt2 link`. It exercises every SDK boundary without depending on Gradle or Kotlin. Application id and version are fixed in the example; the keystore/password are separate test-only signing inputs.

### Evidence rail

A checked-in rail builds the example and writes a receipt binding:

- the toolchain identity records consumed, with digests;
- the derivation identities of each step;
- the output BLAKE3;
- the environment facts that could affect bytes (platform, fetch mode).

The rail runs detached with pueue under the repo's long-build conventions. Prefetched bundles replay offline through `SourceFetchOverridePlan`, so the expensive network dependency is one-time.

Concurrent CA/backend work prevents treating a mutable shared workspace as the proof compiler's source. The rail captures an isolated, read-only full-source tree only after `BACKEND TEST SNAPSHOT DONE`, CA final source-ready, the backend's serialized manifest/lock/vendor transaction, and the S0 B2/root integrator second-source stability barrier, including uncommitted owner files. Per-file BLAKE3 values (including `Cargo.lock` and every included owner source), canonical source-profile BLAKE3 and manifest BLAKE3 are Mantle-owned content identities; a separately named SHA-256 NAR digest is retained only for Nix tree interoperability. Every included source member is rehashed before and after execution; the evidence directory and Cargo target stay outside that tree. The first Mantle CLI binary retains its independently pinned SHA-256 and BLAKE3 and is used only as that exact executable; the later source snapshot for the APK library compiler is recorded separately rather than conflated with the first binary's source revision. Ignored vendored sources are excluded only when the proof's Cargo command does not activate the opt-in vendor configuration.

The reviewed Google `aapt2` requests an ELF interpreter at `/lib64`, absent from the Bubblewrap tmpfs root. The adapter instead admits exact Nix glibc/libgcc source trees with independently pinned full-tree NAR SHA-256 and signed PathInfo. Both complete trees are derivation inputs to every SDK stage; the shell's command argv explicitly invokes the declared glibc loader and a declared library path. No host `/lib64` or ambient `/nix/store` bind is part of the proof. The runner checks all four reviewed archive SHA-256 values, though command-line tools 19.0 are not consumed by the APK stages.

### Determinism proof

Two clean signed rebuilds in fresh store and state directories must produce the same output BLAKE3. A negative control perturbs the Java source and asserts the digest changes. This catches accidental ambient state in any of the six steps, the same class of drift the busybox kbuild pinning exposed in the self-hosting proof.

### Structural verification

Verification is bounded observation, not device truth:

- zip local file headers parse and entries are in deterministic order;
- compiled `AndroidManifest.xml` is present;
- at least one DEX entry is present;
- when signed, an APK Signing Block is present and `apksigner verify` exits successfully;
- a flipped byte in a signed region makes verification fail.

The tamper negative control proves the verifier observes content, not just shape.

## Failure and abuse controls

- Missing network or missing prefetch bundle: the rail fails closed with a blocker record, not a silent skip.
- Determinism mismatch: the rail writes a blocker identifying the mismatched digest comparison and retains the complete per-step builder observations in scratch.
- Toolchain digest drift: identity binding fails before execution, per the admission change.

## Testing

- Positive: build succeeds, structural checks pass, both rebuild digests match.
- Negative: perturbed source changes the digest; tampered APK fails `apksigner verify`; missing toolchain fails with the admission diagnostic.
