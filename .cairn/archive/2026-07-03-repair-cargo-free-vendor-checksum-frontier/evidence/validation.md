# Evidence: repair-cargo-free-vendor-checksum-frontier

## Scope

Task-IDs: I1, I2, I3, I4, V1, V2, V3

Covers:

- r[rust_package_planning.vendor_material_checksum_repair]
- r[rust_package_planning.vendor_source_material_drift_diagnostics]
- r[rust_package_planning.cargo_free_fixed_point_frontier_rerun]

## Implementation evidence

### I1/I2 — vendor material repaired against Cargo.lock

The known blocker was `astral-tokio-tar@0.6.3`. The checkout-local, ignored
`vendor-deps/` tree was refreshed from Cargo's own vendored output at
`/tmp/mantle-vendor-refresh` instead of editing checksum metadata by hand. The
repo intentionally tracks zero `vendor-deps/` files (`git ls-files vendor-deps | wc -l` => `0`), so this evidence records the source-tree proof input rather than force-adding a partial vendor tree.

Focused audit (pueue task 67):

```text
lock_name_version_checksum	name = "astral-tokio-tar" version = "0.6.3" source = "registry+https://github.com/rust-lang/crates.io-index" checksum = "08648fef353ab39a9d26f909ad53fc4f071be4c91853b78523f5cc3d9e5ebffd"
vendor_name_version	15:name = "astral-tokio-tar";16:version = "0.6.3";...
vendor_package_checksum	08648fef353ab39a9d26f909ad53fc4f071be4c91853b78523f5cc3d9e5ebffd
```

Adjacent source-material blockers discovered by the fast classifier were also refreshed from `/tmp/mantle-vendor-refresh`:

```text
block-buffer-0.12.1
crc-fast
crypto-common-0.2.2
digest-0.11.3
hybrid-array
itertools-0.15.0
libc
md-5-0.11.0
nix-0.31.3
object_store
quick-xml
rustls-webpki
typenum
```

### I3 — checksum drift diagnostics remain fail-closed

`src/rust_plan.rs` now records bounded digest evidence on native registry source blockers:

- `planning_path`
- `checksum_manifest_path`
- `digest_algorithm`
- `digest_subject`
- `expected_digest`
- `actual_digest`

The validator now checks Cargo's `.cargo-checksum.json` package checksum, every recorded file SHA-256, unsafe checksum paths, missing recorded files, and unrecorded extra files before admitting vendored registry source material.

Negative coverage added:

- `no_cargo_capture_reports_vendor_package_checksum_drift_with_digest_evidence`
- `no_cargo_capture_reports_vendor_file_checksum_drift_with_digest_evidence`

## Verification evidence

### V1 — fast Cargo-free classifier no longer reports source-material blockers

Receipt: `/tmp/mantle-rust-plan-after-vendor-repair2-20260703.json`

Extraction (pueue task 62):

```json
{
  "registry_ready": true,
  "registry_blockers": 0,
  "package_ready": true,
  "package_blockers": 0,
  "unit_ready": true,
  "unit_blockers": 0,
  "derivation_ready": true,
  "derivation_blockers": 0,
  "topology_status": null
}
```

### V2 — Cargo-free fixed-point proof succeeded after repair

Command (pueue task 28):

```text
cargo run -p mantle --bin mantle --quiet -- --json self-build --cargo-free --fixed-point --out /tmp/mantle-cargo-free-fixed-point-vendor-repair-20260703T204500Z --rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc --target x86_64-unknown-linux-gnu
```

Output summary (pueue task 28 log):

