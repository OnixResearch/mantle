## Phase 1: Source closure and first rung

- [x] [serial] Inventory the Clankers package/dependency graph for the first rung and confirm `clanker-message` is the smallest useful target. ✅ 2m 4s (started: 2026-05-13T04:25:26Z → completed: 2026-05-13T04:27:30Z; evidence: `bootstrap/evidence/clankers-first-rung-inventory.json`, `cargo metadata --locked --offline`)
- [x] [serial] Create a fixed Clankers source/vendor closure for the first rung, including any required registry, git, path, and workspace sources. ✅ 3m 17s (started: 2026-05-13T04:27:30Z → completed: 2026-05-13T04:30:47Z; evidence: `packages/clankers/clanker-message-source-closure.json`, `packages/clankers/clanker-message-Cargo.lock`; verified JSON/lock consistency and `openspec validate build-clankers-with-crunch --strict`)
- [x] [depends:source-closure] Add `packages/clankers/clanker-message.ncl` or equivalent package-set entry that uses `bootstrap/rust.ncl`, offline Cargo, sandbox-local `CARGO_HOME`, and deterministic `CARGO_TARGET_DIR`. ✅ 1m 4s (started: 2026-05-13T04:32:52Z → completed: 2026-05-13T04:33:56Z; evidence: `packages/clankers/clanker-message.ncl`, `crunch eval`, extracted `/bin/sh -n`)
- [x] [depends:first-rung-derivation] Validate the first-rung derivation with `crunch eval`, extracted shell syntax check, and `git diff --check`. ✅ 25s (started: 2026-05-13T04:33:56Z → completed: 2026-05-13T04:34:21Z; evidence: eval produced `clanker-message-0.1.0` with 61 inputs and 23763-byte builder, `/bin/sh -n /tmp/clanker-message-builder.sh`, `git diff --check`, `openspec validate build-clankers-with-crunch --strict`)
- [x] [depends:first-rung-derivation] Build the first rung with Crunch and record source/vendor closure digests, Cargo command, and output artifact path. ✅ 23m 40s (started: 2026-05-13T04:34:56Z → completed: 2026-05-13T04:58:36Z; evidence: `bootstrap/evidence/clankers-first-rung-build.json`; output: `.crunch-drain/store/51q0cx0gisx9gwjmf242ink6sxwjxmi6-clanker-message-0.1.0/lib/libclanker_message.rlib`)

## Phase 2: Clankers ladder expansion

- [ ] [depends:first-rung-build] Add the next low-dependency Clankers crates one at a time, preserving offline Cargo and fixed source closure behavior.
- [ ] [depends:ladder-expansion] Introduce native build-script tool inputs only when a selected crate requires them, with a receipt explaining the observed failure and smallest added tool.
- [ ] [depends:ladder-expansion] Add the root `clankers` binary derivation and install `$out/bin/clankers`.
- [ ] [depends:root-binary] Run a network-free root binary smoke such as `$out/bin/clankers --help` or `$out/bin/clankers --version` and record the observed output.

## Phase 3: Completion

- [ ] [depends:root-smoke] Run `openspec validate build-clankers-with-crunch --strict` and the smallest relevant Crunch/Cargo regression checks before archive.
- [ ] [depends:validation] Sync and archive the OpenSpec only after the first defined success claim is actually satisfied.
