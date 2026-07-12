# Validation evidence

Recorded: 2026-07-12

## Focused positive and negative tests

All commands used isolated Cargo target and temp roots under `/tmp`.

```text
$ cargo test -q -p crunch-release-core
184 passed; 0 failed

$ cargo test -q -p crunch-bootstrap-core
70 passed; 0 failed

$ cargo test -q -p mantle --bin mantle cairn_handoff
3 passed; 0 failed

$ cargo test -q -p mantle --bin mantle onix_profile_
2 passed; 0 failed

$ cargo test -q -p mantle --bin mantle source_root_provider
23 passed; 0 failed

$ cargo test -q -p mantle --bin mantle bootstrap_source_root::
25 passed; 0 failed

$ cargo test -q -p mantle --bin mantle bootstrap_capabilities_is_discoverable
1 passed; 0 failed

$ cargo test -q -p mantle --bin mantle flake_check_workflow_has_one
1 passed; 0 failed

$ cargo test -q -p mantle --test release_cli cairn_handoff
2 passed; 0 failed

$ cargo test -q -p mantle --test release_cli onix_release_profile
3 passed; 0 failed
```

The covered negative cases include fabricated declared digests, tampered
bundle-local bytes, wrong role/schema pairs, empty required handoffs, duplicate
artifact/policy paths, receipt reuse after a release-manifest projection change,
authenticated-status promotion, missing Onix handoff evidence, and missing
strict deterministic evidence. Existing deterministic-core coverage in the
184-test release-core suite rejects impure mode, degraded audits, failed
isolation, missing perturbations/normalization controls, and output mismatch.

Changed-file formatting check:

```text
$ rustfmt --check --edition 2024 --config skip_children=true <changed Rust files>
PASS
```

## Production capability report

```text
$ mantle --json bootstrap capabilities
{
  "schema": "mantle-source-root-capability-report-v1",
  "observations": {
    "platform_supported": true,
    "host_c_compiler_available": true,
    "host_make_available": true,
    "host_tar_available": true
  },
  "operations": [
    {
      "operation": "bootstrap-source-root-materialization",
      "status": "supported-host-assisted",
      "capability_class": "host-assisted-source-materialization",
      "command": ["mantle", "bootstrap", "--source-root", "<source-root-manifest.json>"]
    },
    {
      "operation": "self-build-source-root",
      "status": "unsupported",
      "command": null,
      "blockers": ["Mantle self-build has no executable full-source source-root provider chain"]
    }
  ]
}
```

Both operation records carry host-influence notes and the non-claim that this is
not a full-source bootstrap.

## Cairn lifecycle validation

```text
$ cairn validate --root .
valid: true

$ cairn gate proposal enforce-hermetic-release-handoff --root .
verdict: PASS
receipt_hash: e078ab5a22e12f569a43064ced32485d8d4fa4366a47887ac130ce2bc9cfbf30

$ cairn gate design enforce-hermetic-release-handoff --root .
verdict: PASS
receipt_hash: e7f17802fa8ee717ceb98ea034869e8933430d56448c53a3dc829e3e66724f7c

$ cairn gate tasks enforce-hermetic-release-handoff --root .
verdict: PASS
receipt_hash: 46c22d28052eddc8218cca7e274307d9ae822f529631ad421a28793da308e9af
```

No sync or archive command was run.

## Main-branch integration checkpoint

After integration with atomic release publication and content-bound rebuild authority, pueue task `341` successfully reran `crunch-release-core`, `crunch-bootstrap-core`, focused Cairn handoff and release-evidence binary tests, Cairn handoff CLI tests, and all 13 `release_reproduce_` tests in one isolated target. The merge-specific handoff path now measures the planned artifact/policy bytes before staging, includes those files in the pure atomic publication plan, remeasures while assembling, and verifies the staged receipt again before no-clobber commit. This closes unplanned-artifact and post-plan replacement seams without claiming external Cairn authentication. Pueue task `400` then passed Cairn validation and proposal/design/tasks gates with no issues; the external accepted-authentication dependency and final production smoke remain explicitly incomplete.

## Required flake-check attempt

```text
$ nix flake check
FAIL (exit 1)
```

The final attempt evaluated the flake and began 404 checks, then the existing
`checks.x86_64-linux.bootstrap-blocker-inventory` rail failed with:

```text
bootstrap blocker inventory: 40 findings across 4 classes,
396 evidence-backed suppressions, 0 promotion claims, enforce=true
FAIL: bootstrap blocker inventory is not clean; expected 0 findings and 0 promotion claims
```

An earlier attempt in the same session reached the repository-wide format rail
and reported pre-existing drift in untouched paths including
`crates/crunch-pipeline/src/lib.rs`, `crates/crunch-store/src/layer.rs`, several
`tests/*_offline_rail.rs` files, and
`vendor/snix-castore/src/blobservice/combinator.rs`. None of those paths is part
of this change. The focused changed-file formatting check passes.

This is exact closeout evidence, not a passing flake-check claim. The final
lifecycle task remains unchecked.

## External authentication blocker

See `external-authentication-blocker.md`. Cairn's
`authenticate-stack-provenance-inputs` remains active, metadata-blocked, 0/13
complete, and absent from Cairn's archive. Mantle therefore records and accepts
only `authentication_status: "not-authenticated"`; authenticated provenance is
not claimed.
