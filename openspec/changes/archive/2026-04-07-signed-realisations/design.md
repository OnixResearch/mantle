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

crunch is pre-release. There are no users relying on unsigned PathInfo.
No backwards compat constraint.

## Goals / Non-Goals

**Goals:**
- Always sign PathInfo after every successful build
- Verify signatures on every cache hit (local and remote)
- Auto-generate a signing key on first run if none configured
- Nix-compatible key format and signature format (interop with cache.nixos.org)
- Bulk re-sign command for migrating existing unsigned entries
- Extend `crunch store verify` to check signatures

**Non-Goals:**
- Key rotation automation
- Multi-key signing in a single build (one signing key per invocation)
- Threshold signatures or HSM integration

## Decisions

### 1. Mandatory signing, no unsigned output

**Choice:** `Builder` requires a `SigningKey`. Every PathInfo gets signed.
No `Option`, no "unsigned mode".

**Rationale:** crunch is pre-release. There's no user base to break.
Unsigned PathInfo is strictly worse — it can't be published, can't be
verified, and hides corruption. Making signing optional adds a code path
that produces inferior output for zero benefit.

**Alternative:** Optional signing key, unsigned by default. Rejected —
that's a compatibility stance for a released product. crunch isn't one.

### 2. Auto-generate key on first run

**Choice:** If `--signing-key` is not provided and
`$CRUNCH_CONFIG_DIR/signing-key` does not exist, generate a new ed25519
keypair, write it to `$CRUNCH_CONFIG_DIR/signing-key`, and use it. The
key name is `crunch-<hostname>-1`.

**Rationale:** Signing must be mandatory, so first-run UX can't require
the user to generate a key manually. Auto-generation mirrors how SSH
handles host keys. The generated key is local-only — publishing requires
distributing the public key explicitly.

**Implementation:** `ed25519_dalek::SigningKey::generate(&mut OsRng)`.
Serialize to Nix format via `format!("{}:{}", name, base64(secret ++ public))`.
Write to config dir with 0600 permissions.

### 3. Verify all cache hits, local and remote

**Choice:** Every `check_cache` hit — whether from local redb or a
remote binary cache — verifies that at least one signature matches a
trusted key. Failed verification = cache miss (triggers rebuild).

**Rationale:** "Trust local redb" is the Nix model, but it's wrong for
crunch's goals. crunch replaces the full stack and must detect:
- Bitrot in the redb file
- Corruption from interrupted writes
- Bugs in crunch's own PathInfo construction
- Shared/network store tampering

ed25519 verification costs ~0.1ms. On a build with 200 cached deps,
that's 20ms total. Invisible against actual build times.

**Alternative:** Verify only remote, trust local. Rejected — it saves
microseconds at the cost of a silent corruption window.

**Escape hatch:** `--trust-unsigned` skips verification entirely. Needed
for migration (existing unsigned entries) and debugging.

### 4. Local builder's own key always trusted

**Choice:** The public key corresponding to the configured signing key is
implicitly added to the trusted set. No need to list it separately in
`--trusted-public-keys`.

**Rationale:** Without this, the builder would fail to verify its own
outputs on the next cache check. Requiring users to list their own public
key in trusted-public-keys is a footgun.

**Implementation:** Extract public key from `SigningKey` at startup,
prepend to `trusted_keys` vec.

### 5. Sign in orchestrate.rs, not via SigningPathInfoService wrapper

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

### 6. Nix-compatible key format

**Choice:** Use the same keypair format as `nix-store --generate-binary-cache-key`:
`<name>:<base64-encoded-64-bytes>` where the first 32 bytes are the
ed25519 secret key and the last 32 are the public key.

**Rationale:** Lets users reuse existing Nix signing keys. The parser
already exists as `nix_compat::narinfo::parse_keypair()`.

### 7. Trusted keys default includes cache.nixos.org

**Choice:** `cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=`
is in the default trusted set. `--trusted-public-keys` overrides (not
appends to) the default.

**Rationale:** Substitution from cache.nixos.org must work out of the box.
Overriding (not appending) gives users full control when they explicitly
set keys.

### 8. Migration via `crunch store sign --all`

**Choice:** Provide `crunch store sign --all` to bulk-sign every unsigned
PathInfo in redb. This is the migration path from the current unsigned
state.

**Rationale:** After upgrading, existing cache entries have no signatures
and would all be treated as cache misses (triggering rebuilds).
`crunch store sign --all` is faster than rebuilding everything.

## Risks / Trade-offs

**[First-run rebuild storm]** Users upgrading from unsigned crunch will
see all cached entries become cache misses on the first build after
upgrade. Mitigation: `crunch store sign --all` in the upgrade notes, and
`--trust-unsigned` as a temporary escape hatch.

**[Key loss]** If the auto-generated key is lost (disk failure, deleted
config dir), all previously signed entries become unverifiable. They'll
be treated as cache misses and rebuilt. This is the correct behavior —
unverifiable means untrusted.

**[Performance]** ~0.1ms per verification, ~1ms per signing. Both
negligible against actual build times (seconds to minutes).

**[Nix interop]** crunch-specific derivations have different ATerm
hashes than Nix, so their output paths differ. Signatures from
cache.nixos.org only apply to Nix-originated paths fetched via
substitution. crunch-built paths get crunch's own signature.
