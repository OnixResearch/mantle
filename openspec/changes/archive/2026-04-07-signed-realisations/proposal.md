## Why

Every PathInfo crunch persists today has `signatures: vec![]`. There is
no way for a consumer to verify that a build output actually came from a
trusted builder. This matters in three scenarios:

1. **Binary cache publishing.** When crunch serves narinfos (or writes
   them to S3/GCS), downstream Nix and crunch clients need at least one
   valid signature to accept the path. Without signatures, anything
   crunch publishes is untrusted.

2. **Multi-machine builds.** Once crunch gains remote builders or
   distributed scheduling, the coordinator must verify that results
   returned by a worker haven't been tampered with. Signatures on
   realisations are the standard mechanism for this.

3. **Local store integrity.** Bitrot, interrupted writes, bugs in crunch
   itself — any of these can corrupt redb entries silently. Verifying
   signatures on cache hits catches corruption at the trust boundary
   rather than propagating bad data into downstream builds.

Nix already defines the format: an ed25519 signature over a fingerprint
of `(store_path, nar_sha256, nar_size, references)`. The vendored
nix-compat crate has `SigningKey`, `VerifyingKey`, `Signature`, and the
fingerprint computation. snix-store has `SigningPathInfoService`, a
wrapper that signs on `put()`. None of this is wired into crunch.

## What Changes

Wire signing and verification into the build pipeline as a mandatory,
always-on feature:

1. **Key management.** Accept a signing key via `--signing-key <path>`
   (Nix-format keypair file). If no key is provided and none exists at
   `$CRUNCH_CONFIG_DIR/signing-key`, generate one automatically on first
   run. Accept trusted public keys via
   `--trusted-public-keys <name:key,...>`. The local builder's own public
   key is always implicitly trusted.

2. **Sign on build.** After `persist_and_export_output` computes the
   PathInfo, sign it before calling `PathInfoService::put()`. Every
   PathInfo gets a signature. No unsigned output.

3. **Verify on every cache hit.** Both local redb lookups and remote
   substitutions verify at least one signature against trusted keys.
   Reject paths with zero valid signatures. `--trust-unsigned` available
   as an escape hatch for migration/debugging.

4. **CLI surface.** `crunch store sign <path>` re-signs an existing
   PathInfo. `crunch store verify <path>` checks signatures against
   trusted keys (extends existing `crunch store verify`).

## Capabilities

### New Capabilities
- `sign_pathinfo`: signs a PathInfo using a local ed25519 signing key
- `verify_pathinfo_signatures`: checks signatures against trusted keys
- Auto-generated signing key on first run
- `--signing-key`, `--trusted-public-keys`, `--trust-unsigned` CLI flags
- `crunch store sign` subcommand

### Modified Capabilities
- `persist_and_export_output`: always signs PathInfo before persistence
- `check_cache` (both local and remote paths): verifies signatures
- `crunch store verify`: additionally checks signatures

## Impact

- **Files**: new `crates/crunch-build/src/signing.rs`, modified
  `orchestrate.rs`, `main.rs`, `crates/crunch-store/` (if extracted)
- **APIs**: `Builder::new()` takes a `SigningKey` (required, not optional);
  `check_cache` takes list of trusted `VerifyingKey`s
- **Dependencies**: `ed25519-dalek` already in nix-compat; no new deps
- **Testing**: unit tests with the existing `DUMMY_KEYPAIR` from
  snix-store fixtures; integration tests verifying round-trip sign+verify
- **Migration**: existing unsigned redb entries fail verification on
  first access after upgrade. `crunch store sign --all` bulk-signs them.
  `--trust-unsigned` as a temporary escape hatch.
