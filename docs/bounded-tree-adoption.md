# Bounded Tree Adoption

Mantle consumes `bounded-tree-core` and `bounded-tree-cap` from the canonical
Radicle project:

```text
rad:zqhtZvsteJhxCJE96dMAZSZ9y1PX
```

The reviewed source revision is:

```text
250f3b25d0933c2fa106aa82d1cc17a496716f06
```

Its recursive Nix source hash is
`sha256-ZS8gQ8abQk5Pstm0VuQiGMmOrZUsK3TIJ3SItgwGoEY=`. The generated Nix
lock and reviewed lock-vendor table bind this same source content; SHA-256 is
the existing Nix interoperability hash, not a replacement for Mantle BLAKE3
identity.

Cargo and Nix use the same immutable HTTPS Git projection:

```text
https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git
```

There is no sibling-path or GitHub fallback.

## BLAKE3 cohort

The 2026-10-21 owner change admits BLAKE3 `=1.8.5` without changing tree
identities, ordering, or capability boundaries. The published revision retains
the native Devenv gate, focused tree/content-change checks, and native Cairn
archive, validation, and traceability receipts under
`.cairn/archive/2026-10-21-admit-mantle-blake3-cohort/evidence/`.

Mantle owns the exact revision and content-hash pins, generated Cargo and Nix
locks, and consumer integration checks. Owner acceptance alone does not prove
Mantle/Basalt integration, build correctness, or release eligibility.

## Nonblocking member-open cohort

The pinned 2026-10-27 implementation revision uses the existing capability API to open source members nonblocking and no-follow before opened-descriptor kind validation. Its FIFO-substitution regression failed before and passed after the fix; the complete owner Devenv/deny-all Octet gate and separate public copy/revalidation smoke passed. Native owner archive publication is `f73685b4b85ae7446ee3e52eb199ae871cd32ffd`, under `.cairn/archive/1970-01-01-avoid-blocking-member-substitution/`. The implementation pin above deliberately identifies the tested implementation, not the later archive-only commit.

This prevents waiting for a substituted FIFO writer, not arbitrary filesystem latency. Mantle's Wasm source-closure verifier/private copier is the immediate new consumer; Bounded Tree maintainers own the primitive and Mantle maintainers own digest framing, intake, and immutable admission.

## Mapping

Mantle uses the shared mechanism at these boundaries:

- Release tree planning maps retained `TreeCopyLimits` and compatibility DTOs
  to `bounded-tree-core`. The adapter maps typed blockers back to stable Mantle
  diagnostic classes.
- Release tree observation, hashing, source revalidation, and destination copy
  use `bounded-tree-cap` through caller-opened capabilities.
- Frontend directory artifacts use shared ordered member facts and shared copy
  execution. Mantle keeps its `mantle-frontend-artifact-tree-v1` root preimage,
  executable-bit policy, artifact references, manifests, and store layout.
- Wasm source-closure verification and private copying use bounded observation, copy, and revalidation while preserving Mantle's existing v1 native-name/BFS digest framing and source-only directory-role admission.

Mantle-specific chapter transport and release identity projections still read
admitted members. These adapters preserve product formats. They do not
reimplement generic traversal or copy planning.

## Compatibility

Positive parity covers deterministic plans, nested trees, internal symbolic
links, file bytes, executable modes, frontend identities, and destination
content.

Negative parity covers malformed paths, duplicate paths, missing parents,
unsupported entries, external links, source type or content changes, limit
failures, and non-empty destinations.

The shared shell rejects external symbolic links in directory artifacts. This
is a fail-closed narrowing of the former frontend traversal behavior. No stored
artifact identity is promoted from rejected input.

## Rollback

The pre-adoption Mantle revision is:

```text
20349655d5a890b063f6b9e9a488a65c313f0369
```

To restore the reviewed pre-adoption implementation before publication, restore
these paths from that revision and regenerate the Nix lock through Nix:

```text
git restore --source 20349655d5a890b063f6b9e9a488a65c313f0369 -- \
  Cargo.toml Cargo.lock flake.nix \
  crates/crunch-release-core/Cargo.toml \
  crates/crunch-release-core/src/lib.rs \
  crates/crunch-release-core/src/tree_copy.rs \
  src/release_tree_copy.rs src/frontend_artifact_store.rs
nix flake lock
```

Rollback restores the dependency and adapter code together. It does not delete
already stored frontend artifacts or release evidence.

## Claim boundary

Bounded Tree owns deterministic tree-shape admission, ordered member facts,
capability-relative observation, source revalidation, and bounded copy
mechanics.

Mantle retains root authorization, release-root kinds, frontend identities,
bundle layout, product diagnostics, evidence meaning, publication, rollback,
retention, and release decisions.

A passing shared plan does not prove source correctness, atomic snapshots,
publication durability, artifact meaning, build correctness, or release
eligibility.
