# Provider-backed fixed-point proof — native static-PIE CRT

Date: 2026-06-26

## Command

Pueue task: 162 (final-source rerun after rustfmt)

```sh
set -euo pipefail
export PATH=/home/brittonr/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
  /home/brittonr/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin/cargo \
  --config 'build.rustc-wrapper=""' build -p mantle --bin mantle
/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --fixed-point \
  --out /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-native-static-pie-crt-2026-06-26-rerun4 \
  --rustc /home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out/bin/rustc \
  --toolchain-closure /home/brittonr/git/mantle/target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json \
  --rust-source-provider /home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out \
  --target x86_64-unknown-linux-musl
```

## Bundle

```text
/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-native-static-pie-crt-2026-06-26-rerun4
```

## Result

Source: `/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-native-static-pie-crt-2026-06-26-rerun4/meta.json` and pueue task 162 output.

```text
schema: mantle-cargo-free-fixed-point-proof-v1
status: success
fixed_point: true
blocker: null
stage1.execution_status: success
stage1.unit_count: 686
stage1.failed_unit_count: 0
stage1.binary_blake3: b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3
stage2.execution_status: success
stage2.unit_count: 686
stage2.failed_unit_count: 0
stage2.binary_blake3: b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3
source_built_toolchain_closure.status: enforced-source-built
source_built_toolchain_closure.claim: true
source_built_toolchain_closure.policy_digest_blake3: c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093
rust_source_provider.status: validated
rust_source_provider.metadata_digest_blake3: 526d6decd98515e0f3f758e30c43f93ed15658923087c2eb55b3350bd07a4158
selected_c_compiler.name: cc
selected_c_compiler.compiler_family: gcc
selected_c_compiler.content_digest_blake3: 3dccee70848def6a2838a865bceeeaffafc26a9a3249e9f7988f17cc8b14f9f2
non_claims: not-crunch-bootstrap, not-release-reproducibility, not-full-cargo-compatibility
```

## Pueue output excerpt

```text
{"schema":"mantle-cargo-free-fixed-point-proof-v1","status":"success","root":"/home/brittonr/git/mantle","bundle_dir":"/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-native-static-pie-crt-2026-06-26-rerun4","fixed_point":true,..."stage1":{"execution_status":"success","unit_count":686,"failed_unit_count":0,"binary_blake3":"b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3"},"stage2":{"execution_status":"success","unit_count":686,"failed_unit_count":0,"binary_blake3":"b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3"},..."blocker":null,"non_claims":["not-crunch-bootstrap","not-release-reproducibility","not-full-cargo-compatibility"]}
```
