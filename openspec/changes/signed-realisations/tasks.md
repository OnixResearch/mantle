## Phase 1: Signing infrastructure

- [ ] Create `crates/crunch-build/src/signing.rs` with `sign_pathinfo()` pure function
- [ ] Add `signing_key: Option<SigningKey<ed25519_dalek::SigningKey>>` field to `Builder`
- [ ] Wire `sign_pathinfo()` into `persist_and_export_output` (before `put()`)
- [ ] Unit tests: sign round-trip, unsigned when no key, fingerprint matches nix-compat

## Phase 2: Verification

- [ ] Add `verify_pathinfo_signatures()` pure function in `signing.rs`
- [ ] Add `trusted_keys: Vec<VerifyingKey>` field to `Builder`
- [ ] Wire verification into remote substitution path in `check_cache`
- [ ] Add `--trust-unsigned` flag to skip verification
- [ ] Unit tests: valid sig accepted, invalid sig rejected, no trusted key = reject, trust-unsigned = accept

## Phase 3: CLI integration

- [ ] Add `--signing-key <path>` flag to `crunch build` and `crunch self-build`
- [ ] Add `--trusted-public-keys <name:key,...>` flag
- [ ] Add `crunch store sign <path>` subcommand (re-sign existing PathInfo)
- [ ] Extend `crunch store verify` to report signature status per path
- [ ] Default trusted keys include cache.nixos.org-1 public key
- [ ] Config file support: `$CRUNCH_CONFIG_DIR/signing-key`, `$CRUNCH_CONFIG_DIR/trusted-public-keys`

## Phase 4: Integration tests

- [ ] Integration test: build with signing key, verify output PathInfo has signature
- [ ] Integration test: substitute from mock cache, verify signature checked
- [ ] Integration test: unsigned remote path rejected without `--trust-unsigned`
- [ ] Integration test: `crunch store sign` adds signature to unsigned PathInfo
