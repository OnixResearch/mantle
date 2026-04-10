## Phase 1: Fix proof helper

- [x] Treat unset `SNIX_BUILD_SANDBOX_SHELL` and `/bin/sh` as placeholders in `scripts/prove-self-hosting.sh`
- [x] Search common static-shell locations before launching the proof
- [x] Fall back to `nix-build '<nixpkgs>' -A pkgsStatic.busybox --no-out-link` when no static shell is already installed

## Phase 2: Verify

- [x] Run `./scripts/prove-self-hosting.sh --check` and confirm it reports a real static shell path
- [x] Rerun the full self-hosting proof and confirm `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out`