```text
status=0
out_dir=/tmp/mantle-cargo-free-fixed-point-vendor-repair-20260703T204500Z
stdout=/tmp/mantle-cargo-free-fixed-point-vendor-repair-20260703T204500Z.stdout
stderr=/tmp/mantle-cargo-free-fixed-point-vendor-repair-20260703T204500Z.stderr
meta.status	success
meta.fixed_point	true
stage1.execution_status	success
stage1.blocker	<none>
stage2.execution_status	success
stage2.blocker	<none>
stage1.binary_blake3	ca00cd5866b0128434f95a0e0cf63ff2e5cc947eb90b60206cb9078bfe44215d
stage2.binary_blake3	ca00cd5866b0128434f95a0e0cf63ff2e5cc947eb90b60206cb9078bfe44215d
```

Receipt extraction (pueue task 60):

```text
== stage1 ==
rust_plan.receipt_hash	f35cc306c76d14a44599ce438f420ea9bcfbc6a843d9a9a64f5edb5915e2ef6d
native_registry.digest_blake3	964dc130610257aadedbc27a24284a58560fb01086c6686f2f5839af753a0ffd
native_registry.ready	true
topology.execution_status	success
topology.unit_executions	687
== stage2 ==
rust_plan.receipt_hash	f35cc306c76d14a44599ce438f420ea9bcfbc6a843d9a9a64f5edb5915e2ef6d
native_registry.digest_blake3	964dc130610257aadedbc27a24284a58560fb01086c6686f2f5839af753a0ffd
native_registry.ready	true
topology.execution_status	success
topology.unit_executions	687
```

This is a bounded Cargo-free fixed-point success for the recorded proof mode. It remains explicitly non-claiming for Mantle bootstrap, release reproducibility, source-built toolchain closure, and full Cargo compatibility.

### V3 — final validation

Focused rust-plan tests (pueue task 80):

```text
cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=1

test result: ok. 191 passed; 0 failed; 0 ignored; 0 measured; 951 filtered out; finished in 0.22s
```

Formatting, proof-guide guard, proof-guide self-test, and whitespace check (pueue task 83):

```text
cargo fmt -p mantle --check
cargo -Zscript scripts/check-operator-proof-guide.rs
cargo -Zscript scripts/check-operator-proof-guide.rs --self-test
git diff --check

operator proof guide drift check passed
operator proof guide checker self-test passed
```

Cairn validation and gates after executed sync (pueue task 91, compact transcript in `/tmp/mantle-repair-vendor-cairn-gates-postsync-20260703`):

```json
{
  "validate": { "valid": true, "issues": 0, "specs_validated": 18, "changes": 2 },
  "proposal": { "valid": true, "verdict": "PASS", "receipt_hash": "c16ca18c6f5cb19116188cb1e014b4cf09717cfca41fa2353cd43604f5cf7d4e" },
  "design": { "valid": true, "verdict": "PASS", "receipt_hash": "5cc0d8aa4a5b5dffbef7d5d5984bbec4eaba35ee3756801e59779a331e0fb6fc" },
  "tasks": { "valid": true, "verdict": "PASS", "receipt_hash": "1ec75c57f8d8a3f7c00a67865f4529eb7cc5a0a0cf0be9c0c5618ccc8fa3af76" }
}
```

Executed sync and post-sync validation (pueue task 90):

```text
sync mutated=true receipt_hash=68f1bd275e8e3dea900182d9f3bfb57e77d0e42767b35add2cb4d74276f20f8e
validate valid=true issues=0 specs_validated=18 changes=2
requirement counts:
vendor_material_checksum_repair 1
vendor_source_material_drift_diagnostics 1
cargo_free_fixed_point_frontier_rerun 1
```

## Post-archive validation

Archive execution (pueue task 94):

```json
{
  "change": "repair-cargo-free-vendor-checksum-frontier",
  "dry_run": false,
  "blocked": null,
  "reasons": [],
  "mutated": true,
  "receipt_hash": "d66c2114dbf20c6b5128d4181b5f3eba66d02987cbf1efd61cae719cf90d754b"
}
```

Post-archive validation (pueue task 95):

```json
{
  "valid": true,
  "issues": 0,
  "specs_validated": 17,
  "changes": 1
}
```
