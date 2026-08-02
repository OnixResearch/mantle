# Live GuixPkgs hello proof summary

## Result

A pinned GuixPkgs `hello.unwrapped` graph reached Mantle's `realized` state.
Consumption used neither Nix nor Guix. Mantle hydrated four exporter-signed
runtime paths through its normal HTTP cache, PathInfo, castore, scheduler,
worker, and store boundaries.

The proof also completed exact reuse and receipt-selected fresh-store hydration.
The bounded provenance audit completed with one finding. Therefore, the strongest
audit state remains `realized`, not `provenance-audited`.

## Producer identity

- GuixPkgs revision: `b3ec7b4f03c87a4d5fb67cc18e022567535e1795`
- GuixPkgs flake fingerprint: `4c3e2df7b232e75545f7175c72906f1a86e62308a10b0e69c93d607b60a82183`
- Upstream Guix revision: `ac03c482b1910a1672427beaea07ddcd1d652806`
- `guix-transfer` revision: `dc8f92898f6345120f5911e2e4d83a3bf8c6aad4`
- Producer Nix: `2.35.0`
- Package: `packages.x86_64-linux.hello.unwrapped`
- Root derivation: `/nix/store/nad8x3j7m2nz1x19678i2arv2g1g4k99-hello-2.12.2.drv`
- Root output: `/nix/store/23afiym7dlgh1lxk1lyx6xrzplwnwhsk-hello-2.12.2`
- Recursive graph units: 1,176

GuixPkgs Cachix supplied translated dependencies, but it did not contain the
selected root. Producer-side Nix built that root. The producer then signed the
four-member runtime closure with the retained public key
`mantle-live-guixpkgs-export-1:KqV1aJoHpBe+5MZt1KLCjo+JjtFCeIVYdevTAbtOXpo=`.
The secret key is not retained.

## Mantle identities

- Plan: `5bdd1c462583305a5d637f7490a39c44c6678da999d9bef6522fefa0b34bf336`
- Import receipt: `bda1a1cb8edd7ecd3259f7dc91ccabc58030df6d8d837bc0501aa287e1956506`
- Source bundle: `8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777`
- First realization receipt: `ad158b28b5aa386171cef780371621912b2c9a62a516b2e3029ae0e9748b3954`
- Build report: `5a812d064acbf72a9e2c6b16326c0e85b8b843c2c9234888e12fbcce4f9935f4`
- Reuse receipt: `9dc410c24a8276f2e0f4c09a28327222843adb28d66318b8cac53a48639b02c7`
- Audit: `780ae228c03b83ede91f530a3cdbfb1143acd6dfe5920dd209f06fe33a7bd5e3`
- Audit observation: `5c9a515d829dfa82f4b4b198ab3a4f1d4a00426c7f5124cd967d873a3be1333b`
- Fresh pull closure plan: `d88b153f61dc4b7fcdba29378edc93d275333afd719c90c01900dd32f9617570`

The first run recorded four `remote-substituted` paths. The reuse run recorded
four `local-reuse` closure paths, 1,172 `not-required-cache-only` units, and four
`already-present` runtime units.

## Audit disposition

The first live audit exposed 44 classifier false positives. Safe store suffix
normalization, store-shebang splitting, and bounded zstd inspection removed them.
Positive and negative tests cover all three changes.

One finding remains:

```text
unclassified-executable: /nix/store/6f1n9cmgsbm5s1nhjgsw8xghphrkh9vc-glibc-2.41/bin/mtrace
```

This file has executable mode but no shebang. Its shell and Perl polyglot body
invokes ambient `perl`. Mantle does not classify this as a known executable.
The audit correctly stays failed and keeps `realized` as its strongest state.

## Negative evidence

- Wrong key: `http-closure-untrusted-narinfo`
- One-member limit: `http-closure-member-limit: maximum 1`
- Tampered receipt: `foreign realization receipt digest mismatch`
- Missing member: `http-closure-missing-narinfo`

All four checks returned status 3. No failed preflight wrote a realization
receipt or exported a store path.

## Non-claims

This proof does not establish Guix evaluator parity, `guix-transfer`
correctness, original `/gnu/store` identity, direct Guix cache authentication,
local Mantle rebuild compatibility, package correctness, reproducibility,
bootstrap parity, runtime safety, deployment readiness, or release eligibility.
Future Guix, GuixPkgs, `guix-transfer`, Nixpkgs, and cache revisions remain
outside this proof.
