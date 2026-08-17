# Real provider fixed-point release bundle verification — 2026-06-25

Task-ID: V2
Covers: r[rust_package_planning.bundle_provider_fixed_point_release_evidence]

## Inputs

- Self-hosting proof bundle: `target/self-hosting-proof/release-provider-fixed-point-base-2026-06-25`
- Provider fixed-point proof: `/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-rerun2-2026-06-25`
- Release evidence bundle: `target/release-evidence/provider-fixed-point-release-evidence-2026-06-25`

## Self-hosting proof summary excerpt

```text
schema: mantle-self-hosting-proof-v2
proof_mode: FixedPoint
selected_provider_kind: legacy-fetch
stage1_binary: 84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd binaries/stage1-mantle
stage2_binary: 84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd binaries/stage2-mantle
stage1_equals_stage2: true
stage0_bwrap_equals_stage2_bwrap: true
stage0_busybox_equals_stage2_busybox: true
stage2_hermeticity_mode: strict
stage2_fallback_events: []
```

## Release bundle manifest artifact check

```text
-rw-r--r-- 1 brittonr brittonr 2.0K Jun 25 17:48 target/release-evidence/provider-fixed-point-release-evidence-2026-06-25/manifest.json
-rw-r--r-- 1 brittonr brittonr 4.6K Jun 25 17:48 target/release-evidence/provider-fixed-point-release-evidence-2026-06-25/proof/provider-fixed-point/meta.json
-rw-r--r-- 1 brittonr brittonr  16K Jun 25 17:48 target/release-evidence/provider-fixed-point-release-evidence-2026-06-25/proof/self-hosting/manifest.json
```

## Bundle-local provider fixed-point verification

Command:

```text
/home/brittonr/.cargo-target/debug/mantle release verify target/release-evidence/provider-fixed-point-release-evidence-2026-06-25 --require-provider-fixed-point-proof
```

Output:

```text
release evidence verified: /home/brittonr/git/mantle/target/release-evidence/provider-fixed-point-release-evidence-2026-06-25
release id: provider-fixed-point-release-evidence-2026-06-25
binaries: 1
source digest: e831802092c14e9d5cc3c53d5f40accc3ba0edff977b63e5d05678bbdae72481
stage2 digest: 84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd
proof mode: fixed-point
reproducibility: absent
deterministic release: absent
provider fixed-point proof: valid
provider fixed-point proof source: bundled
provider fixed-point proof bundle: /home/brittonr/git/mantle/target/release-evidence/provider-fixed-point-release-evidence-2026-06-25/proof/provider-fixed-point
provider fixed-point bounded evidence role: cargo-free-source-built-handoff-evidence
provider fixed-point proof artifact digest: fe2cf79289372b1231ab95cf0d7161218ce6740f80e864b137858d20c679df59
provider fixed-point stage binary digest: 288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3
provider fixed-point closure policy digest: c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093
provider fixed-point non-claim: not-crunch-bootstrap
provider fixed-point non-claim: not-release-reproducibility
provider fixed-point non-claim: not-full-cargo-compatibility
```
