# Design: Real-toolchain APK adapter proof

## Context

The adapter's offline tests use stub tools. The real question is whether the five derivations, driven by the admitted prebuilt toolchain, produce a structurally valid APK with reproducible bytes. That question is answered only by executing the real path once and recording what happened.

The repo already has the pattern for this: execution-gated proof changes that record environment prerequisites, run detached with pueue, and never quote results from memory.

## Architecture

```text
examples/android-minimal.ncl
  -> adapter lowering (unchanged)
  -> admitted toolchain components (fixed-output fetch, or prefetched bundle)
  -> five sandboxed derivations
  -> APK output
  -> structural verification + digest receipt
  -> second clean rebuild in a fresh store
  -> determinism comparison
```

### Example application

`examples/android-minimal.ncl` is deliberately small: one activity, the manifest, one Java source, no resources that need `aapt2 compile` beyond the manifest link step. It exercises every step boundary while keeping the first real run cheap. Application id and version are fixed in the example.

### Evidence rail

A checked-in rail builds the example and writes a receipt binding:

- the toolchain identity records consumed, with digests;
- the derivation identities of each step;
- the output BLAKE3;
- the environment facts that could affect bytes (platform, fetch mode).

The rail runs detached with pueue under the repo's long-build conventions. Prefetched bundles replay offline through `SourceFetchOverridePlan`, so the expensive network dependency is one-time.

### Determinism proof

Two clean rebuilds in fresh store and state directories must produce the same output BLAKE3. A negative control perturbs the Java source and asserts the digest changes. This catches accidental ambient state in any of the five steps, the same class of drift the busybox kbuild pinning exposed in the self-hosting proof.

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
- Determinism mismatch: the rail reports the first differing digest pair and the step that produced it.
- Toolchain digest drift: identity binding fails before execution, per the admission change.

## Testing

- Positive: build succeeds, structural checks pass, both rebuild digests match.
- Negative: perturbed source changes the digest; tampered APK fails `apksigner verify`; missing toolchain fails with the admission diagnostic.
