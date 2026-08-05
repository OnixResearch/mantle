# Fix build spike evidence (I6, I7, I8, I9)

Date: 2026-08-04. Worktree `.pi/worktrees/add-fix-nix-producer`, branch `cairn/add-fix-nix-producer`.

Stores moved to `~/mantle-fix-producer/` after `/tmp` (datapool/tmp, 251G) hit 100% capacity mid-pull. `/tmp` had 21G free after the move.

## Source pin (I6)

`packages/fix/fix-src.ncl` pins `https://github.com/psyclyx/fix.git` at rev `fd675c2e938da6c9f444a00893b6716d66c89927` (0.3.0). Hash resolved through `mantle build --fix`: `sha256-qLTYqSdaxPNMsI59PVgOCMVZjBn655xujFeraqxyK/M=`. A rerun with the corrected hash fetched and materialized the tree.

## Toolchain admission (I7)

`packages/fix/zig-toolchain.ncl` pins the ziglang.org 0.16.0 x86_64-linux tarball (`sha256-Bgb91FvN9FDW0bCoP3tdr+5XLZqd5LKoKlzv472t6s0=`, resolved via `--fix`).

The build actually consumed signed nixpkgs cache closures (nixpkgs pin `dfd9566f82a6e1d55c30f861879186440614696e`), pulled with `mantle --nix-compat store pull --from https://cache.nixos.org --closure --trusted-public-keys cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=`:

```text
zig 0.16.0, libgit2 1.9.4 (out/lib/dev), curl 8.21.0 (out/dev),
openssl 3.6.3 (out/dev), zlib 1.3.2 (out/dev), libssh2 1.11.1,
pkg-config-wrapper 0.29.2
```

Every pull reported `skipped_untrusted=0 skipped_hash_mismatch=0`. Paths recorded in `packages/fix/nixpkgs-toolchain.ncl`.

## Sandboxed fix build (I8)

```text
mantle --nix-compat --store ~/mantle-fix-producer/nixpkgs-toolchain-store \
  --state-dir ~/mantle-fix-producer/nixpkgs-toolchain-state build packages/fix/fix.ncl
→ ~/mantle-fix-producer/nixpkgs-toolchain-store/qjj0nm512hrcnivix5fd4wcfsyffkp4s-fix-0.3.0
```

The build ran `zig build --release=fast` inside the Mantle bwrap sandbox from the pinned source. Smoke:

```text
fix eval -E '1 + 2'                          → 3
fix eval -E '{ answer = 6 * 7; }' -A answer  → 42
```

## Attestation (I9)

`mantle attest show /nix/store/qjj0nm512hrcnivix5fd4wcfsyffkp4s-fix-0.3.0` returns a persisted artifact attestation with runtime-reference edges to zig 0.16.0, glibc 2.42, and libgit2 1.9.4-lib (among others).

## Instantiation parity smoke

```text
fix instantiate /tmp/fix-toy.nix  → /nix/store/3566jfpw1q9qbl21x4srkxhqw29r70jz-fix-toy.drv
nix-instantiate /tmp/fix-toy.nix  → /nix/store/3566jfpw1q9qbl21x4srkxhqw29r70jz-fix-toy.drv
nix-store --export of both paths: byte-identical (DRV-EXPORT-IDENTICAL)
```

`toy` expression: `derivation { name = "fix-toy"; builder = "/bin/sh"; system = "x86_64-linux"; }`.

## Boundary fact recorded

`fix instantiate` writes `.drv` files through the daemon protocol (log line `[daemon] stored fix-toy.drv`). Instantiation needs a reachable daemon as a store-write transport but runs no builds. The spec's daemon scenario now rejects realization commands (`fix build`/`run`/`switch`) and requires the adapter to copy the concrete closure out and admit it through Mantle's own ATerm parsing.

## Non-claims

This spike proves build and instantiation mechanics only. It does not prove evaluator correctness, nixpkgs-scale parity for this binary, or realization readiness.
