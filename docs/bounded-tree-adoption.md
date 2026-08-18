# Bounded Tree Adoption

Mantle consumes `bounded-tree-core` and `bounded-tree-cap` from the canonical
Radicle project:

```text
rad:zqhtZvsteJhxCJE96dMAZSZ9y1PX
```

The reviewed source revision is:

```text
b0fd0103bc9eed2c1b6d852045959462d105d8f1
```

Cargo and Nix use the same immutable HTTPS Git projection:

```text
https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git
```

There is no sibling-path or GitHub fallback.

## Mapping

Mantle uses the shared mechanism at two boundaries:

- Release tree planning maps retained `TreeCopyLimits` and compatibility DTOs
  to `bounded-tree-core`. The adapter maps typed blockers back to stable Mantle
  diagnostic classes.
- Release tree observation, hashing, source revalidation, and destination copy
  use `bounded-tree-cap` through caller-opened capabilities.
- Frontend directory artifacts use shared ordered member facts and shared copy
  execution. Mantle keeps its `mantle-frontend-artifact-tree-v1` root preimage,
  executable-bit policy, artifact references, manifests, and store layout.

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
