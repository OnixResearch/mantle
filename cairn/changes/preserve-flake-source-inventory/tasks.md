## Phase 1: Baseline and source filter

- [x] [serial] I1 Record the clean-commit Nix failure, focused host baseline, false-completion cases, and unrelated-work isolation. r[mantle.flake_source_inventory.self_description]
- [x] [serial] I2 Add only the exact `flake.nix` path to the shared filtered source while retaining Git-metadata exclusion. r[mantle.flake_source_inventory.self_description]
- [x] [depends:mantle.flake_source_inventory.self_description] I3 Extend focused source-inventory evidence with positive flake inclusion and negative Git-metadata exclusion assertions. r[mantle.flake_source_inventory.self_description]

## Phase 2: Verification

- [x] [parallel] V1 Run focused host tests, rustfmt, strict first-party Clippy, and Tiger Style without touching unrelated bootstrap work. r[mantle.flake_source_inventory.self_description]
- [ ] [serial] V2 Run the clean-commit Nix `nextest` and full flake checks, Cairn validation/gates, accepted-spec sync, and archive with bounded evidence. r[mantle.flake_source_inventory.self_description]
