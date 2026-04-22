# Tasks: no-std functional core

## Phase 1: Boundary inventory and scaffolding

- [ ] Inventory workspace crates into three buckets: `no_std now`, `split now`,
      and `shell only`, then record the std-only reasons for every non-core
      bucketed crate
- [ ] Add workspace members `crunch-attestation-core` and
      `crunch-project-core` with `#![no_std]` + `extern crate alloc`
- [ ] Wire Cargo manifests so the new core crates use only no-std-compatible
      dependencies and the existing std crates depend on the new core crates
- [ ] Document which APIs remain on the std-facing `crunch-attestation` and
      `crunch-project` crates versus which APIs move into the new core crates

## Phase 2: Extract `crunch-attestation-core`

- [ ] Move schema types, canonicalization, digesting, versioning, and pure
      validation/policy transforms into `crunch-attestation-core`
- [ ] Keep discovery, file loading, release-bundle scanning, and any other
      filesystem-facing logic in the std `crunch-attestation` crate
- [ ] Replace any std-only inputs in moved code with plain owned data types
      that the shell adapter constructs before calling the core
- [ ] Add positive tests for canonicalization, digest stability, and valid
      policy evaluation inside `crunch-attestation-core`
- [ ] Add negative tests for malformed attestations, duplicate nodes, invalid
      roots, oversized collections, and other failure cases inside
      `crunch-attestation-core`
- [ ] Add std-adapter tests proving file-system discovery happens outside the
      no-std core boundary

## Phase 3: Extract `crunch-project-core`

- [ ] Move manifest/lock models, versioning, merge/drift logic, refresh
      planning, outcome application, and generated-input planning into
      `crunch-project-core`
- [ ] Keep git subprocess execution, URL/file hashing I/O, tempdirs, and file
      reads/writes in std shell adapters outside the no-std core crate
- [ ] Replace any std-only error payloads or input types in moved code with
      plain data that can cross the core boundary without `std`
- [ ] Add positive tests for manifest validation, lock upgrades, refresh
      planning, stale detection, and generated-input planning inside
      `crunch-project-core`
- [ ] Add negative tests for version mismatch, invalid manifests, missing patch
      references, stale/refresh failure propagation, and other malformed inputs
      inside `crunch-project-core`
- [ ] Add std-adapter tests proving shell layers do the file/process/network
      work and pass only normalized data into the no-std core

## Phase 4: Verification and boundary hardening

- [ ] Add `cargo check -p crunch-attestation-core --target
      wasm32-unknown-unknown` to the validation path
- [ ] Add `cargo check -p crunch-project-core --target
      wasm32-unknown-unknown` to the validation path
- [ ] Add a dependency-boundary check that fails if no-std core crates pull in
      std-only runtime crates such as `tokio`, `reqwest`, `ureq`, `tempfile`,
      or `clap`
- [ ] Confirm existing std-facing crates preserve behavior while delegating pure
      transforms to the new core crates
- [ ] Record second-wave candidates (`crunch-glue`, `crunch-build`,
      `crunch-store`, `crunch-shell`) and their blocking std dependencies so
      later no-std extraction work starts from an explicit backlog

## Validation

- [ ] Run `openspec validate no-std-functional-core`
- [ ] Run no-std target checks for both new core crates
- [ ] Run focused tests for `crunch-attestation-core`, `crunch-project-core`,
      and their std adapters
- [ ] Run the dependency-boundary check and confirm it fails closed on std-only
      core dependencies
