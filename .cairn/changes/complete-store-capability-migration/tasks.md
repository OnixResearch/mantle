# Tasks: Complete store capability migration

## Phase 1: Capability boundaries

- [ ] I1 Inventory every `StoreHandle` consumer, raw Snix service escape, and writable store authority. Record one capability owner, allowed operation set, caller list, and migration order for each use. r[store_lifecycle.capability_only_access]
- [ ] I2 Add Mantle-owned request, result, observation, and error values plus concrete capabilities for transfer objects, output admission, archives, attestations, roots, action results, and administration. Keep raw services private to `crunch-store`. r[store_lifecycle.capability_only_access]
- [ ] I3 Migrate `crunch-pipeline`, `src/store_cmd.rs`, `src/remote_build.rs`, `src/remote_transfer.rs`, and remaining cache or archive callers. Remove every external broad-handle allowlist entry after its caller moves. r[store_lifecycle.capability_only_access]
- [ ] I4 Split local output admission from publisher execution. Return an ordered bounded publication effect plan, execute it in the application shell, and record typed success or failure observations. r[store_lifecycle.publication_effect_plan]
- [ ] I5 Add a deterministic Rust architecture checker and compile-fail fixtures for broad-handle access, raw-service escape, unrelated capability use, vendor types in application ports, and writable base-store authority. r[store_lifecycle.capability_architecture_guard]
- [ ] I6 Update ADR 0058, store ownership documentation, and capability diagrams with final owners, effect boundaries, migration removal, evidence scope, and non-claims. r[store_lifecycle.capability_architecture_guard]

## Phase 2: Verification

- [ ] V1 Add positive tests for each capability. Add negative tests for unrelated authority, malformed requests, missing facts, publisher failure, wrong effect identity, stale observations, and adapter failure. r[store_lifecycle.capability_only_access] r[store_lifecycle.publication_effect_plan]
- [ ] V2 Run `nix develop -c cargo test -p crunch-store`, `nix develop -c cargo test -p crunch-pipeline`, `nix develop -c cargo test -p mantle --bin mantle store_cmd::`, and `nix develop -c cargo test -p mantle --bin mantle remote_transfer::`. Preserve exact command output in `cairn/changes/complete-store-capability-migration/evidence/focused-validation.md`. r[store_lifecycle.capability_only_access] r[store_lifecycle.publication_effect_plan]
- [ ] V3 Run the new architecture checker, its positive fixture, and every negative fixture. Prove that external raw-service and broad-handle consumer counts are zero. Preserve exact diagnostics in `cairn/changes/complete-store-capability-migration/evidence/architecture-validation.md`. r[store_lifecycle.capability_architecture_guard]
- [ ] V4 Run `nix develop -c cargo fmt --check -p crunch-store -p crunch-pipeline -p mantle`, focused first-party Clippy with `-D warnings`, `git diff --check`, `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`, `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal complete-store-capability-migration --root .`, `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design complete-store-capability-migration --root .`, `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks complete-store-capability-migration --root .`, and `nix flake check -L`. r[store_lifecycle.capability_architecture_guard]
