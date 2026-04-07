## Phase 1: Audit current CLI surface

- [ ] Compare `README.md` against `src/main.rs` and the current subcommand set
- [ ] Record every outdated default, missing command, and stale environment note that needs rewriting

## Phase 2: Rewrite the README

- [ ] Update quick-start and CLI sections to include the current command families, including project-management commands
- [ ] Update store-path documentation to explain `--store-prefix`, `--nix-compat`, and the distinction between logical and physical store locations
- [ ] Update integrity and cache sections to mention signing, trusted public keys, and the unsigned-cache escape hatch
- [ ] Remove or rewrite stale setup notes such as outdated tool requirements or writable-store assumptions

## Phase 3: Verify

- [ ] Check the rewritten README against `crunch --help`, `crunch build --help`, `crunch self-build --help`, and `crunch store --help`
- [ ] Confirm the documented examples and defaults match the current implementation before landing the change
