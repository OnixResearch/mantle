# ADR 0055: Plan HTTP cache closures before root admission

## Status

Accepted (2026-08-01)

## Context

Mantle can import explicit paths from Nix-compatible HTTP binary caches. Each explicit path passes signature, store-prefix, NAR hash, PathInfo, castore, and export checks.

An explicit root can still reference runtime paths that are absent. Root import alone therefore cannot prove usable closure state.

Mantle also has a separate foreign-derivation compiler. That compiler recomputes Mantle-owned target identities. Those identities cannot address ordinary Nix binary-cache objects.

## Decision Drivers

- Reuse existing Nix binary caches without adopting Nix evaluation semantics.
- Preserve exact foreign store identity for substitution.
- Fail before content mutation when closure metadata is incomplete or untrusted.
- Keep untrusted graph planning pure, bounded, and testable.
- Reuse Mantle's existing PathInfo and castore admission.
- Keep package correctness and rebuild claims separate from cache admission.

## Decision

Mantle adds an explicit metadata-first HTTP closure-pull mode for one root.

The mode has two phases:

1. Fetch and validate every signed narinfo in the root reference graph. A pure planner enforces member, depth, duplicate, conflict, metadata, and aggregate NAR-size limits.
2. Import complete local or remote dependency members in deterministic order. Import the selected root last.

No NAR request starts until metadata discovery completes.

The planner binds the normalized cache authority, trust-policy BLAKE3, logical store prefix, root, limits, and canonical member facts into a Mantle-owned BLAKE3 plan identity. Nix SHA-256, store-path, NAR, and narinfo facts remain in their required Nix identity domains.

A local PathInfo counts as reusable only when its exact store path matches and its full castore node is complete.

The first operator surface accepts exactly one HTTP root. Existing explicit HTTP pull stays non-recursive. Directory pull stays unchanged.

A dependency failure can leave verified dependency objects as resumable cache state. It cannot persist or export the selected root. A successful closure report does not prove package correctness, local rebuild compatibility, evaluator parity, reproducibility, or release eligibility.

The generic cache mechanism does not evaluate Nix expressions. A later receipt-bound adapter can select roots from accepted foreign-import artifacts.

## Alternatives Considered

### Recurse during ordinary explicit pull

Rejected because it expands network and mutation authority without an explicit operator request.

### Download each NAR while discovering references

Rejected because a late missing or untrusted member could leave the selected root admitted before closure completeness is known.

### Use translated Mantle target paths for Nix cache lookup

Rejected because translated BLAKE3 paths are not Nix binary-cache identities.

### Add a Cachix-specific client

Rejected for the public-cache slice because Cachix uses the standard Nix binary-cache protocol.

### Add evaluator integration first

Rejected because evaluation and cache consumption have separate trust and identity boundaries.

## Consequences

- Closure pull performs more narinfo requests before content transfer.
- The complete metadata graph consumes bounded memory before NAR download.
- Single-root scope avoids partial publication across multiple selected roots.
- Verified dependencies can support resumable retries after a later failure.
- Public Nix-compatible caches become usable without Nix on the consumption path.
- Private cache credentials and foreign-import receipt binding remain separate work.
