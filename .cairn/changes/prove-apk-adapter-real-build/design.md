# Design: Real-toolchain APK adapter proof

## Context

The adapter's offline tests prove same-run extraction admission and rejection before SDK effects; they do not run synthetic tool stages or produce APKs. The real question is whether three authenticated extraction derivations and six SDK stages, driven by the admitted prebuilt toolchain and a signed native runtime cohort, produce a structurally valid signed APK with reproducible bytes. That question is answered only by executing the real path and recording what happened.

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

The fresh-build entrypoint consumes an already-entered pinned Nix devshell,
not a new `nix develop` evaluation of the immutable 10,000-member snapshot
for every SDK command. It verifies the exact signed Bubblewrap output and
invokes that executable directly for sandboxed keytool and `apksigner`.
Before a long build the first separately pinned Mantle executable and
verifier tools need Nix GC roots: losing the unrooted output invalidates a
later receipt recheck even when its derivation can recreate identical bytes.
For an interrupted post-build stage, a second read-only, independently
pinned full-source verifier snapshot can inspect the persisted original
build snapshot, original-HTTPS source state and replay, full passing ignored
test log, three distinct build stores and every APK. Its success receipt
records both snapshot identities and repeats actual reviewed-tool
verification and the signed-region tamper control. It never treats a
missing build as recovered evidence.

The reviewed-to-published source bridge lowers the fetch root using the
published Android source-v1 records after checking the separate reviewed
metadata. This preserves the original-HTTPS bundle's exact FOD name, SRI
SHA-256 and source identity; fetching a second, reviewed-metadata-named
derivation would incorrectly classify the already imported sources as
network-required. The retained bundle has four records, including separately
checked command-line tools; the APK root consumes three. An imported and
pinned bundle is not enough alone: its three build roots must preflight `ready`
and realize three fresh uncached outputs through `SourceFetchOverridePlan`.

The reviewed JDK's `jar --update --date` rejects a ZIP entry timestamp of
`1980-01-01T00:00:00Z`; `1980-01-01T00:00:02Z` is its first valid value.
Nickel authoring and both Rust APK contracts now use the same two-second
lower bound before dispatch, rather than admitting a deterministic plan that
fails only when the real zipalign stage invokes `jar`.

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

The direct adapter binds the plan's declared `keys/` member beneath a separately
signed test-only keystore store object, never the app source object. It also
rejects a signing password stored inside that app source object. Only the
`apksigner` derivation declares the keystore and password store objects; the
other five stages cannot read signing material through their common app source
input. A same-byte key at another member, a collocated app-source key, or an
escaping symlink is rejected before any SDK stage.

The actual `apksigner` invocation also needs two distinct password file paths:
one for the keystore and one for its private key. It consumes the stream
separately for each role, so reusing a one-line file twice fails at EOF.
The adapter rejects a shared path before dispatch; the fixture admits
`store.pass` and `key.pass` as independently addressable members of the
separately signed password object. The test-only bytes can be equal.

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
