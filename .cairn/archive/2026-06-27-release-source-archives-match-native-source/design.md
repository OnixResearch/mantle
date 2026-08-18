## Context

`release create` writes a temporary source tar before copying it into the evidence bundle. That archive used the self-build staging allowlist plus a few explicit workflow files. The provider fixed-point path, however, uses native Rust planning, whose path-source digest walks the package root while skipping `.agent`, `.git`, `.jj`, `.pi`, `target`, and root `cairn`.

When the publisher builds a provider proof from the live checkout but the witness replays from the reduced source archive, the root package source digest changes. That digest feeds native unit identity and rustc metadata, so a witness can produce a valid fixed-point provider proof that cannot match the signed provider release binary.

## Decisions

### 1. Align release source archives with native path-source visibility

**Choice:** Include every tracked path that is visible to native path-source hashing, plus verified `vendor-deps/`, instead of reusing the self-build staging allowlist.

**Rationale:** The release source archive is the material witnesses replay. It must preserve the source identity that the publisher's native provider proof used, otherwise digest-first witness binding correctly fails.

### 2. Keep private/runtime skips fail-closed

**Choice:** Skip `.agent`, `.git`, `.jj`, `.pi`, and `target` at any path component, and skip root `cairn` just like native source hashing.

**Rationale:** Ignored scratch directories and private signing material must not enter release source archives, even if a caller accidentally force-tracks them. Root Cairn lifecycle evidence is not part of native package source identity.

## Risks / Trade-offs

- Source archives become larger because tracked package-adjacent docs, scripts, and tests are retained.
- This does not by itself prove an independent witness; provider/self-hosting proofs and final verification still need to be rerun from the corrected source snapshot.
