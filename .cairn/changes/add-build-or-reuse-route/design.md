# Design: Build or reuse on the cache-inputs route

## Context

`cache-inputs-build-roots-v1` hydrates the selected roots' direct input closure
from one receipt-bound signed cache and refuses every present root output,
because a present root can only be attested by a cache key. The route therefore
cannot satisfy "an unchanged job is reused from the store without re-running" on
the route where Mantle is the builder.

The adopted decision is option A: the build route may reuse a root only when
Mantle itself built it earlier and signed it with its builder key. The route id
is `cache-inputs-build-roots-v2`; v1 keeps meaning "always built".

Relevant facts from the code map: route constants and validation live in
`src/foreign_executable_plan.rs` (constants at :62-63, `resolve_realization_route`
:537-557, `validate_plan_route` :641-673); present-root refusal is
`validate_cache_inputs_roots` (`src/foreign_realization.rs:378-392`) called from
`src/foreign_realization_shell.rs:214-233,265-281`; the only local signer is the
realize keypair passed to `crunch_build::signing::sign_pathinfo_with_store_dir`
(`crates/crunch-build/src/orchestrate.rs:1911`); receipt v1 is
`src/foreign_realization_receipt.rs` with domain
`b"mantle-foreign-realization-receipt-v1\0"`; the realize path holds no lock
today.

## Decisions

### Decision: A new route constant, not a mode flag

**Choice:** Add `CACHE_INPUTS_BUILD_OR_REUSE_ROUTE = "cache-inputs-build-roots-v2"`
beside the two existing constants. `resolve_realization_route` accepts it only
when the policy preserves paths, and `validate_plan_route` applies the same
`/nix/store` + substitution-audit + path-drift checks and the same
fixed-output-root refusal as v1. `validate_foreign_realization_admission`
treats both cache-input routes as build routes for fixed-output rejection.

**Rationale:** The route is recorded in the plan, the receipt, and the reports;
a new value keeps v1 and every other route byte-identical.

### Decision: Reuse is keyed by builder fingerprint and a write-once index

**Choice:** Realization on v2 computes, per selected root output:
- builder keys = the local realize keypair (role `builder`) plus every
  `--trusted-builder-public-key` entry (role `builder`, verify-only);
- the trust-policy digest over sorted builder/cache fingerprints;
- the original receipt from
  `<state-dir>/foreign-realization/built-roots/<root derivation basename>.json`.

`classify_present_root` is pure and returns either a reuse fact (original
receipt digest, NAR facts, signer) or a typed refusal from the nine reasons in
the requirement. A present root is reused iff its indexed receipt is a valid v2
receipt built on this route, its entry matches derivation/output/path, its NAR
hash matches both the entry and the bytes re-read from the store, and a PathInfo
signature verifies over the Nix fingerprint with a trusted builder key whose
fingerprint equals the entry's signer fingerprint.

**Rationale:** Path presence alone never qualifies; the index is the record of
Mantle's own build. Write-once publication with an identical-bytes allowance
means two concurrent runs cannot fork the evidence.

**Cache-key fingerprints:** The contract defines one fingerprint encoding
(`b"mantle-builder-public-key-v1\0"` ++ ed25519 public key). The trust-policy
digest uses the same encoding for the plan's cache keys, so the digest depends
only on key material and never on Nix key names.

**input_closure_blake3:** Digest of
`b"mantle-realization-input-closure-v1\0"` ++ canonical JSON of the root's
admitted input closure as recorded in `cache_closure` facts, sorted by logical
path and reduced to `{path, nar_sha256, nar_size}`. It binds the built root to
the exact hydrated inputs; the receipt already records the full facts.

### Decision: Locks, shell order, and no work on reuse

