## Phase 1: Signing infrastructure

- [ ] Create `crates/crunch-build/src/signing.rs` with `sign_pathinfo()` pure function
- [ ] Add `signing_key: SigningKey<ed25519_dalek::SigningKey>` field to `Builder` (required, not optional)
- [ ] Wire `sign_pathinfo()` into `persist_and_export_output` (before `put()`)
- [ ] Auto-generate signing key in `main.rs` when no key file exists
- [ ] Unit tests: sign round-trip, fingerprint matches nix-compat, auto-generated key works

## Phase 2: Verification on all cache hits

- [ ] Add `verify_pathinfo_signatures()` pure function in `signing.rs`
- [ ] Add `trusted_keys: Vec<VerifyingKey>` field to `Builder`
- [ ] Implicitly trust the local builder's own public key
- [ ] Wire verification into `check_cache` for both local redb and remote paths
- [ ] Add `--trust-unsigned` flag to skip verification
- [ ] Unit tests: valid sig accepted, invalid sig rejected, no trusted key = cache miss, trust-unsigned = accept, local builder key implicitly trusted

## Phase 3: CLI integration

- [ ] Add `--signing-key <path>` flag to `crunch build` and `crunch self-build`
- [ ] Add `--trusted-public-keys <name:key,...>` flag
- [ ] Default trusted keys include `cache.nixos.org-1` public key
- [ ] Add `crunch store sign <path>` subcommand (re-sign existing PathInfo)
- [ ] Add `crunch store sign --all` for bulk migration of unsigned entries
- [ ] Extend `crunch store verify` to report signature status per path
- [ ] Config file support: `$CRUNCH_CONFIG_DIR/signing-key`, `$CRUNCH_CONFIG_DIR/trusted-public-keys`

## Phase 4: Integration tests

- [ ] Integration test: build produces signed PathInfo, re-read verifies
- [ ] Integration test: substitute from mock cache, signature verified against trusted key
- [ ] Integration test: unsigned remote path rejected without `--trust-unsigned`
- [ ] Integration test: `crunch store sign --all` signs all unsigned entries
- [ ] Integration test: corrupted signature in local redb triggers cache miss + rebuild
