# Provider rerun: Mantle binary warning frontier

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]

## Command

```text
pueue task: 136
cwd: /home/brittonr/git/mantle
command:
set -euo pipefail
export PATH="/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig"
cargo build -p mantle --bin mantle
rm -rf /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26
env -u SNIX_BUILD_SANDBOX_SHELL /home/brittonr/.cargo-target/debug/mantle --json self-build \
  --cargo-free \
  --fixed-point \
  --out /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26 \
  --rustc /home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out/bin/rustc \
  --rust-source-provider /home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out \
  --toolchain-closure /home/brittonr/git/mantle/target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json \
  --target x86_64-unknown-linux-musl
```

## Result

```text
pueue_wait task 136: Done after 10m 11s
schema: mantle-cargo-free-fixed-point-proof-v1
status: blocked
fixed_point: false
bundle_dir: /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26
stage1.execution_status: blocked
stage1.status_code: 0
stage1.unit_count: 679
stage1.failed_unit_count: 1
stage1.selected_c_compiler.name: cc
stage1.selected_c_compiler.compiler_family: gcc
stage1.selected_c_compiler.content_digest_blake3: 3dccee70848def6a2838a865bceeeaffafc26a9a3249e9f7988f17cc8b14f9f2
source_built_toolchain_closure.policy_digest_blake3: c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093
```

## Parsed receipt summary

Receipt parsed from `/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26/stage1/receipt.json`.

```json
{
  "blocker_class": "rustc-failed",
  "failed_kind": "bin",
  "failed_package": "path+native#mantle@0.1.0",
  "failed_target": "crunch",
  "failed_unit_id": "native:b766e54ac96cfcf5c4b1b5f6fda714924cbc7d046bb286d74e270959753b401b:path+native#mantle@0.1.0:crunch:bin:build",
  "unit_count": 679,
  "relevant_message_lines": [
    "relocation R_X86_64_32 against `.bss.maplock' can not be used when making a PIE object; recompile with -fPIE",
    "failed to set dynamic section sizes: bad value",
    "collect2: error: ld returned 1 exit status",
    "error: aborting due to 1 previous error"
  ]
}
```

## Frontier assessment

The rerun advanced past the previous local unused-item warning blocker. It did not prove a fixed point. The current deterministic frontier is a provider musl static-PIE link failure in the root package binary build, not the earlier `BTreeSet`, `MUSL_TARGET_GCC_ALIAS`, or dormant-code warning surface.
