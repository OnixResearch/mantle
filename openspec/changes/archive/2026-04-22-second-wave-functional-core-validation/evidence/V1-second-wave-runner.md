Evidence-ID: second-wave-functional-core-validation-v1-second-wave-runner
Task-ID: V1
Artifact-Type: verification-note
Covers: architecture.nostd.core.workspace.tier, architecture.nostd.core.workspace.tier.visible, functional.core.dedicated.nostd.crates.first.wave, functional.core.dedicated.nostd.crates.second.wave, functional.core.shell.adapters.effect.translation.project.refresh.io.in.shell, functional.core.shell.adapters.effect.translation.attestation.file.discovery.in.shell, functional.core.nostd.boundary.continuously.verified, functional.core.nostd.boundary.continuously.verified.regression.introduces.std.leak, portability.nostd.core.compiles.without.std.target, portability.nostd.core.dependency.allowlist, portability.nostd.core.dependency.allowlist.catches.std.leak
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-22

## Commands

```text
openspec validate second-wave-functional-core-validation
bash scripts/test-check-no-std-core-runner.sh bootstrap-wasm-target
bash scripts/test-check-no-std-core-runner.sh missing-wasm-target
cargo test -p crunch-shell-core --lib
cargo test -p crunch-release-core --lib
cargo test -p crunch-shell adapter_preserves_path_order_and_appends_bin -- --nocapture
cargo test -p crunch-shell non_utf8_with_path_is_rejected -- --nocapture
cargo test -p crunch --bin crunch create_and_verify_release_bundle_round_trip -- --nocapture
cargo test -p crunch --bin crunch load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact -- --nocapture
cargo test -p crunch --test release_cli release_verify_rejects_manifest_schema_mismatch -- --nocapture
cargo test -p crunch --test release_cli release_verify_rejects_missing_workflow_provenance -- --nocapture
cargo test -p crunch --test release_cli release_verify_rejects_claim_boundary_violation -- --nocapture
cargo test -p crunch --test release_cli release_verify_rejects_proof_linkage_source_digest_mismatch -- --nocapture
./scripts/check-no-std-core-deps.sh
./scripts/check-no-std-core-purity.sh
./scripts/check-no-std-core-scope.sh
./scripts/check-no-std-core-api-shape.sh
./scripts/check-no-std-core-ownership.sh
./scripts/check-no-std-core.sh   # pueue task 17
```

## Results

### Change validation + deterministic wasm probes

- `openspec validate second-wave-functional-core-validation` → `Change 'second-wave-functional-core-validation' is valid`
- `bash scripts/test-check-no-std-core-runner.sh bootstrap-wasm-target` → `bootstrap scenario OK`
- `bash scripts/test-check-no-std-core-runner.sh missing-wasm-target` → `missing-target scenario OK`

Probe meaning:

- bootstrap probe proves runner invokes `rustup target add wasm32-unknown-unknown` before `openspec`/`cargo` work when rustup is available and target is absent
- missing-target probe proves runner exits before `openspec`/`cargo` work when rustup is unavailable and wasm target is missing

### Inventory + checker surface

Files inspected:

- `openspec/specs/functional-core/validation/adopted-core-inventory.toml`
- `openspec/specs/functional-core/validation/deps-allowlist.txt`
- `openspec/specs/functional-core/evidence/ownership-review.md`
- `scripts/no_std_core_checks.py`
- `scripts/check-no-std-core.sh`
- `Cargo.toml`

Observed state:

- workspace now contains adopted four-core tier:
  - `crunch-attestation-core`
  - `crunch-project-core`
  - `crunch-shell-core`
  - `crunch-release-core`
- inventory file is the single checked-in source of truth for:
  - core crate directories
  - lib/manifests
  - required exports
  - first-wave legacy std paths
  - required second-wave std adapter files
- ownership review lists the five required second-wave std adapter files:
  - `crates/crunch-shell/src/lib.rs`
  - `crates/crunch-shell/src/adapter.rs`
  - `crates/crunch-shell/src/types.rs`
  - `src/release_evidence.rs`
  - `src/release_cmd.rs`
- ownership review also lists every currently derived touched std path outside the first-wave legacy set and records the verdict that shell/release business logic remains in `crunch-shell-core` and `crunch-release-core`

### Feature/dependency audit

- `./scripts/check-no-std-core-deps.sh` →
  `dependency allowlist OK: arrayref, arrayvec, blake3, cfg-if, constant_time_eq, crunch-attestation-core, crunch-project-core, crunch-release-core, crunch-shell-core, data-encoding, itoa, memchr, serde, serde_core, serde_json, zmij`
