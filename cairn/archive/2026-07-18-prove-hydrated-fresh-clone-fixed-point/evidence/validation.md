# Validation evidence: hydrated fresh-clone fixed point

Date: 2026-07-18

## Implementation identity

The implementation was committed before the admitted proof. The final fresh clone used commit `9106845c9971cdc40d0f1cb92e50a01534da9a74`.

Relevant implementation commits:

- `4a365a37` — fail-closed source policy, proof plumbing, report contract, documentation, and lifecycle package;
- `d3140c93` — exact legacy-provider archive classification for case-sensitive payloads;
- `3cd93b10` — deduplicate the evaluated closure before connected acquisition;
- `5c843943` — deterministic bounded chunks for fixed-fetch files larger than 64 MiB;
- `93d2ba53` — retain compressed tarball acquisition bytes and replay the bounded extractor plus recursive verifier offline;
- `9106845c` — satisfy first-party schema parity and Tiger Style without weakening the proof boundary.

## Focused positive and negative validation

Post-change focused checks passed:

```text
crunch-build fetcher tests: 22 passed; 0 failed
source_bundle::tests: 64 passed; 0 failed
self_build::tests: 172 passed; 0 failed
fresh_clone_fixed_point::tests: 4 passed; 0 failed
self_hosting non-expensive harness: 53 passed; 0 failed; expensive proof filtered
cargo check -p mantle --bin crunch: PASS
```

Coverage includes connected acquisition success and fixed-output mismatch rejection; duplicate URL/kind mappings; missing, stale, unpinned, and tampered records; wrong Git revision; unsupported source kinds; override lifetime; exact preservation of the three-record `fresh-clone-inputs` profile; unmatched-fetch denial before acquisition; proof-line parse/tamper rejection; bounded chunk layout positive/negative cases; malformed archive rejection; source-only stage seeding; report path-redaction; zero-live-fetch enforcement; and mismatched binary non-admission.

## Real connected source authority

Pueue task `84` evaluated `bootstrap/bwrap.ncl`, `bootstrap/busybox.ncl`, and `bootstrap/rust.ncl`, acquired every missing fixed input, replayed tarball extraction, ran fixed-output verification, and exported:

```text
format=mantle-source-bundle-v1
records=12
payload_bytes=600587433
readiness=Ready
```

The three oversized compressed acquisitions were represented by 2, 5, and 3 contiguous entries respectively; every entry remained at or below the named 64 MiB limit. The producer did not retain expanded toolchain trees as JSON payloads.

Pueue task `109` assembled those 12 records with the exact vendored-Cargo, unpacked-provider, and provider-manifest records:

```text
mode=fresh-clone-fixed-point
records=15
manifest_blake3=a293649560e4763bdfeb515ce90c17c72b3cde463b1a297a773e92349e2a45e6
provider_kind=musl.cc-native-reduced-v1
```

The manifest BLAKE3 above was retained outside the bundle and supplied independently to hydration and both proof stages.

## Final fresh-clone proof

Pueue task `387` cloned committed source `9106845c9971cdc40d0f1cb92e50a01534da9a74` with `--no-local`. Assertions confirmed the clone initially lacked `vendor-deps/`, source-only state, and prior proof output.

Pueue task `388` hydrated the 15-record profile into that clone:

```text
format=mantle-self-build-source-hydration-v1
manifest_blake3=a293649560e4763bdfeb515ce90c17c72b3cde463b1a297a773e92349e2a45e6
imported_record_count=15
existing_record_count=0
pinned=true
```

Pueue task `401` created a new empty `CARGO_HOME` and passed:

```text
CARGO_NET_OFFLINE=true cargo metadata --offline --locked --format-version 1 \
  --config .cargo/vendor-config.toml
```

Pueue task `406` then executed the complete ignored stage0 → stage1 → stage2 proof with `CARGO_NET_OFFLINE=true`, `CRUNCH_NO_FUSE=1`, the independently supplied manifest identity, hydrated source-only state, and the contracted hydration report. The captured final log reports:

```text
test self_hosting_stage0_stage1_stage2 ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 53 filtered out; finished in 3315.30s
```

The contracted report is preserved byte-for-byte as `evidence/fresh-clone-fixed-point.json`. Pueue task `567` confirmed the durable copy matches the proof-bundle report. Its admitted facts are:

```text
expected_manifest_blake3=a293649560e4763bdfeb515ce90c17c72b3cde463b1a297a773e92349e2a45e6
source_state_blake3=c57a28c660f09fd52c3f0d16a5ddbb19207ffc96eeff1e7a7a615551251bb5cc
staged_source_store_name=vgw8b44xl9c6lxw4ck8hy5vrgf2nla4z-mantle-src
stage0.source_policy=require-override
stage0.source_override_count=12
stage0.live_fetch_events=0
stage2.source_policy=require-override
stage2.source_override_count=12
stage2.live_fetch_events=0
stage1_binary_blake3=162b38afe3022d34b480621614782376926ac4c70b954a0c1a5e7cc889f3769d
stage2_binary_blake3=162b38afe3022d34b480621614782376926ac4c70b954a0c1a5e7cc889f3769d
fixed_point=true
```

