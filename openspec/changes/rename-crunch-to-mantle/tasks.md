## Phase 1: Identity contract

- [x] [serial] Confirm the final spelling is `Mantle`/`mantle` and keep `mantel` out of current docs, help text, and tests. Evidence: `evidence/identity-inventory.md`.
- [x] [parallel] Inventory tracked user-facing Crunch/crunch occurrences and classify each as current surface, compatibility surface, or historical archive. Evidence: `evidence/identity-inventory.md`.
- [x] [parallel] Inventory generated/release/proof artifact schemas and fixtures that carry product identity or command names. Evidence: `evidence/identity-inventory.md`.

## Phase 2: CLI and defaults

- [x] [serial] Rename the canonical package/binary/help surface to `mantle` and add/adjust tests for `mantle --help` and `mantle --version`. Evidence: root package/bin renamed to `mantle`; `tests/identity_cli.rs` covers `mantle --help` and `mantle --version`; verified with `cargo test --test identity_cli -- --nocapture`. ✅ 7m (started: 2026-05-14T17:15:08Z → completed: 2026-05-14T17:22:00Z)
- [x] [depends:CLI rename] Define the legacy `crunch` command behavior as alias, migration diagnostic, or removal, with tests for the chosen behavior. Evidence: retained `crunch` as a compatibility alias binary using the same implementation; `tests/identity_cli.rs` covers `crunch --help` reporting canonical Mantle help. ✅ 7m (started: 2026-05-14T17:15:08Z → completed: 2026-05-14T17:22:00Z)
- [x] [depends:CLI rename] Update default project-management filenames/directories to Mantle names and add migration/conflict tests. Evidence: `src/project_cmd.rs` now defaults to `mantle-project.ncl`, `mantle.lock`, and `.mantle/inputs.ncl`; conflicting legacy `crunch-project.ncl`/`crunch.lock`/`.crunch` beside current files fails with a migration diagnostic; `tests/project_cli.rs` covers Mantle defaults and conflict detection. Verified with `cargo test --test project_cli -- --nocapture` and `openspec validate rename-crunch-to-mantle --strict`. ✅ 6m (completed: 2026-05-14T17:21:28Z)
- [x] [depends:CLI rename] Update the default logical store prefix to `/mantle/store` while preserving explicit `--nix-compat` behavior and legacy `/crunch/store` compatibility tests. Evidence: CLI default `--store-prefix` is now `/mantle/store`; `tests::default_store_prefix_is_mantle` covers parsed defaults, `tests::nix_compat_overrides_store_prefix_to_nix_store` preserves `--nix-compat`, `tests::explicit_legacy_crunch_store_prefix_is_preserved` preserves explicit `/crunch/store`, and `tests/identity_cli.rs` checks help shows `/mantle/store` without stale `/crunch/store`. Verified with `cargo fmt --check`, `cargo test --bin mantle store_prefix -- --nocapture`, `cargo test --test identity_cli -- --nocapture`, and `openspec validate rename-crunch-to-mantle --strict`. ✅ 3m (started: 2026-05-14T17:29:43Z → completed: 2026-05-14T17:32:46Z)

## Phase 3: Evidence, docs, and guards

- [ ] [parallel] Update release, attestation, self-build, and parity evidence schemas/fixtures/docs to use Mantle identity without widening proof claims.
- [ ] [parallel] Update README, operator docs, bootstrap docs, scripts, examples, and OpenSpec current specs to use Mantle identity where current-facing.
- [ ] [serial] Add a deterministic stale-branding check with an allowlist for archives, compatibility tests, migration docs, and intentional external names.
- [ ] [depends:stale-branding-check] Run focused CLI/default-path/project-management/release-evidence tests plus the stale-branding check.
- [ ] [depends:verification] Commit the rename implementation with verification evidence and a clean worktree.
