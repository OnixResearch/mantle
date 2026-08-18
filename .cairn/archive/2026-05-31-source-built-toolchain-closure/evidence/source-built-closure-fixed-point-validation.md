# Source-built closure fixed-point validation

Task-ID: V5
Covers: rust_package_planning.source_built_toolchain_closure

## Command

Pueue task `76` ran after the final code changes:

```sh
mantle --json self-build \
  --cargo-free \
  --fixed-point \
  --out /tmp/mantle-source-built-closure-fixed-point-target-20260601T015217Z \
  --rustc .pi/source-built-closure-proof-20260601T004426Z/rustc-source-root-target-wrapper \
  --target x86_64-unknown-linux-musl \
  --toolchain-closure .pi/source-built-closure-proof-20260601T004426Z/toolchain-closure-with-binutils.json
```

Result summary copied to `evidence/source-built-closure-fixed-point-success-summary.json`.

## Result

- `status`: `success`
- `fixed_point`: true
- bundle: `/tmp/mantle-source-built-closure-fixed-point-target-20260601T015217Z`
- manifest: `evidence/source-built-closure-toolchain-closure-with-binutils.json`
- stage1 receipt: `/tmp/mantle-source-built-closure-fixed-point-target-20260601T015217Z/stage1/receipt.json`
- stage2 receipt: `/tmp/mantle-source-built-closure-fixed-point-target-20260601T015217Z/stage2/receipt.json`
- stage1 execution status: `success`
- stage2 execution status: `success`
- stage1 units: 686
- stage2 units: 686
- stage1 failed units: 0
- stage2 failed units: 0
- stage1 Cargo guard absent marker: true
- stage2 Cargo guard absent marker: true
- stage1 smoke status: 0
- stage2 smoke status: 0
- stage1 binary digest: `16d94f103e5626e46a64fab6a8fe7600a36b12a5a92cdf225890935e007a0dd1`
- stage2 binary digest: `16d94f103e5626e46a64fab6a8fe7600a36b12a5a92cdf225890935e007a0dd1`
- stage1 policy digest: `d9743a184149959bcb6cf3ad77f6b090e80ddceb06194bc3dadd573b30e949a4`
- stage2 policy digest: `d9743a184149959bcb6cf3ad77f6b090e80ddceb06194bc3dadd573b30e949a4`
- fixed-point policy digest equality: proven by matching stage policy digests
- fixed-point binary equality: proven by matching stage binary digests

## Toolchain closure accounting

- policy digest: `d9743a184149959bcb6cf3ad77f6b090e80ddceb06194bc3dadd573b30e949a4`
- member count: 8
- source-built member count: 4
- seed exception count: 4

Source-built members include the source-root musl C toolchain entries plus the declared `ar`/`ranlib` native-helper aliases required by target C build scripts.

Seed exceptions remain explicit:

| Name | Reason | Digest |
| --- | --- | --- |
| `rustc-source-root-target-wrapper` | documented Rust compiler seed wrapping prebuilt Nix rustc 1.94.1 while target linking uses source-root musl GCC | `cd09d656a6166262636ceabe8edff99c6d03483a140f0bf3ede811e5e069e58d` |
| `nix-rust-1.94.1-sysroot` | documented Rust std/sysroot seed required until Rust itself is source-built | `2a83d326f806f58772d3d858cefcc00a290cc088404060942702ab221f21fad7` |
| `host-pkg-config` | documented pkg-config seed for native build-script compatibility | `1934fc971472324df5ec3a2873c77c6db6cb567b50b1063a93bf7453fdfaaa75` |
| `host-cc-for-host-rust-units` | documented host linker seed for proc-macro and build-script host units | `2df0ced29003a4f4156b8984e65385d08dcaf38441ea01a48b4ceb174a7dd6d7` |

## Remaining non-claims

The proof intentionally still reports:

- `not-crunch-bootstrap`
- `not-release-reproducibility`
- `not-source-built-toolchain-closure`
- `not-full-cargo-compatibility`

Reason: the target C toolchain closure is source-root materialized and receipt-bound, but Rust compiler/sysroot and host compatibility helpers remain documented seed exceptions. This is real end-to-end fixed-point evidence for the current bounded source-root target closure, not a claim that Rust itself is source-built.

## Artifact files

- Fixed-point summary: `evidence/source-built-closure-fixed-point-success-summary.json`
- Fixed-point stderr: `evidence/source-built-closure-fixed-point-success-stderr.txt`
- One-shot self-build summary: `evidence/source-built-closure-self-build-success-summary.json`
- One-shot self-build stderr: `evidence/source-built-closure-self-build-success-stderr.txt`
- Toolchain closure manifest: `evidence/source-built-closure-toolchain-closure-with-binutils.json`