Stage0 remained explicitly practical and recorded two bounded fallback events: host bwrap for the checkout transition and checkout source discovery. Stage2 was strict and recorded zero fallback events. Neither stage recorded a live fixed-source fetch.

## Quality and policy rails

- Pueue task `385`: package-scoped Rustfmt, strict first-party Clippy, and serialized first-party lib/test suites passed.
- Pueue task `377`: the complete configured first-party Tiger Style check passed after explicit foreign-type construction and FCIS/interface cleanup.
- Pueue task `378`: dependency policy reported `advisories ok, bans ok, licenses ok, sources ok`.
- Pueue tasks `380` and `407`: schema generation, adversarial checker self-test, freshness check, Rust producer parity, positive/negative contract fixtures, and machine-schema integration tests passed; the final checker reported `20 contracted, 49 classified`.
- Pueue task `384`: `nix flake check --no-build -L` evaluated every host-compatible package/check/app/dev-shell derivation and reported `all checks passed!`; incompatible non-host systems were explicitly omitted.
- `git diff --check`: passed.

A separate full-build attempt (`nix flake check -L`, task `381`) was not counted as passing evidence: the configured remote builder was unavailable and the local release-signing check requires `/run/secrets/vars/nix-signing-key/key`, which is absent on this host. This does not weaken the successful Nix evaluation claim.

## Lifecycle closeout before archive

Cairn sync executed without blockers and merged the complete requirement into `cairn/specs/bootstrap-inventory/spec.md`. The accepted requirement was re-read intact at lines 171–208. Sync receipt: `448b669f84b7f0b7acaabda4741564483f4f6aa0242ee5f3a61e41861b75139f`.

Final pre-archive validation used policy `mantle-default` / policy hash `a430c8146f2f251557945fb6f9fe894115f9774a71f504ae5f931550745a936f` and reported no findings or issues. All ten tasks were complete. Gate receipts:

```text
proposal: 41f22e44f595f0774518e3d3d08c4696999d808af9e11b75637be648752ac731 PASS
design:   2ae1459946e35124ad888afb0c72d8bc91dab7193672c43820ac1c14cb6eb68f PASS
tasks:    b67cd6f41a55e2b754f80f74f735c6d1e5f34bf9615bbee851a0d02e3953eb25 PASS
```

Tracey reported `145/145 referenced (profile mantle-default)` after evidence-backed implementation and verification links were added. No push is authorized in this session.

## Adversarial audit and claim boundary

The selected runtime-override mechanism survived the real closure test: exact kind/URL/revision matching preserved derivation identities, duplicate mappings failed closed, source-plan scratch storage outlived asynchronous builds, and both fresh stages reported the same authenticated source-state identity with zero live acquisition. Preseeded build outputs and URL rewriting remain rejected because they would bypass policy evidence or alter derivation identity.

This evidence proves one hydrated legacy-provider fixed point for the recorded source authority, committed source closure, x86_64-linux platform, and proof tool boundary. It does not prove full-source bootstrap, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.

## Archive receipt

Cairn archived the complete active package to `cairn/archive/2026-07-18-prove-hydrated-fresh-clone-fixed-point` with receipt `273acd98a15f5d7e98a4cbb1da2096d7fe06ae8b305cc2420afe7f948a6e1800` and mutation-manifest hash `eb12fff7c052b12aa3dc0be69ecad5268ebd88d18488fc7350ab467d53bae870`. The active path is absent and the accepted requirement remains in `cairn/specs/bootstrap-inventory/spec.md`.

## Exact post-archive validation transcript

```json
{
  "change_issues": [],
  "changes": 0,
  "cross_repo_dependencies": [],
  "cross_repo_evidence_issues": [],
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 25,
      "scenario_blocks": 59,
      "substantive_requirement_blocks": 25
    },
    {
      "path": "./cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/examples/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 22,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 22
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "./cairn/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 32,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 24,
      "scenario_blocks": 80,
      "substantive_requirement_blocks": 24
    },
    {
      "path": "./cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 66,
      "scenario_blocks": 93,
      "substantive_requirement_blocks": 66
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 35,
      "scenario_blocks": 109,
      "substantive_requirement_blocks": 35
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 129,
      "scenario_blocks": 447,
      "substantive_requirement_blocks": 129
    },
    {
      "path": "./cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "./cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/store-transports/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 19,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 58,
      "scenario_blocks": 164,
      "substantive_requirement_blocks": 58
    },
    {
      "path": "./cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 28,
  "substance": [],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}
```

```text
traceability coverage ok: 145/145 referenced (profile mantle-default)
```
