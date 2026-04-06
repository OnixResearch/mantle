## Context

crunch builds derivations and persists PathInfo with `signatures: vec![]`.
The vendored code already has all the cryptographic primitives:

- `nix_compat::narinfo::SigningKey` wraps ed25519-dalek signing
- `nix_compat::narinfo::VerifyingKey` wraps ed25519-dalek verification
- `nix_compat::narinfo::fingerprint()` computes the canonical fingerprint
- `nix_compat::narinfo::Signature` holds `(name, 64-byte ed25519 sig)`
- `snix_store::pathinfoservice::SigningPathInfoService` wraps any
  PathInfoService and signs on `put()`
- `nix_compat::narinfo::parse_keypair()` parses Nix-format keypair files

crunch does not use any of this. Every `PathInfo` constructed in
`orchestrate.rs` hard-codes `signatures: vec![]`.

## Goals / Non-Goals

**Goals:**
- Sign PathInfo after every successful build
- Verify signatures on substituted paths from remote caches
- Provide CLI flags for key paths and trusted keys
- Nix-compatible key format and signature format (interop with cache.nixos.org)
- Extend `crunch store verify` to check signatures

**Non-Goals:**
- Key generation (use `nix-store --generate-binary-cache-key` or openssl)
- Automatic key rotation
- Multi-key signing in a single build (one signing key per invocation)
- Threshold signatures or HSM integration

## Decisions

### 1. Sign in orchestrate.rs, not via SigningPathInfoService wrapper

**Choice:** Add signing directly in `persist_and_export_output` after
constructing the PathInfo, before `PathInfoService::put()`.

**Rationale:** `SigningPathInfoService` is a composition-framework wrapper
designed for snix's service composition DSL. crunch constructs its
`PathInfoService` directly in `main.rs`, not via the snix composition
system. Wrapping adds indirection for no benefit. The signing logic is
four lines: compute fingerprint, call `signing_key.sign()`, push the
`Signature` onto `path_info.signatures`.

**Alternative:** Use `SigningPathInfoService` as a wrapper. Rejected
because it forces the signing key into the service layer rather than the
build layer, and crunch's Builder already has direct access to PathInfo
before persistence.

### 2. Optional signing key on Builder

**Choice:** `Builder::new()` takes `Option<SigningKey<ed25519_dalek::SigningKey>>`.
If `None`, PathInfo is persisted unsigned (current behavior). If `Some`,
every PathInfo gets signed before `put()`.

**Rationale:** Backwards-compatible. Local-only builds don't need
signatures. Signing becomes mandatory only when publishing or
distributing.

**Implementation:** New field `signing_key: Option<SigningKey<...>>` on
`Builder`. `sign_pathinfo(&mut PathInfo, &SigningKey)` is a pure
function in a new `signing.rs` module. Called from
`persist_and_export_output` when `self.signing_key.is_some()`.

### 3. Verification on substitution, not on every cache check

**Choice:** Verify signatures only when a PathInfo comes from a *remote*
source (NixHTTPPathInfoService). Local redb entries are trusted.

**Rationale:** The local database is already trusted storage - if
someone can modify redb, they can modify the store. Verifying local
entries on every cache check wastes cycles. Remote entries are the trust
boundary.

**Implementation:** `check_cache` in `orchestrate.rs` already
distinguishes local vs remote paths. After fetching from remote,
call `verify_signatures(&path_info, &trusted_keys)`. If zero trusted
signatures, log a warning and skip (with `--trust-unsigned`) or return
cache miss (without it).

### 4. Nix-compatible key format

**Choice:** Use the same keypair format as `nix-store --generate-binary-cache-key`:
`<name>:<base64-encoded-64-bytes>` where the first 32 bytes are the
ed25519 secret key and the last 32 are the public key.

**Rationale:** Lets users reuse existing Nix signing keys. The parser
already exists as `nix_compat::narinfo::parse_keypair()`.

### 5. Trusted keys as CLI flag, not config file only

**Choice:** `--trusted-public-keys cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=`
on the command line, with a default set matching cache.nixos.org. Also
readable from `$CRUNCH_CONFIG_DIR/trusted-public-keys` (one per line).

**Rationale:** Matches Nix's `trusted-public-keys` setting. CLI flag
overrides file. Default includes cache.nixos.org so substitution works
out of the box.

## Risks / Trade-offs

**[Performance]** Signing is ~1ms per PathInfo (ed25519 is fast). Verification
is similarly cheap. No measurable impact on build times.

**[Key availability]** Without `--signing-key`, builds produce unsigned
PathInfo. This is fine for local use but means `crunch store verify` on
signature counts will report "no signatures". The error message must be
clear: "no signing key configured; pass --signing-key to sign builds".

**[Nix cache.nixos.org interop]** crunch-specific derivations have
different ATerm hashes than Nix, so their output paths differ. Signatures
from cache.nixos.org only apply to Nix-originated paths fetched via
substitution. crunch-built paths get crunch's own signature.
