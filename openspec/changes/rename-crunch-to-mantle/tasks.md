## Phase 1: Identity contract

- [x] [serial] Confirm the final spelling is `Mantle`/`mantle` and keep `mantel` out of current docs, help text, and tests. Evidence: `evidence/identity-inventory.md`.
- [x] [parallel] Inventory tracked user-facing Crunch/crunch occurrences and classify each as current surface, compatibility surface, or historical archive. Evidence: `evidence/identity-inventory.md`.
- [x] [parallel] Inventory generated/release/proof artifact schemas and fixtures that carry product identity or command names. Evidence: `evidence/identity-inventory.md`.

## Phase 2: CLI and defaults

- [ ] [serial] Rename the canonical package/binary/help surface to `mantle` and add/adjust tests for `mantle --help` and `mantle --version`.
- [ ] [depends:CLI rename] Define the legacy `crunch` command behavior as alias, migration diagnostic, or removal, with tests for the chosen behavior.
- [ ] [depends:CLI rename] Update default project-management filenames/directories to Mantle names and add migration/conflict tests.
- [ ] [depends:CLI rename] Update the default logical store prefix to `/mantle/store` while preserving explicit `--nix-compat` behavior and legacy `/crunch/store` compatibility tests.

## Phase 3: Evidence, docs, and guards

- [ ] [parallel] Update release, attestation, self-build, and parity evidence schemas/fixtures/docs to use Mantle identity without widening proof claims.
- [ ] [parallel] Update README, operator docs, bootstrap docs, scripts, examples, and OpenSpec current specs to use Mantle identity where current-facing.
- [ ] [serial] Add a deterministic stale-branding check with an allowlist for archives, compatibility tests, migration docs, and intentional external names.
- [ ] [depends:stale-branding-check] Run focused CLI/default-path/project-management/release-evidence tests plus the stale-branding check.
- [ ] [depends:verification] Commit the rename implementation with verification evidence and a clean worktree.
