## Phase 1: Signing infrastructure

- [x] Create `crates/crunch-build/src/signing.rs` with `sign_pathinfo()` pure function ✅
- [x] Add `keypair: KeyPair` + `trusted_keys` + `trust_unsigned` fields to `Builder` (required, not optional) ✅
- [x] Wire `sign_pathinfo()` into `persist_and_export_output` (before `put()`) ✅
- [x] Auto-generate signing key in `build_cmd.rs::load_or_generate_keypair()` when no key file exists ✅
- [x] Unit tests: sign round-trip, fingerprint matches nix-compat, auto-generated key works (14 tests in signing.rs) ✅

## Phase 2: Verification on all cache hits

- [x] Add `verify_pathinfo_signatures()` pure function in `signing.rs` ✅
- [x] Add `trusted_keys: Vec<VerifyingKey>` field to `Builder` ✅
- [x] Implicitly trust the local builder's own public key via `build_trusted_keys()` ✅
- [x] Wire verification into `check_cache` for both local redb and remote paths ✅
- [x] Add `--trust-unsigned` flag to skip verification ✅
- [x] Unit tests: valid sig accepted, invalid sig rejected, no trusted key = cache miss, trust-unsigned = accept, local builder key implicitly trusted (in signing.rs tests) ✅

## Phase 3: CLI integration

- [x] Add `--signing-key <path>` flag to `crunch build` and `crunch self-build` ✅
- [x] Add `--trusted-public-keys <name:key,...>` flag ✅
- [x] Default trusted keys include `cache.nixos.org-1` public key ✅
- [x] Add `crunch store sign <path>` subcommand (re-sign existing PathInfo) ✅
- [x] Add `crunch store sign --all` for bulk migration of unsigned entries ✅
- [x] Extend `crunch store verify` to report signature status per path ✅
- [x] Config file support: `$CRUNCH_CONFIG_DIR/signing-key`, `$CRUNCH_CONFIG_DIR/trusted-public-keys` ✅

## Phase 4: Integration tests

- [x] Integration test: build produces signed PathInfo, re-read verifies ✅
- [x] Integration test: substitute from mock cache, signature verified against trusted key ✅
- [x] Integration test: unsigned remote path rejected without `--trust-unsigned` ✅
- [x] Integration test: `crunch store sign --all` signs all unsigned entries ✅
- [x] Integration test: corrupted signature in local redb triggers cache miss + rebuild ✅
