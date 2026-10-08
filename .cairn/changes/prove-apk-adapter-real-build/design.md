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

### Four-tree local admission gate

The signed replay binds four distinct complete source objects: app source,
native glibc, native libgcc, and test-only signing. `ApkInputs` names the
independent `native_libgcc_root`; its ordered runtime library directories bind
glibc's loader and `libc.so.6` under the glibc root and `libgcc_s.so.1` under
the libgcc root. Every SDK derivation depends on both native objects.
The operator stages and independently checks each full native Nix tree rather
than copying individual shared libraries. `crunch-android-admit observe` only
reports NAR SHA-256, BLAKE3, and size without store effects. Review matching
observations for both original Nix trees and staged copies, then supply
expected facts for all four trees before any `admit`. Only a trusted signature
on each exact non-CA PathInfo plus matching castore and physical NAR read-back
can establish `signed-physical`; an observed-only receipt is not an APK build,
a source-bundle admission, or a signed-ready claim. Disjoint fresh state/store
roots are required for subsequent A/B replays.

The reviewed Google `aapt2` requests an ELF interpreter at `/lib64`, absent
from the Bubblewrap tmpfs root. The adapter must instead declare the exact
native glibc/libgcc source trees as derivation inputs to every SDK stage and
invoke the declared glibc loader with an explicit library path; a host `/lib64`
or ambient `/nix/store` bind does not establish this proof. The proof compiler
must use an isolated, content-bound source snapshot rather than a mutable
shared workspace. These are requirements, not evidence that any APK stage
has executed.
### Determinism proof

Two clean rebuilds in fresh store and state directories must produce the same output BLAKE3. A negative control perturbs the Java source and asserts the digest changes. This catches accidental ambient state in any of the six signed stages, the same class of drift the busybox kbuild pinning exposed in the self-hosting proof.

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
