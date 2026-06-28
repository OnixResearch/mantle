# Provider fixed-point path normalization proof

Date: 2026-06-27
Task-ID: provider-fixed-point-path-normalization-proof
Covers: r[verification_evidence.provider_fixed_point_path_normalization]

## Summary

Provider-backed Cargo-free fixed-point proof was rerun from the corrected source snapshot after deterministic path normalization and the build-script package-root repair.

Result: success. Stage 1 and stage 2 produced byte-identical Mantle binaries.

- Proof bundle: `/tmp/mantle-provider-fixed-point-4731da6c-fix4`
- Stage 1 binary BLAKE3: `fcf2c1329d6976a1d3377c04f20df9f3540cbcf4463a6adbdcf03f15c3e4a408`
- Stage 2 binary BLAKE3: `fcf2c1329d6976a1d3377c04f20df9f3540cbcf4463a6adbdcf03f15c3e4a408`
- `meta.json` status: `success`
- `meta.json` fixed_point: `true`
- Stage 1 execution_status: `success`, unit_count: `686`
- Stage 2 execution_status: `success`, unit_count: `686`
- Both stage receipts record `deterministic_release_paths: true` and remaps to `/mantle/release/source` and `/mantle/release/execution`.
- Compile-time provider helper path is normalized to `/mantle/release/provider/sandbox-shell`.

## Command

```text
export PATH="/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig"
export SNIX_BUILD_SANDBOX_SHELL="/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox"
out="/tmp/mantle-provider-fixed-point-4731da6c-fix4"
rm -rf "$out"
/home/brittonr/.cargo-target/debug/mantle self-build --cargo-free --fixed-point --out "$out" --rustc /home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out/bin/rustc --toolchain-closure /home/brittonr/git/mantle/target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json --rust-source-provider /home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out --target x86_64-unknown-linux-musl
```

## Output

Captured from pueue task 321:

```text
Cargo-free fixed-point: success
bundle: /tmp/mantle-provider-fixed-point-4731da6c-fix4
stage1_binary_blake3: fcf2c1329d6976a1d3377c04f20df9f3540cbcf4463a6adbdcf03f15c3e4a408
stage2_binary_blake3: fcf2c1329d6976a1d3377c04f20df9f3540cbcf4463a6adbdcf03f15c3e4a408
```

## Meta excerpt

From `/tmp/mantle-provider-fixed-point-4731da6c-fix4/meta.json`:

```text
"fixed_point": true
"status": "success"
"stage1"."binary_blake3": "fcf2c1329d6976a1d3377c04f20df9f3540cbcf4463a6adbdcf03f15c3e4a408"
"stage1"."execution_status": "success"
"stage1"."failed_unit_count": 0
"stage1"."unit_count": 686
"stage2"."binary_blake3": "fcf2c1329d6976a1d3377c04f20df9f3540cbcf4463a6adbdcf03f15c3e4a408"
"stage2"."execution_status": "success"
"stage2"."failed_unit_count": 0
"stage2"."unit_count": 686
```

## Receipt checks

Targeted grep confirmed both stage receipts contain:

```text
"deterministic_release_paths": true
"to": "/mantle/release/source"
"to": "/mantle/release/execution"
"SNIX_BUILD_SANDBOX_SHELL": "/mantle/release/provider/sandbox-shell"
```

## Notes

An earlier rerun failed to write the stage receipt because `/tmp` was full. The stale work dir `/tmp/mantle-self-hosting-current-9f3c4a71-work` was removed, freeing space before the successful proof rerun. The successful proof was then run from a fresh output bundle.