**Choice:** The shell takes one exclusive `fs2` file lock per selected root
derivation basename under `<state-dir>/foreign-realization/locks/`, before any
store open, and holds every guard until the receipt is published. The shell
then opens the store without remotes and checks presence. For a root whose
outputs are all present, classification decides reuse; any refusal blocks the
job. If every selected root output is reusable, the shell skips hydration and
runs the scheduler store-only path, then proves each reused root came back as a
cache hit (`reused_root_outcome_is_cache_hit`) before publishing. If any root
output is absent, the shell probes the cache only for absent roots, hydrates the
admitted seed closure, reopens without substituters, re-asserts absence, and
builds exactly as v1. After the run it re-verifies each reused root's PathInfo
against its classification and rejects any rebuilt reused root.

Store access goes through two new narrow crunch-store methods:
`recompute_nar_sha256(node)` (render the PathInfo node and hash the NAR stream)
and the existing `export_cached_path_info`. The verifier never trusts the
recorded `nar_sha256` alone; it also recomputes from content.

**Rationale:** A decision made before mutation plus a post-run guard is the only
way both to guarantee "no build on reuse" and to keep one receipt builder.
Skipping hydration when nothing is absent makes the reuse run free of cache
traffic.

### Decision: Receipt v2 is a separate closed-world type

**Choice:** `ForeignRealizationReceiptV2` repeats the v1 field set and adds the
required `roots` array, with its own schema string and digest domain. v1 stays
untouched. Nesting is exactly the shared contract: `signer {key_name,
public_key_blake3, role}`, `built {mantle_version,
execution_profile_digest_blake3, input_closure_blake3}`,
`reused {original_receipt_blake3, source, builds, downloads}`. The v2 builder
reuses the v1 unit/closure classification by constructing the same material for
the v2 struct; a reused root's unit reports `already-present`.

**Rationale:** Kiln and Cairn pin the v1 schema; a new schema keeps their v1
readers correct. Closed-world parsing plus digest recomputation is what makes an
index entry trustworthy.

### Decision: One additive vendored accessor

**Choice:** `nix_compat::narinfo::VerifyingKey` gains
`pub fn public_key_bytes(&self) -> [u8; 32]`, needed to compute the fixed
fingerprint without re-parsing key strings. No behavior changes.

**Rationale:** The vendor fork already carries Mantle patches; a two-line
accessor is smaller and safer than duplicating Nix key parsing in `src/`.

## Risks / Trade-offs

- A present root that is not reusable blocks route v2 for that job (typed
  refusal), as decided. Operators rebuild into a new state directory or accept
  the refusal; no automatic store deletion.
- Reuse does not separate `reused` from fresh work for Kiln Verify; the adapter
  follow-up and a force-rebuild are recorded non-goals.
- Key rotation without old-key trust stops reuse (`signer-mismatch`); passing
  the retired key as `--trusted-builder-public-key` restores it verify-only.
- The index is per state directory; a different `--state-dir` has no built-root
  history and therefore builds.

## Validation

Positive: first run `built` with an index entry; second run `reused` with zero
builds, zero NAR downloads, and the first receipt's digest as the link; changed
derivation builds; retired key accepted via `--trusted-builder-public-key`;
concurrent realizations yield one `built` and one `reused`.

Negative: cache-key-only root; tampered builder signature beside a valid cache
signature; altered bytes; missing, tampered, foreign-route, or
other-derivation index entries; same key name with different public key;
untrusted/rotated builder key; cache-offered absent root; route v1 unchanged
(refuses a present root, emits v1).

Unit: `classify_present_root` for each refusal and the reuse path, fingerprint
and trust-policy digests, receipt v2 digest stability and closed-world parse.

Docs: `docs/foreign-import-operator-guide.md` and
`docs/foreign-import-trust-model.md` describe route v2, the builder key, the
index, the refusals, and the non-claims.

## Non-Claims

- Reuse proves only that an earlier Mantle build signed these bytes; it does not
  prove package correctness, reproducibility, evaluator equivalence, or fresh
  execution.
- Cairn's reuse-link check validates the recorded link and fingerprints, not
  signature bytes.
- Reuse from a cache, key rotation by rebuild-compare-co-sign, Kiln receipt-v2
  support, and Verify force-rebuild remain follow-ups.
