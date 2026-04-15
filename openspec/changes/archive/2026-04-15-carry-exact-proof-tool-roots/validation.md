# Validation evidence

## Command transcripts

- `cargo test -p crunch --bin crunch self_build::tests -- --nocapture`
  - transcript: `openspec/changes/carry-exact-proof-tool-roots/evidence/self-build-tests.txt`
  - key result: `test result: ok. 78 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out`
- `cargo test -p crunch --bin crunch resolve_explicit_bootstrap_bwrap_source_ignores_stale_sibling -- --nocapture`
  - transcript: `openspec/changes/carry-exact-proof-tool-roots/evidence/stale-sibling-unit.txt`
  - key result: `test self_build::tests::resolve_explicit_bootstrap_bwrap_source_ignores_stale_sibling ... ok`
- `cargo test -p crunch --test self_hosting prove_self_hosting_script_exports_strict_later_stage_hermeticity_by_default -- --nocapture`
  - transcript: `openspec/changes/carry-exact-proof-tool-roots/evidence/exports-strict-default.txt`
  - key result: `test prove_self_hosting_script_exports_strict_later_stage_hermeticity_by_default ... ok`
- `cargo test -p crunch --test self_hosting write_proof_bundle_copies_stage_artifacts_and_manifest -- --nocapture`
  - transcript: `openspec/changes/carry-exact-proof-tool-roots/evidence/write-proof-bundle.txt`
  - key result: `test write_proof_bundle_copies_stage_artifacts_and_manifest ... ok`
- `CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE=strict SNIX_BUILD_SANDBOX_SHELL=target/proof-busybox-static/bin/busybox ./scripts/prove-self-hosting.sh --bundle-dir target/self-hosting-proof/strict-hermeticity-check`
  - transcript: `openspec/changes/carry-exact-proof-tool-roots/evidence/strict-proof-output.txt`
  - summary artifact: `openspec/changes/carry-exact-proof-tool-roots/evidence/strict-proof-summary.txt`
  - key result: `test self_hosting_stage0_stage1_stage2 ... ok`
- `openspec validate carry-exact-proof-tool-roots`
  - transcript: `openspec/changes/carry-exact-proof-tool-roots/evidence/openspec-validate.txt`
  - key result: `Change 'carry-exact-proof-tool-roots' is valid`

## Summary artifact checks

From `openspec/changes/carry-exact-proof-tool-roots/evidence/strict-proof-summary.txt`:

- stale sibling entries are present:
  - `store_bwrap_entries: ["00000000000000000000000000000000-stale-bwrap", "pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap"]`
  - `store_busybox_entries: ["00000000000000000000000000000000-stale-busybox", "7zf934zcyvfz7wg4xf82j97qrvdiaqax-busybox"]`
- later-stage exact-root reuse is recorded:
  - `stage0_bwrap: ... /pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap`
  - `stage2_bwrap: ... /pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap`
  - `stage0_busybox: ... /7zf934zcyvfz7wg4xf82j97qrvdiaqax-busybox/bin/busybox`
  - `stage2_busybox: ... /7zf934zcyvfz7wg4xf82j97qrvdiaqax-busybox/bin/busybox`
- equality checks pass:
  - `stage0_bwrap_equals_stage2_bwrap: true`
  - `stage0_busybox_equals_stage2_busybox: true`
- later-stage strict mode and zero fallback are recorded:
  - `stage2_hermeticity_mode: strict`
  - `stage2_fallback_events: []`
  - `stage2_report: crunch-built:/home/brittonr/git/crunch/crunch/target/self-hosting-proof/work/tmp/.tmpT0sgQj/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin`

## Code evidence

- hidden exact-root handoff args:
  - `src/main.rs:233-237`
  - `src/main.rs:815-868`
- exact-path validation and reuse plumbing:
  - `src/self_build.rs:1173-1281`
  - `src/self_build.rs:1584-1608`
  - `src/self_build.rs:1661-1725`
  - `src/self_build.rs:1779-1827`
- stale-sibling unit coverage:
  - `src/self_build.rs:2268-2330`
- end-to-end stale-sibling proof assertion:
  - `tests/self_hosting.rs:2788-2827`
  - `tests/self_hosting.rs:2991-3036`
