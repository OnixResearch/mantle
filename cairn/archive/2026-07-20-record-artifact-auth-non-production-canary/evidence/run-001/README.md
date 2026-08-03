# Mantle artifact-auth non-production canary run 001

This product-owned archive records the bounded public evidence produced on 2026-07-20 from Mantle `55fb02decfa42325ed192ac1249e8a6432844d50` with artifact-auth `799459346d5416fbd7b9f55840a7371441b55afa`.

## Observed result

- A real `examples/hello.ncl` build produced the archived build report and store output.
- Capture and fresh-process replay passed with receipt `blake3:7c7e8fbe3f9823204fe0dbc99c0ae81e6aee35aaa7fc3faafcc4c0f20dfef4c9`.
- Adding the complete public-key token digest to Mantle-owned revocation state caused the original receipt to fail closed.
- A later fresh process returned `CurrentnessNotCurrent`.

## Members

`harness.rs` is the exact temporary Rust example compiled against the landed revision. The JSON files and log are the public build, receipt, replay, and revocation observations. `build.json` intentionally retains historical local paths from the exact report; those paths are observations and are not live dependencies. `manifest.ncl` declares the claim boundary. Its cross-consumer revisions are review linkage, not a joint signature or attestation. `BLAKE3SUMS` binds every regular member except itself.

Private signing material, complete trust-source files, and mutable secret state are intentionally absent.

## Validation

```text
nix shell nixpkgs#nickel -c nickel typecheck manifest.ncl
./hash-evidence.rs . > regenerated && cmp BLAKE3SUMS regenerated
```

A negative fixture containing a symlink must be rejected by `hash-evidence.rs`. JSON and log members must contain no private-key marker or serialized secret-key field.

## Non-claims

This archive is non-production operational evidence. It does not establish remote trust discovery, global revocation freshness, cache or build admission, registry publication, release eligibility, production rollout, or standalone authority.
