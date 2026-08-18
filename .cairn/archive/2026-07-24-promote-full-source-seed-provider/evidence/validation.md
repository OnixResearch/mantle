# Validation evidence: full-source provider admission

Date: 2026-07-19

## Implementation identity

The provider implementation was committed before the admitted build and external admission sequence. The committed implementation identity is `0acc862a` (`replace bootstrap delegation with a source-built provider chain`). `bootstrap/seed.ncl` still selected the legacy provider while this evidence was produced.

## Committed-source construction and closure

From committed implementation source, Mantle exported and verified the complete materialized fixed-fetch closure for `bootstrap/seed-full-toolchain.ncl`, then passed the offline source preflight before building the provider. The durable source authority is:

```text
source_closure_records=51
source_closure_payload_bytes_approx=544.67 MiB
source_closure_manifest_blake3=2bd4fb6404fd0b3ac208fb1d8f9d2e1f28aa164a48f7470cfe199f17456a70cb
readiness=Ready
missing=0
stale=0
unsupported=0
untrusted=0
```

The closure contains both pinned sbase revisions and contains no `musl.cc`, `seed-legacy.ncl`, or `/mantle/store/` record metadata. Its payload remains external; the independently supplied manifest BLAKE3 binds it to this evidence.

The committed-source provider build completed at:

```text
provider_store_name=q366f9q6n1silgmv8w0c2ypxxw4p4yw3-full-source-seed-toolchain
provider_log=m5grcxvz6kpl5z48s44hn2xgwj8d79aq-full-source-seed-toolchain.drv.log
provider_log_status=success
```

The builder's bounded runtime admission covered C and C++ static/dynamic compilation and execution, assembly/link execution, archive indexing, object inspection/transformation, CRT/libc/libgcc/libstdc++ surfaces, malformed C/C++/assembly rejection, and undefined-symbol rejection. The normalized shared runtimes retain basename `DT_NEEDED` entries and reject absolute store-path dependencies.

## Independent identity and external admission

An independent complete Unix-mode tree hash was repeated twice over the final provider with identical results:

```text
provider_tree_entries=1242
provider_file_bytes=154002038
provider_output_blake3=f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f
provider_metadata_blake3=107a4a6d7d17b7da362b4bb64830cbb4ffc64ba3ce68772aabfe05584781be23
```

`mantle bootstrap full-source-provider-admit` then observed the materialized provider and complete source bundle, matched both independently supplied BLAKE3 identities, executed all 25 positive/rejection runtime steps, and wrote the create-new report `evidence/full-source-provider-admission.json`. The admitted bytes are also preserved at the archive-stable bootstrap path `bootstrap/evidence/full-source-provider-admission.json` for selected-provider proof bundles. Its admitted facts are:

```text
schema=mantle-full-source-provider-admission-v2
status=admitted
provider_id=full-source-v1
required_tool_count=18
required_runtime_count=10
runtime_smoke_step_count=25
provider_output_blake3=f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f
source_closure_manifest_blake3=2bd4fb6404fd0b3ac208fb1d8f9d2e1f28aa164a48f7470cfe199f17456a70cb
provider_metadata_blake3=107a4a6d7d17b7da362b4bb64830cbb4ffc64ba3ce68772aabfe05584781be23
```

Admission rejects digest mismatch, incomplete or planned source records, state-pinned source metadata, legacy closure markers, TinyCC delegation, generated stubs, missing compiler internals, missing C++/runtime surfaces, unsafe symlinks, host fallback markers, malformed metadata, and rejected-command output leftovers.

## Focused validation before selector promotion

Pueue task `280`:

```text
nix develop -c cargo test -p mantle --test bootstrap_eval

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.38s
```

Pueue task `281`:

```text
nix develop -c cargo test -p mantle --bin mantle full_source_provider::tests

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1585 filtered out; finished in 0.01s
```

`git diff --check` also passed before the evidence commit.

## Selector promotion validation

After the admission evidence commit, `bootstrap/seed.ncl` was changed to a direct `seed-full.ncl` import with no environment branch or legacy fallback. `mantle bootstrap --fetch` now loads `bootstrap/seed-legacy.ncl` explicitly as a compatibility command, separate from selected self-build authority. The self-hosting proof identity defaults to `full-source`, and every full-source proof bundle copies the archive-stable admission report from `bootstrap/evidence/full-source-provider-admission.json`.

Post-selector focused results:

```text
pueue task 301: bootstrap_eval — 17 passed; 0 failed
pueue task 302: explicit legacy fetch compatibility — 3 passed; 0 failed
pueue task 303: non-expensive self-hosting harness — 55 passed; 0 failed; 1 expensive proof filtered
pueue task 291: full-source admission core — 11 passed; 0 failed
pueue task 293: fresh-clone fixed-point report core — 4 passed; 0 failed
pueue task 295: machine-schema integration — 4 passed; 0 failed
machine schema generation: PASS (21 contracted, 50 classified)
cargo fmt --check -p mantle: PASS
git diff --check: PASS
```

The bootstrap-stable admission copy is byte-identical to the create-new Cairn evidence report. The old `source-boundary-only` selector receipt was retired rather than retained with false legacy-selection claims.

The pre-proof blocker inventory self-test remained intentionally non-green: task `299` reported 25 source-marker findings after stale selector-boundary suppression was removed. Those findings were not relabeled as clean before fixed-point evidence existed.

## Authenticated selected-provider fixed point

The selected self-build source graph was regenerated after adding explicit pinned Linux 6.6 UAPI headers for bwrap and BusyBox. The final 66-record `fresh-clone-fixed-point` profile has independently supplied manifest BLAKE3:

