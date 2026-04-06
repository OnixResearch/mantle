# Signed Realisations Specification

## Purpose

Defines how crunch signs build outputs and verifies signatures on
substituted paths, enabling trust in binary caches and remote builders.

## Requirements

### Requirement: Sign PathInfo after build

The system MUST sign each PathInfo with an ed25519 key before persisting
it, when a signing key is configured. The signature MUST cover the
canonical narinfo fingerprint: `1;<store_path>;sha256:<nar_hash_nix32>;<nar_size>;<references>`.

If no signing key is configured, PathInfo MUST be persisted unsigned
(backwards-compatible).

#### Scenario: Build with signing key

- GIVEN `--signing-key /path/to/key` is provided
- AND the key file contains a valid Nix-format ed25519 keypair
- WHEN a derivation is built successfully
- THEN the persisted PathInfo contains exactly one signature
- AND the signature verifies against the corresponding public key

#### Scenario: Build without signing key

- GIVEN no `--signing-key` flag is provided
- WHEN a derivation is built successfully
- THEN the persisted PathInfo has `signatures: []`

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

### Requirement: Verify signatures on substituted paths

When fetching a PathInfo from a remote binary cache, the system MUST
verify that at least one signature matches a trusted public key.

If no signature matches any trusted key, the system MUST treat the
path as a cache miss (fall through to local build) unless
`--trust-unsigned` is passed.

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

#### Scenario: Trust-unsigned override

- GIVEN `--trust-unsigned` is passed
- AND the remote narinfo has no valid signatures
- WHEN the narinfo is fetched during substitution
- THEN the path is accepted despite missing signatures

### Requirement: Local PathInfo trusted without verification

PathInfo entries already in the local redb database MUST NOT be
re-verified on cache check. The local database is trusted storage.

#### Scenario: Local cache hit skips verification

- GIVEN a PathInfo in the local redb database (with or without signatures)
- WHEN `check_cache` runs
- THEN the entry is returned as a cache hit with no signature check

### Requirement: Default trusted keys

The system MUST include `cache.nixos.org-1` in the default set of
trusted public keys, so substitution from the default cache works
without extra configuration.

The default MAY be overridden by `--trusted-public-keys` or a
config file at `$CRUNCH_CONFIG_DIR/trusted-public-keys`.

#### Scenario: Default cache.nixos.org trust

- GIVEN no `--trusted-public-keys` flag
- AND no config file
- WHEN substitution fetches a narinfo signed by `cache.nixos.org-1`
- THEN the signature is accepted

### Requirement: Re-sign existing PathInfo

`crunch store sign <path>` MUST load the PathInfo from redb, compute
a new signature using the provided signing key, append it to
`signatures`, and persist it back.

If the PathInfo already has a signature from the same key name, the
system SHOULD replace it rather than duplicate.

#### Scenario: Sign an unsigned path

- GIVEN an unsigned PathInfo in redb for `/nix/store/<hash>-hello`
- WHEN `crunch store sign /nix/store/<hash>-hello --signing-key <key>`
- THEN the PathInfo gains one signature
- AND `crunch store info` shows the signature

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
