# Rust source provider materialized proof evidence (2026-06-16)

## Scope

This records the first successful source-built Rust provider materialization, provider smoke evidence, provider-backed Cargo-free one-shot self-build, and provider-backed fixed-point proof for this change.

It does **not** claim the whole toolchain closure non-claim can be removed: the fixed-point proof still records `not-source-built-toolchain-closure` because no enforced source-built toolchain closure manifest was supplied for host linker/pkg-config/native-helper closure accounting.

## Provider materialization

Command (pueue task 22):

```text
ROOT=/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe
OUTPUT_DIR=$ROOT/provider-out
TMPDIR=$ROOT/tmp
cargo run -p mantle --bin mantle -- -v bootstrap rust-source-provider --recipe bootstrap/rust-source.ncl --output-dir "$OUTPUT_DIR"
```

Result:

```text
Materialized Rust source provider /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/provider-out
  recipe_digest_blake3: 77ff5fb4416ae590bd209c756ff02afe9eeeb11c6026139af6deb5505480b441
  metadata_path: /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/provider-out/share/mantle-rust-provider/provider.json
  metadata_digest_blake3: ea71d14c79548bddfa6da8347d994b7a0e2df706d348a4ffbe2052ec39835058
```

Transcript:

```text
target/rust-source-provider-final-rustdoc-goal-probe/transcript-2026-06-15.txt
```

Provider metadata excerpt:

```json
{
  "schema": "mantle-rust-source-provider-v1",
  "provider_id": "mantle-rust-source-provider",
  "host_triple": "x86_64-unknown-linux-gnu",
  "target_triple": "x86_64-unknown-linux-musl",
  "provenance": {
    "source_built": true,
    "uses_prebuilt_rust": false,
    "build_recipe": "rust-final-source-route"
  },
  "artifacts": [
    { "role": "rustc", "path": "bin/rustc", "content_digest_blake3": "4b95f6c1bc2f62a700b72df53aea070d041e6c1b96299f39d5def8a1dfc93c10" },
    { "role": "cargo", "path": "bin/cargo", "content_digest_blake3": "0d83e8b5b5eb76fc06afd5e757e79f651b0296846dcb8cb64f5f3c20e5f04a7c" },
    { "role": "rustdoc", "path": "bin/rustdoc", "content_digest_blake3": "4efc4de1f1659e89b650e3196e1248bc3c5f64153c6432d0349937d4550fde7b" },
    { "role": "host-rustlib", "path": "lib/rustlib/x86_64-unknown-linux-gnu/lib", "content_digest_blake3": "c37f80a7788d4d17886daf01a76000a66b5a31260a91ba04687f5c850a05bed3" },
    { "role": "target-rustlib", "path": "lib/rustlib/x86_64-unknown-linux-musl/lib", "content_digest_blake3": "bbb039a83b90b498e0f8d578969660408d5e7667534c6b6276c52f65b7e80e4d" }
  ]
}
```

## Import and smoke

Command (pueue task 16):

```text
cargo run -p mantle --bin mantle -- -v bootstrap rust-source-provider \
  --import-dir /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/provider-out \
  --output-dir /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider \
  --smoke \
  --smoke-evidence-dir /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/smoke-evidence
```

Result:

```text
Imported Rust source provider /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider
  input: /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/provider-out
  metadata_path: /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/share/mantle-rust-provider/provider.json
  metadata_digest_blake3: ea71d14c79548bddfa6da8347d994b7a0e2df706d348a4ffbe2052ec39835058
  policy_digest_blake3: b3e55c13cc59bcc3ffd709ce4486df79ddb6dcbee8763a95e0754ad3fdb94aa0
Smoked Rust source provider /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider
  rustc_path: /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/bin/rustc
  target_triple: x86_64-unknown-linux-gnu
  smoke_output_digest_blake3: 995cc66d889712963c8c9a76e984156abd333e471263f5de93b4039aea7649a2
  smoke_evidence_dir: /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/smoke-evidence
  smoke_evidence_summary: /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/smoke-evidence/smoke.json
  smoke_evidence_metadata_digest_blake3: ea71d14c79548bddfa6da8347d994b7a0e2df706d348a4ffbe2052ec39835058
  smoke_evidence_policy_digest_blake3: b3e55c13cc59bcc3ffd709ce4486df79ddb6dcbee8763a95e0754ad3fdb94aa0
```

Smoke transcript:

```text
/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/import-smoke-transcript-2026-06-15.txt
```

## Cargo-free execution fix

The provider-built Rust compiler is a source-built stable Rust 1.94.0. Mantle still uses `#![feature(register_tool)]` for tigerstyle lint namespaces, so provider-backed native topology execution must compile Mantle units with `RUSTC_BOOTSTRAP=1`, matching the existing full self-build derivation behavior.

Implemented fix:

- `src/cargo_free_self_build.rs` now sets `RUSTC_BOOTSTRAP=1` on one-shot and fixed-point rust-plan child launches.
- `src/rust_plan.rs` forwards only the allowlisted `RUSTC_BOOTSTRAP` compile env to rustc children, alongside the existing `SNIX_BUILD_SANDBOX_SHELL` allowlist.
- Negative env filtering still rejects unrelated ambient env such as `LD_PRELOAD` and secret tokens.

