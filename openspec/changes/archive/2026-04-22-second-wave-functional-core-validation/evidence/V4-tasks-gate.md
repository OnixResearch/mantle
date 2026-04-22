Evidence-ID: second-wave-functional-core-validation-v4-tasks-gate
Task-ID: V4
Artifact-Type: verification-note
Covers: architecture.nostd.core.workspace.tier.visible, architecture.nostd.core.crate.boundary.effectful.dependency.outside, functional.core.dedicated.nostd.crates.second.wave, functional.core.apis.plain.data.typed.results.normalized.request.no.ambient.reads, functional.core.shell.adapters.effect.translation.shell.activation.path.translation.in.shell, functional.core.shell.adapters.effect.translation.release.evidence.bundle.io.in.shell, functional.core.nostd.boundary.continuously.verified.regression.introduces.std.leak, portability.nostd.core.compiles.without.std.target, portability.nostd.core.dependency.allowlist.catches.std.leak
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-22

## Commands

```text
./scripts/check-no-std-core.sh   # pueue task 17
openspec_gate stage=tasks change=second-wave-functional-core-validation
```

## Results

### Full runner packet

`./scripts/check-no-std-core.sh` passed as pueue task `17`.

Packet summary:

- first-wave core tests stayed green:
  - `crunch-attestation-core` → `65 passed`
  - `crunch-project-core` → `77 passed`
- second-wave core tests stayed green:
  - `crunch-shell-core` → `17 passed`
  - `crunch-release-core` → `6 passed`
- first-wave shell-boundary tests stayed green:
  - `shell_adapter_keeps_discovery_outside_core ... ok`
  - `shell_adapter_keeps_refresh_io_outside_core ... ok`
- second-wave shell/release commands stayed green:
  - `adapter_preserves_path_order_and_appends_bin ... ok`
  - `non_utf8_with_path_is_rejected ... ok`
  - `create_and_verify_release_bundle_round_trip ... ok`
  - `load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact ... ok`
  - `release_verify_rejects_manifest_schema_mismatch ... ok`
  - `release_verify_rejects_missing_workflow_provenance ... ok`
  - `release_verify_rejects_claim_boundary_violation ... ok`
  - `release_verify_rejects_proof_linkage_source_digest_mismatch ... ok`
- checker-script commands all passed:
  - `dependency allowlist OK: ... crunch-release-core, crunch-shell-core ...`
  - `purity check OK`
  - `scope check OK`
  - `API shape check OK`
  - `ownership check OK: ... crates/crunch-shell/src/{adapter.rs,lib.rs,types.rs}, src/{release_cmd.rs,release_evidence.rs}, ...`
- deterministic wasm probes already passed in the closeout packet:
  - `bootstrap scenario OK`
  - `missing-target scenario OK`

### Tasks-stage gate packet

`openspec_gate stage=tasks change=second-wave-functional-core-validation` ended with:

```text
VERDICT: PASS
```

Gate summary:

- inventory/checker centralization traced to `I1`/`V1`
- runner/bootstrap/allowlist requirements traced to `I2`/`V1`/`V4`
- purity/scope/API-shape requirements traced to `I3`/`V1`
- first-wave shim enforcement traced to `I4`/`V1`
- ownership history union + required second-wave adapter files traced to `I5`/`V1`/`V4`
- shell and release boundaries traced to `I6`/`V2` and `I7`/`V3`

## Required evidence summary

Final packet quotes the runner transcript for the full first-wave-plus-second-wave command set, names the reviewed shell/release adapter files, includes the deterministic bootstrap-wasm and missing-wasm probe results, and shows `openspec_gate stage=tasks change=second-wave-functional-core-validation` ending in `VERDICT: PASS`.
