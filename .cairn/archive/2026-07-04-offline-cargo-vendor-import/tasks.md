## Implementation

- [x] [serial] I1 Add normalized vendored Cargo source facts to the Cargo import functional core, including vendor root identity, package entries, lockfile source identities, checksum metadata status, and deterministic blockers. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: `src/cargo_import.rs` now models lock packages, vendor source facts, vendor package entries, checksum status, BLAKE3 identities, and blockers; unit tests passed in task 502.
- [x] [serial] I2 Teach the CLI shell to read supported `.cargo/config.toml` / `.cargo/config` directory source replacement facts without consulting Cargo, network services, or ambient Cargo caches. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: `read_vendor_source_facts` parses local directory source replacement and `cargo_import_ignores_ambient_cargo_home_vendor_material` passed in task 480/496.
- [x] [serial] I3 Validate vendored packages against `Cargo.lock` and `.cargo-checksum.json`, preserving Cargo SHA-256 metadata for interoperability and emitting Mantle BLAKE3 identities for reports. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: `validate_vendor_checksum` verifies Cargo SHA-256 file entries and records BLAKE3 directory identity; stale checksum tests passed in tasks 480 and 502.
- [x] [serial] I4 Generate `vendor_src` / `vendor_name` source inputs in `.mantle/inputs.ncl` and wire them into `mantle.offlineCargoPackage` only when vendored material is accepted. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: generated Nickel renderer includes optional `vendor_src` / `vendor_name`; `cargo_import_apply_wires_vendor_input_into_generated_project` passed in task 480/496.
- [x] [serial] I5 Update README/operator docs to describe pre-existing vendored-source import support and non-claims without suggesting that Mantle runs `cargo vendor` or fetches dependencies. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: README and `docs/operator-workflows.md` describe declared pre-existing vendor material, fail-closed blockers, ambient cache/network exclusion, and non-claims.

## Verification

- [x] [serial] V1 Positive: `mantle import cargo --plan` on a workspace with vendored registry dependencies reports a ready plan with vendor source input facts, no mutation, and stable BLAKE3 plan digests. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: `cargo_import_plan_accepts_declared_vendored_registry_source` passed in task 480/496.
- [x] [serial] V2 Positive: `mantle import cargo --apply` writes `mantle-project.ncl` and `.mantle/inputs.ncl` that pass the accepted vendor input into `mantle.offlineCargoPackage`, and the generated project builds or runs in the offline Cargo smoke rail. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: `cargo_import_apply_wires_vendor_input_into_generated_project` passed in task 480/496; `tests/offline_cargo_project.rs` passed in task 506, with the bwrap run smoke skipped because this host lacks the required bwrap `/nix/store` setup.
- [x] [serial] V3 Negative: missing vendor directory, stale `.cargo-checksum.json`, ambiguous package identity, unsupported source replacement, missing `Cargo.lock`, and registry/git dependencies without accepted vendor material block before file writes. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: unit and CLI negative tests passed in tasks 480, 502, and 506; stale checksum and ambient/no-vendor cases assert no generated files are written.
- [x] [serial] V4 Negative: a poisoned ambient `CARGO_HOME`, target directory, or network-only registry source is not consulted and does not make the plan ready. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: `cargo_import_ignores_ambient_cargo_home_vendor_material` passed in task 480/496.
- [x] [serial] V5 Run focused Cargo import/offline Cargo tests plus `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .`, proposal gate, design gate, and tasks gate for this change. r[project_workflows.cargo_import_vendored_sources]
  - Evidence: focused checks passed in tasks 480, 500, 502, and 506; validate/proposal/design gates passed in task 507. Tasks gate is recorded in `evidence/commands.txt` after final gate execution.
