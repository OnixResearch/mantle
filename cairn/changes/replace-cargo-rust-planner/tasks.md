## Phase 1: Planning scaffold

- [x] [serial] Define the Cargo-oracle parity contract. r[rust_package_planning.cargo_oracle_parity]
- [x] [serial] Define explicit lockfile/source-closure ownership. r[rust_package_planning.source_closure]
- [x] [serial] Define Mantle Rust unit graph and per-unit derivation requirements. r[rust_package_planning.unit_derivation_graph]
- [x] [serial] Define build-script/proc-macro host-target separation. r[rust_package_planning.host_target_split]
- [x] [serial] Define fail-closed unsupported Cargo behavior boundaries and non-claims. r[rust_package_planning.fail_closed_boundaries]

## Phase 2: Future implementation slices

- [x] [serial] Implement a Cargo-oracle planning command that captures `cargo metadata` and `cargo build --unit-graph` into a normalized Mantle Rust plan. r[rust_package_planning.cargo_oracle_parity]
- [ ] [serial] Implement source-closure receipts for registry/git/path dependencies before build planning consumes them. r[rust_package_planning.source_closure]
- [ ] [serial] Emit initial library/binary Rust units as Mantle derivations with reviewable `rustc` arguments. r[rust_package_planning.unit_derivation_graph]
- [ ] [serial] Add build-script/proc-macro support only after host/target outputs and generated cfg/env/link metadata are represented explicitly. r[rust_package_planning.host_target_split]
- [ ] [serial] Add negative fixtures proving unsupported Cargo behavior fails closed instead of falling back to opaque Cargo builds. r[rust_package_planning.fail_closed_boundaries]