- direct `cargo tree -e normal,features,no-proc-macro -p crunch-shell-core --target wasm32-unknown-unknown --charset ascii` showed only `serde` (`alloc`, `derive`) plus `serde_json` (`alloc`) feature surface
- direct `cargo tree -e normal,features,no-proc-macro -p crunch-release-core --target wasm32-unknown-unknown --charset ascii` showed the same `serde`/`serde_json` no-std feature surface

### Checker scripts

- `./scripts/check-no-std-core-purity.sh` → `purity check OK`
- `./scripts/check-no-std-core-scope.sh` → `scope check OK`
- `./scripts/check-no-std-core-api-shape.sh` → `API shape check OK`
- `./scripts/check-no-std-core-ownership.sh` → `ownership check OK: ... crates/crunch-shell/src/{adapter.rs,lib.rs,types.rs}, src/{release_cmd.rs,release_evidence.rs}, ...`

### Full runner

`./scripts/check-no-std-core.sh` completed successfully as pueue task `17`.

Key transcript lines:

- `Specification 'functional-core' is valid`
- host checks:
  - `cargo check -p crunch-attestation-core` → `Finished 'dev' profile`
  - `cargo check -p crunch-project-core` → `Finished 'dev' profile`
  - `cargo check -p crunch-shell-core` → `Finished 'dev' profile`
  - `cargo check -p crunch-release-core` → `Finished 'dev' profile`
- wasm checks:
  - `cargo check -p crunch-attestation-core --target wasm32-unknown-unknown` → `Finished 'dev' profile`
  - `cargo check -p crunch-project-core --target wasm32-unknown-unknown` → `Finished 'dev' profile`
  - `cargo check -p crunch-shell-core --target wasm32-unknown-unknown` → `Finished 'dev' profile`
  - `cargo check -p crunch-release-core --target wasm32-unknown-unknown` → `Finished 'dev' profile`
- core tests:
  - `crunch-attestation-core` → `test result: ok. 65 passed; 0 failed`
  - `crunch-project-core` → `test result: ok. 77 passed; 0 failed`
  - `crunch-shell-core` → `test result: ok. 17 passed; 0 failed`
  - `crunch-release-core` → `test result: ok. 6 passed; 0 failed`
- first-wave shell boundaries:
  - `shell_adapter_keeps_discovery_outside_core ... ok`
  - `shell_adapter_keeps_refresh_io_outside_core ... ok`
- second-wave shell boundary:
  - `adapter_preserves_path_order_and_appends_bin ... ok`
  - `non_utf8_with_path_is_rejected ... ok`
- second-wave release boundary:
  - `create_and_verify_release_bundle_round_trip ... ok`
  - `load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact ... ok`
  - `release_verify_rejects_manifest_schema_mismatch ... ok`
  - `release_verify_rejects_missing_workflow_provenance ... ok`
  - `release_verify_rejects_claim_boundary_violation ... ok`
  - `release_verify_rejects_proof_linkage_source_digest_mismatch ... ok`
- checker tail:
  - `dependency allowlist OK: ... crunch-release-core, crunch-shell-core ...`
  - `purity check OK`
  - `scope check OK`
  - `API shape check OK`
  - `ownership check OK: ... src/release_cmd.rs, src/release_evidence.rs ...`

## Required evidence summary

`openspec validate` is green. Workspace manifest and crate directories visibly contain the adopted four-core tier plus std adapters. Deterministic bootstrap-target probe shows the runner invokes `rustup target add wasm32-unknown-unknown` successfully before cargo checks when rustup is available and the target is absent. Deterministic missing-target probe shows the runner exits before cargo checks when wasm support is unavailable. All first-wave and second-wave host checks pass. All first-wave and second-wave wasm checks pass. All first-wave and second-wave core and shell-boundary tests named in the spec pass. The feature/dependency audit shows all adopted cores remain inside the approved no-std closure. The five checker scripts all pass explicitly. The runner transcript still shows the attestation/project shell-boundary commands. Inspection confirms the runner/checkers now use the adopted-core inventory, the ownership review derives touched std paths from the active+archived `no-std-functional-core` history union through `HEAD`, lists the five required second-wave std adapter files plus every other derived touched std path outside the first-wave legacy set, keeps the first-wave legacy classifications, uses only `adapter-only|unrelated` for derived touched std paths and required second-wave adapter files, records the required shell/release-in-core verdict, and enforces the exact shell/release exports named in the spec.
