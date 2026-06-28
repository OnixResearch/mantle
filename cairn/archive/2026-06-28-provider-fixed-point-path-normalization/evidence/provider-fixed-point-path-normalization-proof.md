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

## 2026-06-28 provider C-toolchain remap/order rerun

The first provider path-normalized release still drifted during witness replay because native C objects embedded the receipt-bound C compiler path through a broader `/mantle/release/source/.pi/...` remap. The corrected implementation adds a receipt-bound C compiler toolchain remap to `/mantle/release/provider/c-toolchain` and emits GCC prefix-map flags in less-specific-first order so the specific C toolchain remap wins.

Focused validation passed before the final proof rerun:

```text
cargo fmt -p mantle --check
cargo test -p mantle --bin mantle deterministic_release_paths -- --nocapture
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 882 filtered out
cargo test -p mantle --bin mantle c_prefix_map -- --nocapture
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out
cargo test -p mantle --bin mantle rust_plan -- --test-threads=1 --nocapture
  test result: ok. 177 passed; 0 failed; 0 ignored; 0 measured; 706 filtered out
cargo test -p mantle --bin mantle cargo_free -- --test-threads=1 --nocapture
  test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 828 filtered out
```

Corrected provider fixed-point proof:

```text
bundle: /home/brittonr/.cargo-target/repo-targets/mantle/provider-fixed-point-provider-remap-order-fix
stage1_binary_blake3: aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20
stage2_binary_blake3: aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20
status: success
fixed_point: true
stage unit counts: 686 / 686
```

Representative native object inspection after the rerun showed the C toolchain header path normalized to the provider namespace and no `.pi` provider path leakage:

```text
provider_prefix_present True
source_pi_absent True
home_pi_absent True
.debug_line path includes /mantle/release/provider/c-toolchain/x86_64-linux-musl/include/bits
```

Same-source self-hosting proof used for the final release bundle:

```text
bundle: /home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-provider-remap-order-fix-proof
protected_exec_result: success
protected_exec_audit: 52fcb3bbbbbe298028ee960eb0bdc84eb16df24fa73a52a1fbf75ffcc4465ade protected-exec-audit.json
stage0_inventory_doc: 3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb stage0-prerequisites/inventory.md
```

Fresh corrected release bundle and verifier output:

```text
release id: provider-bound-release-evidence-2026-06-28-provider-remap-fixed
bundle: target/release-evidence/provider-bound-release-evidence-2026-06-28-provider-remap-fixed
source archive digest: 314d55875cbe8897fb3b4cee3897ea78908dfc7aa821bcad49e7dcaa08ca23dc
provider fixed-point proof artifact digest: 567ca3015081f31c28b738db55668eafdb4d8b073d42736d9c1477603ed14d27
provider fixed-point stage binary digest: aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20
provider fixed-point release artifact: binaries/01-mantle
provider fixed-point release artifact digest: aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20
self-hosting release artifact: binaries/02-stage2-mantle
self-hosting release artifact digest: 70f02150224073af2dfdabad5b697072a6399c361c8b448f0e17ee01431fc703
release verify --require-provider-fixed-point-proof: provider fixed-point proof: valid
```

Provider-bound witness replay then succeeded from the exported request using the same source-built provider closure:

```text
pueue task: 88
scratch root: /home/brittonr/.cargo-target/repo-targets/mantle/witness-provider-bound-provider-remap-fixed
status: success
witness attestation: /home/brittonr/.cargo-target/repo-targets/mantle/witness-provider-bound-provider-remap-fixed/release-verification/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/witnesses/britton-desktop-provider-witness.json
rebuilt outputs:
  binaries/01-mantle: aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20
  binaries/02-stage2-mantle: 70f02150224073af2dfdabad5b697072a6399c361c8b448f0e17ee01431fc703
```

Final release verification after importing the witness sidecar was captured at `target/release-verification/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/final-release-verify.json`:

```json
{
  "release_attestation_digest": "c16aad67e2981a178e739c52dc07617c90976e2c8d2a566659e008bdf9542880",
  "release_signer_key_name": "crunch-britton-desktop-1",
  "discovered_witness_count": 1,
  "considered_witness_count": 1,
  "technical_class": "external-witness-match",
  "policy_status": "satisfied",
  "final_class": "quorum-satisfied",
  "matching_witness_count": 1,
  "independent_witness_identities": 1,
  "revoked_witness_count": 0,
  "independent_agreement_status": "satisfied",
  "independent_agreement_class": "independent-rebuild-agreement",
  "independent_agreement_counted_witness_count": 1,
  "independent_agreement_skipped_witness_count": 0,
  "independent_agreement_failed_witness_count": 0,
  "independent_agreement_witnesses": [
    {
      "witness_identity": "britton-desktop-provider-witness",
      "signer_key_name": "crunch-britton-desktop-1",
      "signature_valid": true,
      "digest_match": true,
      "independence_domain": "britton-desktop-provider-witness",
      "policy_counted": true,
      "classification_reason": "counted"
    }
  ]
}
```