Prior provider-backed one-shot blocker before this fix:

```text
error[E0554]: `#![feature]` may not be used on the stable release channel
 --> /home/brittonr/git/mantle/crates/crunch-pipeline/../crunch-build/../crunch-store/src/lib.rs:1:1
  |
1 | #![feature(register_tool)]
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## Provider-backed one-shot self-build

Command (pueue task 17):

```text
/home/brittonr/.cargo-target/debug/mantle self-build \
  --cargo-free \
  --out /home/brittonr/git/mantle-rust-provider-selfbuild-one-shot-2026-06-15-rustc-bootstrap \
  --rust-source-provider /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider
```

Result:

```text
Cargo-free self-build: success
binary: /home/brittonr/git/mantle-rust-provider-selfbuild-one-shot-2026-06-15-rustc-bootstrap/mantle
binary_blake3: 4f167e8b45fce438ef821820c3e6ac0405b56db114b81ee816f0cde4cf21fda3
receipt: /home/brittonr/git/mantle-rust-provider-selfbuild-one-shot-2026-06-15-rustc-bootstrap/receipt.json
```

## Provider-backed fixed-point proof

Command (pueue task 30):

```text
/home/brittonr/.cargo-target/debug/mantle self-build \
  --cargo-free \
  --fixed-point \
  --out /home/brittonr/git/mantle-rust-provider-fixed-point-2026-06-15 \
  --rust-source-provider /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider
```

Result:

```text
Cargo-free fixed-point: success
bundle: /home/brittonr/git/mantle-rust-provider-fixed-point-2026-06-15
stage1_binary_blake3: d31cf1d3563e238aabe889758cd20634b152369791e289b59ff9487bfff7c5e0
stage2_binary_blake3: d31cf1d3563e238aabe889758cd20634b152369791e289b59ff9487bfff7c5e0
```

Fixed-point metadata excerpt (`/home/brittonr/git/mantle-rust-provider-fixed-point-2026-06-15/meta.json`):

```json
{
  "status": "success",
  "fixed_point": true,
  "blocker": null,
  "rust_source_provider": {
    "status": "validated",
    "metadata_digest_blake3": "ea71d14c79548bddfa6da8347d994b7a0e2df706d348a4ffbe2052ec39835058",
    "policy_digest_blake3": "b3e55c13cc59bcc3ffd709ce4486df79ddb6dcbee8763a95e0754ad3fdb94aa0",
    "rustc_path": "/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/bin/rustc"
  },
  "stage1": {
    "execution_status": "success",
    "failed_unit_count": 0,
    "unit_count": 686,
    "smoke_status_code": 0,
    "binary_blake3": "d31cf1d3563e238aabe889758cd20634b152369791e289b59ff9487bfff7c5e0"
  },
  "stage2": {
    "execution_status": "success",
    "failed_unit_count": 0,
    "unit_count": 686,
    "smoke_status_code": 0,
    "binary_blake3": "d31cf1d3563e238aabe889758cd20634b152369791e289b59ff9487bfff7c5e0"
  }
}
```

## Remaining non-claims

The fixed-point proof still records these non-claims:

```json
[
  "not-crunch-bootstrap",
  "not-release-reproducibility",
  "not-source-built-toolchain-closure",
  "not-full-cargo-compatibility"
]
```

The `not-source-built-toolchain-closure` non-claim remains because `source_built_toolchain_closure.status` is `not-provided` in the proof metadata. This evidence proves the Rust compiler/sysroot provider route and provider-backed fixed point; it does not yet prove an enforced source-built closure for all host linker/pkg-config/native helper inputs.

## Focused validation

Command (pueue task 15):

```text
rustfmt src/cargo_free_self_build.rs src/rust_plan.rs
rustfmt --check src/cargo_free_self_build.rs src/rust_plan.rs
git diff --check
cargo test -p mantle --bin mantle rust_topology_child_env -- --nocapture
cargo test -p mantle --bin mantle allowed_rust_topology_compile_env_rejects_unrelated_ambient_env -- --nocapture
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
```

The combined command exited successfully. The visible source-toolchain leg reported:

```text
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 734 filtered out; finished in 0.04s
```

Command (pueue task 16):

```text
cargo build -p mantle --bin mantle
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.27s
```

Post-evidence Cairn validation (pueue task 20):

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

Tasks gate (pueue task 21):

```json
{
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS",
  "issues": []
}
```

Post-archive Cairn validation (pueue task 22 after archive):

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}
```

Final validation command (pueue task 17):

```text
rustfmt --check src/cargo_free_self_build.rs src/rust_plan.rs src/rust_source_provider.rs
git diff --check
cargo test -p mantle --bin mantle rust_topology_child_env -- --nocapture
cargo test -p mantle --bin mantle allowed_rust_topology_compile_env_rejects_unrelated_ambient_env -- --nocapture
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Visible result excerpt:

```text
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 734 filtered out; finished in 0.04s

{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```
