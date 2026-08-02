# Live Nixpkgs hello realization proof

## Result

A fresh host Nix export selected `nixpkgs#hello`.
Mantle consumed the concrete graph with Nix absent from `PATH`.
It preserved exact `/nix/store` paths under `preserve-cache-paths-v1`.

Mantle hydrated the signed runtime closure from `cache.nixos.org`.
The ordinary scheduler then observed the selected root without a foreign builder.
A second run reused the exact local closure.
A fresh state also hydrated the closure from the realization receipt.

The bounded castore provenance audit passed with no findings.

## Producer facts

- Host tool: `nix (Nix) 2.35.0`
- Flake fingerprint: `66ff9e24a691d97e6fd607d2125f813b465b5abc017666a03ba54036ba3a11c7`
- Source NAR hash: `sha256-N66fYdUuZ9hpdM7jsQ7CUWtLJduqGDyTGCaLR62CXaQ=`
- Exported root: `/nix/store/0j6kpwz4dm9964pssxn1zf5vql05y3fl-hello-2.12.3.drv`
- Selected output: `/nix/store/nm7p8wxflggcwxfzayhysq4z6a1wg373-hello-2.12.3`
- Runtime closure members: five

The producer metadata resolves the indirect `nixpkgs` reference to an immutable store source.
It does not include a Git revision.

## Mantle identities

- Plan BLAKE3: `a848103aa01984aee50c1f9677e58439aa50804016b3ead4116d0968f7f51157`
- Import receipt BLAKE3: `55124a2d9f552e697d1b82be107011a54bbb4c7d78251ad0eccf0010631c8727`
- Empty source bundle BLAKE3: `8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777`
- Cache closure policy BLAKE3: `82fc2b6d2d215ef26921b4650f431ccb71b69b5b3478031046d74c077502a9c5`
- Route: `cache-only-preserve-v1`

## Realization

The first receipt has status `complete` and strongest state `realized`.
Its BLAKE3 is `8a015495fa1ddbca3dea2c4a743a2eaa8597e41c7d37563b3fdb9bb004b68ea2`.
All five runtime members have the `remote-substituted` disposition.

The reuse receipt has BLAKE3 `202191c1e461f6db386017670a36e8b1996dd3b88ff9594a749d86dd87366402`.
All five runtime members have the `local-reuse` disposition.
The selected root reports `already-present` after scheduler observation.
Four hydrated runtime units report `substituted`.
The remaining 951 graph units report `not-required-cache-only`.

## Audit

The audit has status `pass` and strongest state `provenance-audited`.
Its BLAKE3 is `7c8f46c16ef9b0d2514a371c8b7732ed20e0b90b8f392508b7e84bd35f3ce31b`.
Its observation BLAKE3 is `3097af20ac479116f84e06bbc51e6df29a9483fc8d4d61129bc6e53ba1bc4bce`.
The findings list is empty.

## Fresh receipt-selected pull

The fresh pull admitted all five members with zero local reuse.
The report binds closure plan BLAKE3 `c687fdf9908209f520179b8b7740eab78dbfa7a05eddd58d17f24c5d518a8c22`.

## Negative evidence

- A one-member policy failed before output mutation.
- A changed realization receipt failed its BLAKE3 check.
- An untrusted cache key rejected the root NARInfo.
- Unit tests cover missing members, bad signatures, NAR mismatch, limit exhaustion, exact reuse, and validator rejection before NAR transfer.

See `limit-preflight-stderr.txt`, `tamper-stderr.txt`, and `untrusted-stderr.txt`.

## Claim boundary

This proof establishes bounded cache-only realization, exact reuse, receipt-selected hydration, and static provenance classification.
It does not prove evaluator parity, a local build, package correctness, reproducibility, bootstrap parity, runtime safety, deployment, or release eligibility.
