## Context

The Clankers root bundle uses SHA-256 SRI hashes because `crunch.fetchTarball` consumes Nix-compatible fixed-output hashes. That is correct for fetch compatibility, but it leaves no BLAKE3 final proof hash for Crunch-owned evidence.

## Decisions

### 1. Add a canonical final proof document

**Choice:** Record `packages/clankers/clankers-root-proof.json` with a top-level `final_proof_hash_blake3`. The hash is computed over canonical JSON of the proof payload excluding the hash field itself (`sort_keys`, compact separators, trailing newline).

**Rationale:** This avoids ambiguity around which compatibility hash is the final proof hash while keeping the existing fetch path unchanged.

### 2. Keep compatibility hashes alongside BLAKE3

**Choice:** Leave `bundle_recursive_sha256_sri` and `bundle_archive_sha256_sri` in place, and add `bundle_archive_blake3` plus `final_proof_hash_blake3`.

**Rationale:** SHA-256 SRI remains necessary for fetchTarball/Cargo interop; BLAKE3 becomes the preferred Crunch proof digest.
