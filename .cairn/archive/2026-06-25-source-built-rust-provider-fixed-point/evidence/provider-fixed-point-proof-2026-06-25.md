# Provider fixed-point proof evidence — 2026-06-25

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point]

## Provider and closure inputs

- Rust provider: `target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out`.
- Native closure manifest: `target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json`.
- Closure policy digest: `c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093`.
- Closure member count: 17 total, 17 source-built, 0 seed exceptions.
- Provider metadata digest: `526d6decd98515e0f3f758e30c43f93ed15658923087c2eb55b3350bd07a4158`.
- Provider policy digest: `6636766cbc070f531db3fbcb7da40f8617d311a40d2a5141096f8c76edcabc08`.

## One-shot handoff

Command evidence: pueue task 58.

Summary file: `/home/brittonr/git/mantle-source-built-rust-provider-one-shot-rerun2-2026-06-25.stdout`.

Observed result:

- status: success
- execution_status: success
- unit_count: 686
- failed_unit_count: 0
- cargo_marker_absent: true
- smoke_status_code: 0
- binary digest: `bcf38ab3fae4a0a848fec9c2e5537ca11d438b49693f625d4cec91848d68c758`
- closure status: enforced-source-built
- closure policy digest: `c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093`

## Fixed-point handoff

Command evidence: pueue task 68.

Summary file: `/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-rerun2-2026-06-25.stdout`.

Observed result:

- status: success
- fixed_point: true
- stage1 execution_status: success
- stage2 execution_status: success
- stage1 unit_count: 686
- stage2 unit_count: 686
- stage1 failed_unit_count: 0
- stage2 failed_unit_count: 0
- stage1 cargo_marker_absent: true
- stage2 cargo_marker_absent: true
- stage1 smoke_status_code: 0
- stage2 smoke_status_code: 0
- stage1 binary digest: `288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3`
- stage2 binary digest: `288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3`
- stage1 closure policy digest: `c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093`
- stage2 closure policy digest: `c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093`
- blocker: none

## Bounded claim

This evidence proves the provider-backed Cargo-free one-shot and stage1/stage2 fixed-point handoff under the explicit source-built Rust/native closure listed above. It does not claim crunch bootstrap replacement, release reproducibility, or full Cargo compatibility.
