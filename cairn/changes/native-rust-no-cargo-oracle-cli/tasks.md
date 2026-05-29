# Tasks

## Spec

- [ ] [serial] Add Cargo-free CLI requirement and design. r[rust_package_planning.no_cargo_oracle_cli]

## Implementation

- [ ] [serial] Add explicit Cargo-free CLI flag or subcommand. r[rust_package_planning.no_cargo_oracle_cli]
- [ ] [serial] Route Cargo-free mode through native parser, resolver, unit graph, and executor only. r[rust_package_planning.no_cargo_oracle_cli]
- [ ] [serial] Add guard/audit that rejects Cargo invocation in Cargo-free mode. r[rust_package_planning.no_cargo_oracle_cli]
- [ ] [serial] Extend JSON receipts with Cargo-free mode, compatibility class, blockers, and non-claims. r[rust_package_planning.no_cargo_oracle_cli]
- [ ] [serial] Add CLI tests with Cargo absent or replaced by a failing shim. r[rust_package_planning.no_cargo_oracle_cli]

## Verification

- [ ] [serial] Run CLI tests, failing-Cargo-shim tests, Cargo-free smoke build, and Cairn validation. r[rust_package_planning.no_cargo_oracle_cli]
