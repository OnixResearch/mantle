# Tasks: Build Clankers Root Vendor Bundle

- [x] [serial] Design the root `clankers` fixed source/vendor closure format so the 1,169-crate vendor closure is reproducible without committing a ~1.2 GiB monolithic artifact. ✅ 0m 59s (started: 2026-05-13T13:54:05Z → completed: 2026-05-13T13:55:04Z; design: external fixed `.crunch-drain/artifacts/clankers-root-bundle.tar.zst` plus committed metadata)
- [x] [depends:bundle-design] Implement the source/vendor closure representation and metadata under `packages/clankers/`. ✅ 2m 33s (started: 2026-05-13T13:55:36Z → completed: 2026-05-13T13:58:09Z; metadata: `packages/clankers/clankers-root-bundle.json`; local fixed bundle: `.crunch-drain/artifacts/clankers-root-bundle.tar.zst`; vendor dirs: 786; archive bytes: 70,169,792)
- [ ] [depends:bundle-metadata] Add `packages/clankers/clankers.ncl` using sandbox-local `CARGO_HOME`, deterministic `CARGO_TARGET_DIR`, and offline Cargo.
- [ ] [depends:root-derivation] Build the root `clankers` binary with Crunch and install `$out/bin/clankers`.
- [ ] [depends:root-build] Run a network-free `$out/bin/clankers --help` or `$out/bin/clankers --version` smoke and record evidence.
- [ ] [depends:root-smoke] Validate with `openspec validate build-clankers-root-vendor-bundle --strict` and parent-change validation.
