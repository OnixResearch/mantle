## Phase 1: Source closure and first rung

- [ ] [serial] Inventory the Clankers package/dependency graph for the first rung and confirm `clanker-message` is the smallest useful target.
- [ ] [serial] Create a fixed Clankers source/vendor closure for the first rung, including any required registry, git, path, and workspace sources.
- [ ] [depends:source-closure] Add `packages/clankers/clanker-message.ncl` or equivalent package-set entry that uses `bootstrap/rust.ncl`, offline Cargo, sandbox-local `CARGO_HOME`, and deterministic `CARGO_TARGET_DIR`.
- [ ] [depends:first-rung-derivation] Validate the first-rung derivation with `crunch eval`, extracted shell syntax check, and `git diff --check`.
- [ ] [depends:first-rung-derivation] Build the first rung with Crunch and record source/vendor closure digests, Cargo command, and output artifact path.

## Phase 2: Clankers ladder expansion

- [ ] [depends:first-rung-build] Add the next low-dependency Clankers crates one at a time, preserving offline Cargo and fixed source closure behavior.
- [ ] [depends:ladder-expansion] Introduce native build-script tool inputs only when a selected crate requires them, with a receipt explaining the observed failure and smallest added tool.
- [ ] [depends:ladder-expansion] Add the root `clankers` binary derivation and install `$out/bin/clankers`.
- [ ] [depends:root-binary] Run a network-free root binary smoke such as `$out/bin/clankers --help` or `$out/bin/clankers --version` and record the observed output.

## Phase 3: Completion

- [ ] [depends:root-smoke] Run `openspec validate build-clankers-with-crunch --strict` and the smallest relevant Crunch/Cargo regression checks before archive.
- [ ] [depends:validation] Sync and archive the OpenSpec only after the first defined success claim is actually satisfied.