```text
edfe4135f4573f680dfcfcd87ea6fe592c8575c41d095a1203c953cbcc5c4fa0
```

A fresh checkout at final committed implementation `dd7702f0` was hydrated from that profile after the ordinary first-party quality repairs. Pueue task `131` then ran the full authenticated offline proof with `provider_kind=full-source`, `require-override` source policy, strict later-stage hermeticity, and `CRUNCH_NO_FUSE=1`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 12112.35s
```

The proof bundle at `/home/brittonr/.cache/mantle-full-source-proof-20260723/proof-bundle-v7` retains both binaries, stage audits, the copied admission report, hydration authority, protected-exec audit, summary, and fixed-point report. The archive-stable report is `bootstrap/evidence/full-source-provider-fixed-point.json`:

```text
source_state_blake3=4142c316fae5eba69259bfa7ddeaf3f49df917c90b210e18d68f558921e930ed
stage0_source_policy=require-override
stage0_source_override_count=61
stage0_live_fetch_events=0
stage2_source_policy=require-override
stage2_source_override_count=61
stage2_live_fetch_events=0
staged_source_store_name=kgy2phj3qnhrzbn57v11h30p3qgbkgxz-mantle-src
stage1_binary_blake3=e83fc910caad3db332160b56b4eb9b35ef50600a906b3f24d35f7cbd68bba2bb
stage2_binary_blake3=e83fc910caad3db332160b56b4eb9b35ef50600a906b3f24d35f7cbd68bba2bb
fixed_point=true
```

The two fewer runtime override keys than selected source records are exact acquisition-key deduplications accepted only after payload identity matching; the proof report records the actual installed override authority. No undeclared live acquisition occurred.

After binding the exact manifest, override count, provider kind, fixed-point identity, and stage digests, the enforced blocker inventory reports `0 findings across 0 classes, 436 evidence-backed suppressions, 0 promotion claims`. Missing, stale, tampered, or unknown predecessor markers remain actionable through the negative checker fixtures.

## Final quality and lifecycle rails

The final product tree passed the ordinary first-party quality gate in pueue task `125`:

```text
[1/3] rustfmt
[2/3] clippy
[3/3] first-party workspace tests (serialized; vendored members excluded)
test result: ok. 1596 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.80s
test result: ok. 1596 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.83s
```

Focused closeout evidence:

```text
pueue task 138: bootstrap_eval — 19 passed; 0 failed
pueue task 139: full_source_provider::tests — 11 passed; 0 failed
pueue task 142: derivation_file — 8 passed; 0 failed
pueue task 145: fresh_clone_fixed_point — 4 passed; 0 failed
pueue task 140: machine_schema_contracts — 4 passed; 0 failed
pueue task 152: machine schema generation/check — PASS (21 contracted, 50 classified)
pueue task 137: blocker inventory — 0 findings, 436 evidence-backed suppressions, 0 promotion claims
pueue task 147: nix flake check --no-build -L — all checks passed
pueue task 149: Tracey — 145/145 referenced (profile mantle-default)
```

The Nix evaluation explicitly omitted incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems. It does not claim the unavailable full build rail: configured remote signing/build infrastructure remains absent on this host. `git diff --check` passed after the evidence update.

Final pre-sync Cairn validation had no issues or findings. All eight tasks were complete, and the gates passed with receipts:

```text
proposal: 377cee7c305e2900c5e2d3d13001e3b9decf11009cbec721d633ec1569cc1b63 PASS
design:   6df319e4fe4394a551eb2e3664b1e6e56d98cd790fcefde50ccf550e2c7830a1 PASS
tasks:    8b841c7c7a237e14fe71ef607695beca3557efae33a6cf3c7d372ec61e29ef95 PASS
```

Cairn sync executed with receipt `cdf40cb0d792155a824d59114c2fa8097efe0807f4034550ca095f21bde66345`. The merged accepted requirement was inspected intact at `cairn/specs/bootstrap-inventory/spec.md` lines 210–247.

Cairn archive executed to `cairn/archive/2026-07-24-promote-full-source-seed-provider` with receipt `e192e11c6589a380832bab7b3337803f1ecb4fe22c971c13b6378118a370aa6c` and mutation-manifest hash `0634852ebb2ba677efc59ecfe933c61d4c6b06970c7bd928ac61a0a40c2a2914`.

## Adversarial audit and claim boundary

Static metadata, executable bits, version output, diagnostic private overlays, source-probe roots, and downstream compiler success were not accepted as provider admission. The surviving mechanism binds a real runtime-tested provider tree and a fully materialized source closure to independent BLAKE3 identities after the implementation commit. Selection remains a separate change so a failed or incomplete candidate cannot silently fall back or become bootstrap authority.

This evidence proves the recorded provider construction, bounded runtime surfaces, source/output identity, and one authenticated stage0 → stage1 → stage2 fixed point on the recorded x86_64-linux orchestration boundary. It does not prove compiler correctness, bootstrap-seed correctness, independent rebuild agreement, release reproducibility, deployment success, or full Cargo compatibility.

## Post-archive validation transcript

Command: `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`

```text
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
      "path": "./cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 7
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
      "path": "./cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
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
  "specs_validated": 32,
  "substance": [],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}
```

Post-archive companion rails:

```text
traceability coverage ok: 145/145 referenced (profile mantle-default)
bootstrap blocker inventory: 0 findings across 0 classes, 436 evidence-backed suppressions, 0 promotion claims, enforce=true
```
