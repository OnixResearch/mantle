# Provider-backed proof rerun with selected compiler receipt

Date: 2026-06-25
Change: `source-built-provider-aws-lc-memcmp-guard`
Task: V3
Requirement: `r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]`

## Command

Pueue task 22:

```text
export RUSTC_BOOTSTRAP=1
/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --fixed-point \
  --out /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-memcmp-guard-stage-receipt-2026-06-25 \
  --rustc /home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out/bin/rustc \
  --rust-source-provider /home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out \
  --toolchain-closure /home/brittonr/git/mantle/target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json \
  --target x86_64-unknown-linux-musl
```

## Outcome

The current-code provider-backed fixed-point proof is still blocked; no provider fixed-point success or release artifact is claimed from this run.

The AWS-LC memcmp-guard frontier moved: `aws-lc-sys@0.39.1` build-script metadata execution completed successfully, and the durable stage receipt records the selected receipt-bound C compiler route. The next deterministic blocker is a later `snix-build` library compile failure because `SNIX_BUILD_SANDBOX_SHELL` is not defined at compile time for vendored `snix-build` `env!(...)` sites.

Top-level proof output excerpt:

```text
"status":"blocked"
"unit_count":615
"failed_unit_count":1
"selected_c_compiler":{"role":"c-compiler","name":"cc","execution_path":"/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain/bin/x86_64-linux-musl-gcc","content_digest_blake3":"3dccee70848def6a2838a865bceeeaffafc26a9a3249e9f7988f17cc8b14f9f2","source":{"kind":"local-tree","name":"mantle-source-built-native-root","digest_blake3":"931200fdcbebb3bb6ad25212412fd71808df9a5dbed19d4efd71136611041f5e"},"build_receipt":{"kind":"external-attested-build","name":"mantle-source-built-native-root-receipt","digest_blake3":"931200fdcbebb3bb6ad25212412fd71808df9a5dbed19d4efd71136611041f5e"},"compiler_family":"gcc"}
"blocker":"stage1 blocked: topology execution status was blocked"
```

## Durable receipt extraction

Receipt path:

```text
/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-memcmp-guard-stage-receipt-2026-06-25/stage1/receipt.json
```

Steel extraction summary:

```json
{
  "aws_lc_sys_metadata_run": {
    "execution_status": "success",
    "package_id": "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1",
    "unit_id": "11:registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1:build-script-build:custom-build:build"
  },
  "failed_unit": {
    "blocker": {
      "class": "rustc-failed",
      "message": "error: environment variable `SNIX_BUILD_SANDBOX_SHELL` not defined at compile time ... vendor/snix-build/src/buildservice/oci.rs:32:37 ... vendor/snix-build/src/buildservice/bwrap.rs:33:37"
    },
    "execution_status": "failed",
    "package_id": "path+native#snix-build@0.1.0",
    "target_name": "snix-build",
    "unit_id": "native:d697393b998a100a9f3a01c2b78c01abd3b7cc424cdcc9ce535a8f5f46c28cc6:path+native#snix-build@0.1.0:snix-build:lib:build"
  },
  "metadata_run_count": 61,
  "top_level_selected_c_compiler": {
    "role": "c-compiler",
    "name": "cc",
    "execution_path": "/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain/bin/x86_64-linux-musl-gcc",
    "content_digest_blake3": "3dccee70848def6a2838a865bceeeaffafc26a9a3249e9f7988f17cc8b14f9f2",
    "compiler_family": "gcc",
    "source": {
      "kind": "local-tree",
      "name": "mantle-source-built-native-root",
      "digest_blake3": "931200fdcbebb3bb6ad25212412fd71808df9a5dbed19d4efd71136611041f5e"
    },
    "build_receipt": {
      "kind": "external-attested-build",
      "name": "mantle-source-built-native-root-receipt",
      "digest_blake3": "931200fdcbebb3bb6ad25212412fd71808df9a5dbed19d4efd71136611041f5e"
    }
  },
  "topology_status": "blocked",
  "unit_count": 615
}
```

## Non-claims

This run does not prove provider fixed-point success, release reproducibility, full Cargo compatibility, or bootstrap correctness. It only proves that the prior AWS-LC memcmp-guard frontier is no longer the current blocker and that the proof receipt records the selected source-built compiler route for that run.
