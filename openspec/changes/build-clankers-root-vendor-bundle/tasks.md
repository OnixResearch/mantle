# Tasks: Build Clankers Root Vendor Bundle

- [ ] Design the root `clankers` fixed source/vendor closure format so the 1,169-crate vendor closure is reproducible without committing a ~1.2 GiB monolithic artifact.
- [ ] Implement the source/vendor closure representation and metadata under `packages/clankers/`.
- [ ] Add `packages/clankers/clankers.ncl` using sandbox-local `CARGO_HOME`, deterministic `CARGO_TARGET_DIR`, and offline Cargo.
- [ ] Build the root `clankers` binary with Crunch and install `$out/bin/clankers`.
- [ ] Run a network-free `$out/bin/clankers --help` or `$out/bin/clankers --version` smoke and record evidence.
- [ ] Validate with `openspec validate build-clankers-root-vendor-bundle --strict` and parent-change validation.
