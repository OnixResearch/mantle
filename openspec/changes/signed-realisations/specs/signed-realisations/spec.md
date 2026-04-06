# Signed Realisations Specification

## Purpose

Defines how crunch signs build outputs and verifies signatures on all
cache hits, providing integrity guarantees for binary caches, remote
builders, and local store consistency.

## Requirements

### Requirement: Always sign PathInfo after build

The system MUST sign each PathInfo with an ed25519 key before persisting
it. The signature MUST cover the canonical narinfo fingerprint:
`1;<store_path>;sha256:<nar_hash_nix32>;<nar_size>;<references>`.

There is no unsigned mode. Every persisted PathInfo has at least one
signature.

#### Scenario: Build produces signed PathInfo

- GIVEN a signing key is configured (explicitly or auto-generated)
- WHEN a derivation is built successfully
- THEN the persisted PathInfo contains exactly one signature
- AND the signature verifies against the corresponding public key

### Requirement: Auto-generate signing key on first run

If no signing key is configured via `--signing-key` and no key exists at
`$CRUNCH_CONFIG_DIR/signing-key`, the system MUST generate a new ed25519
keypair, write it to that path with 0600 permissions, and use it.

The key name MUST be `crunch-<hostname>-1`.

#### Scenario: First run auto-generates key

- GIVEN no `--signing-key` flag
- AND `$CRUNCH_CONFIG_DIR/signing-key` does not exist
- WHEN `crunch build` is invoked
- THEN a new keypair is written to `$CRUNCH_CONFIG_DIR/signing-key`
- AND the build proceeds with that key

#### Scenario: Explicit key overrides auto-generation

- GIVEN `--signing-key /path/to/my-key`
- WHEN `crunch build` is invoked
- THEN the provided key is used
- AND no auto-generation occurs

### Requirement: Nix-compatible key format

The system MUST accept signing keys in the Nix keypair format:
`<name>:<base64(secret_key ++ public_key)>` (one line, 64 bytes
decoded). This is the format produced by
`nix-store --generate-binary-cache-key`.

Trusted public keys MUST use the Nix format:
`<name>:<base64(public_key)>` (32 bytes decoded).

#### Scenario: Parse Nix keypair

- GIVEN a file containing `my-cache-1:<base64-64-bytes>`
- WHEN `--signing-key` points to this file
- THEN the signing key is loaded and used for signing

#### Scenario: Reject malformed key

- GIVEN a file containing invalid base64 or wrong byte count
- WHEN `--signing-key` points to this file
- THEN crunch exits with a clear error before any build starts

### Requirement: Verify signatures on all cache hits

Every `check_cache` hit MUST verify that at least one signature matches
a trusted public key. This applies to both local redb entries and remote
binary cache entries.

If no signature matches any trusted key, the system MUST treat the path
as a cache miss and rebuild it.

#### Scenario: Valid local signature

- GIVEN a PathInfo in local redb signed by the current builder's key
- WHEN `check_cache` runs
- THEN the signature is verified
- AND the entry is returned as a cache hit

#### Scenario: Corrupted local signature

- GIVEN a PathInfo in local redb whose signature does not verify
  (e.g., redb bitrot, interrupted write)
- WHEN `check_cache` runs
- THEN the entry is treated as a cache miss
- AND a warning is logged about the verification failure
- AND the derivation is rebuilt

#### Scenario: Valid remote signature

- GIVEN a trusted key `cache.nixos.org-1:<pubkey>`
- AND the remote narinfo has a signature from `cache.nixos.org-1`
- WHEN the narinfo is fetched during substitution
- THEN the signature is verified against the trusted key
- AND the path is accepted as a cache hit

#### Scenario: No trusted signature on remote path

- GIVEN trusted keys that do not include the signer
- AND the remote narinfo has signatures from unknown keys only
- WHEN the narinfo is fetched during substitution
- THEN the path is rejected (treated as cache miss)
- AND a warning is logged naming the untrusted signers

### Requirement: Trust-unsigned escape hatch

The CLI MUST provide `--trust-unsigned` which disables signature
verification on cache hits. This is for migration from unsigned stores
and debugging.

#### Scenario: Trust-unsigned accepts unsigned path

- GIVEN `--trust-unsigned` is passed
- AND a PathInfo in local redb has no signatures
- WHEN `check_cache` runs
- THEN the entry is returned as a cache hit with no verification

### Requirement: Local builder key implicitly trusted

The public key corresponding to the configured signing key MUST be
implicitly added to the trusted key set. Users MUST NOT need to
separately list their own public key in `--trusted-public-keys`.

#### Scenario: Builder trusts its own key

- GIVEN a signing key `my-builder-1:<keypair>`
- AND `--trusted-public-keys` is not set
- WHEN a previously-built PathInfo signed by `my-builder-1` is
  checked for cache hit
- THEN the signature is accepted (key implicitly trusted)

### Requirement: Default trusted keys

The system MUST include `cache.nixos.org-1` in the default set of
trusted public keys.

`--trusted-public-keys` on the CLI overrides the default set entirely
(does not append).

#### Scenario: Default cache.nixos.org trust

- GIVEN no `--trusted-public-keys` flag
- WHEN substitution fetches a narinfo signed by `cache.nixos.org-1`
- THEN the signature is accepted

#### Scenario: Override replaces defaults

- GIVEN `--trusted-public-keys my-cache-1:<key>`
- WHEN substitution fetches a narinfo signed by `cache.nixos.org-1`
- THEN the signature is rejected (cache.nixos.org-1 is no longer trusted)

### Requirement: Re-sign existing PathInfo

`crunch store sign <path>` MUST load the PathInfo from redb, compute
a new signature using the provided signing key, append it to
`signatures`, and persist it back.

If the PathInfo already has a signature from the same key name, the
system MUST replace it rather than duplicate.

`crunch store sign --all` MUST iterate all PathInfo entries in redb and
sign each one. This is the migration path for existing unsigned stores.

#### Scenario: Bulk sign unsigned store

- GIVEN 50 unsigned PathInfo entries in redb
- WHEN `crunch store sign --all --signing-key <key>` is run
- THEN all 50 entries gain a signature
- AND subsequent `check_cache` hits verify successfully

#### Scenario: Re-sign with same key

- GIVEN a PathInfo already signed by `my-key-1`
- WHEN `crunch store sign ... --signing-key <my-key-1-keypair>`
- THEN the PathInfo still has exactly one signature from `my-key-1`

### Requirement: Signature reporting in store commands

`crunch store info <path>` MUST display the signature count and
signer names. `crunch store verify` MUST report whether each path
has valid signatures against the configured trusted keys.

#### Scenario: Store info shows signatures

- GIVEN a PathInfo with signatures from `my-cache-1` and `backup-1`
- WHEN `crunch store info <path>` is run
- THEN the output includes both signer names

#### Scenario: Store verify reports untrusted

- GIVEN a PathInfo signed by `unknown-key-1`
- AND trusted keys do not include `unknown-key-1`
- WHEN `crunch store verify <path>` is run
- THEN the report indicates "0 trusted signatures (1 untrusted: unknown-key-1)"
