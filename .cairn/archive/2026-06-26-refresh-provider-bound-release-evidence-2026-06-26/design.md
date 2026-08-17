## Context

The previous provider-bound release evidence bundle `provider-bound-release-evidence-2026-06-25` proved provider-bound verification for an older packaged provider binary but also recorded that a current-code provider-backed fixed-point rerun still blocked. The native static-PIE CRT fix produced a successful current-code provider fixed point in bundle:

```text
/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-native-static-pie-crt-2026-06-26-rerun4
```

That proof reports `fixed_point: true` and stage1/stage2 binary digest `b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3`.

## Decisions

### 1. Keep release evidence bundle-local and bounded

Package the successful provider proof as `proof/provider-fixed-point` and include its stage binary as `binaries/01-mantle`. Keep the existing full self-hosting proof bundle and its stage2 binary as the second packaged artifact so the release manifest still has full-proof linkage.

### 2. Require both release verification gates

Run `mantle release verify --require-deterministic-release --require-provider-fixed-point-proof` and record the verifier output fields that bind provider proof status, matched artifact path, matched artifact digest, deterministic proof digest, and sandbox isolation evidence digest.

### 3. Preserve portable replay evidence

Copy the bundle and verifier to a replay directory, verify the copy with both gates, then remove the deterministic build proof and confirm verification fails closed. This demonstrates copied artifact sufficiency and missing-proof rejection without claiming deployability or global reproducibility.

## Risks / Trade-offs

- The deterministic rebuild helper is intentionally a bounded copy-rebuild recipe for the packaged artifact set. The transcript must state this is not a full-bootstrap reproducibility claim.
- The second binary remains the existing full self-hosting proof stage2 artifact. The refreshed current-code provider proof is bound to `binaries/01-mantle`.
