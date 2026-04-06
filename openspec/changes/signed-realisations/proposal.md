## Why

Every PathInfo crunch persists today has `signatures: vec![]`. There is
no way for a consumer to verify that a build output actually came from a
trusted builder. This matters in two scenarios that are coming up fast:

1. **Binary cache publishing.** When crunch serves narinfos (or writes
   them to S3/GCS), downstream Nix and crunch clients need at least one
   valid signature to accept the path. Without signatures, anything
   crunch publishes is untrusted.

2. **Multi-machine builds.** Once crunch gains remote builders or
   distributed scheduling, the coordinator must verify that results
   returned by a worker haven't been tampered with. Signatures on
   realisations are the standard mechanism for this.

Nix already defines the format: an ed25519 signature over a fingerprint
of `(store_path, nar_sha256, nar_size, references)`. The vendored
nix-compat crate has `SigningKey`, `VerifyingKey`, `Signature`, and the
fingerprint computation. snix-store has `SigningPathInfoService`, a
wrapper that signs on `put()`. None of this is wired into crunch.

## What Changes

Wire signing and verification into the build pipeline:

1. **Key management.** Accept a signing key via `--signing-key <path>`
   (Nix-format keypair file, same as `nix-store --generate-binary-cache-key`
   output). Accept trusted public keys via `--trusted-public-keys <name:key,...>`
   for verification. Store defaults in `$CRUNCH_CONFIG_DIR/signing-key`
   and `$CRUNCH_CONFIG_DIR/trusted-public-keys`.

2. **Sign on build.** After `persist_and_export_output` computes the
   PathInfo, sign it before calling `PathInfoService::put()`. Use
   the nix-compat `SigningKey::sign()` over the narinfo fingerprint.

3. **Verify on substitute.** When fetching a narinfo from a remote
   cache, verify that at least one signature matches a trusted public
   key. Reject paths with zero valid signatures unless
   `--trust-unsigned` is passed.

4. **CLI surface.** `crunch store sign <path>` re-signs an existing
   PathInfo. `crunch store verify <path>` checks signatures against
   trusted keys (extends existing `crunch store verify`).

## Capabilities

### New Capabilities
- `sign_pathinfo`: signs a PathInfo using a local ed25519 signing key
- `verify_pathinfo_signatures`: checks signatures against trusted keys
- `--signing-key`, `--trusted-public-keys`, `--trust-unsigned` CLI flags
- `crunch store sign` subcommand

### Modified Capabilities
- `persist_and_export_output`: signs PathInfo before persistence
- `check_cache` (remote substitution path): verifies signatures
- `crunch store verify`: additionally checks signatures

## Impact

- **Files**: new `crates/crunch-build/src/signing.rs`, modified
  `orchestrate.rs`, `main.rs`, `crates/crunch-store/` (if extracted)
- **APIs**: `Builder::new()` gains optional `SigningKey` parameter;
  `check_cache` gains list of trusted `VerifyingKey`s
- **Dependencies**: `ed25519-dalek` already in nix-compat; no new deps
- **Testing**: unit tests with the existing `DUMMY_KEYPAIR` from
  snix-store fixtures; integration tests verifying round-trip sign+verify
