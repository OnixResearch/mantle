## Phase 1: Audit current CLI surface

- [x] Compare `README.md` against `src/main.rs` and the current subcommand set
- [x] Record every outdated default, missing command, and stale environment note that needs rewriting

Findings:
- Quick Start listed `protoc` as a build requirement (no longer needed)
- CLI section missing 8 commands: `init`, `check`, `show`, `refresh`, `list-stale`, `upgrade`, `self-build`, `store sign`
- `--store` default documented correctly but `--store-prefix` (default `/crunch/store`) not mentioned anywhere
- `--nix-compat`, `--state-dir`, `--json`, `--signing-key`, `--trusted-public-keys`, `--trust-unsigned` flags undocumented
- Requirements listed `protoc` and `Nix installation` as required — both wrong
- Custom Output Directory section said logical prefix is always `/nix/store` — wrong (default is `/crunch/store`)
- Architecture diagram showed `/nix/store/` output — should be `/crunch/store/`
- Crate Layout missing `crunch-pipeline`, `crunch-project`, `crunch-store`
- Bootstrap section only described Nix mode — `--fetch` mode undocumented
- How It Differs table said "Store paths: Compatible with Nix" and "Builder templates: None" — both outdated
- Stdlib section claimed "Extra fields (closed contract)" — contract was opened
- Exit codes section (0/1/2/3) had no backing in actual code
- No documentation for signing, trust, or project management workflows

## Phase 2: Rewrite the README

- [x] Update quick-start and CLI sections to include the current command families, including project-management commands
- [x] Update store-path documentation to explain `--store-prefix`, `--nix-compat`, and the distinction between logical and physical store locations
- [x] Update integrity and cache sections to mention signing, trusted public keys, and the unsigned-cache escape hatch
- [x] Remove or rewrite stale setup notes such as outdated tool requirements or writable-store assumptions

Changes made:
- Removed `protoc` from build requirements
- Added all 13 commands to CLI section, grouped with build/store/project/self-build flags
- New "Store Paths and Prefixes" section explaining logical vs physical
- New "Signing and Trust" section with examples
- New "Project Management" section covering init/check/show/refresh/list-stale/upgrade
- New "Self-Build" section documenting the bootstrap chain
- Updated Bootstrap with `--fetch` mode
- Updated Architecture diagram to show `/crunch/store/`
- Updated Crate Layout with `crunch-pipeline`, `crunch-project`, `crunch-store`
- Requirements split into build-time vs run-time with explicit "not required" list
- Removed fabricated exit codes
- Updated How It Differs table with store prefix and closure resolution rows
- Removed "Extra fields (closed contract)" claim from stdlib section

## Phase 3: Verify

- [x] Check the rewritten README against `crunch --help`, `crunch build --help`, `crunch self-build --help`, and `crunch store --help`
- [x] Confirm the documented examples and defaults match the current implementation before landing the change

Verification:
- All 11 commands + 4 store subcommands confirmed present via `--help`
- `--store` default: `/nix/store` ✓
- `--store-prefix` default: `/crunch/store` ✓
- `--substituters` default: `https://cache.nixos.org` ✓
- All build flags (`--fix`, `--jobs`, `--import-path`, `--substituters`, `--no-substitute`, `--signing-key`, `--trusted-public-keys`, `--trust-unsigned`) confirmed
- Self-build flags (`--no-verify`, `--no-substitute`, `--signing-key`, `--trusted-public-keys`, `--trust-unsigned`) confirmed
- `bootstrap --fetch` confirmed
- `store sign --all` confirmed
- `refresh [NAMES]...` confirmed
