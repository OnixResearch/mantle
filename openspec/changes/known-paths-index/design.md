## Context

`KnownPaths` has two maps: `by_aterm_hash` (aterm hash → entry) and
`hdm_by_drv_path` (drv path string → HDM bytes). `get_by_drv_path()` needs
to go from drv path → entry, but the entry is keyed by aterm hash. The
current code does `.values().find()`.

## Goals / Non-Goals

**Goals:** O(1) drv path lookup. No public API changes.

**Non-Goals:** Don't change the aterm hash keying — it's used for dedup.

## Decisions

### 1. Add a reverse index

**Choice:** Add `drv_path_to_aterm: HashMap<String, [u8; 32]>` to
`KnownPaths`. Populated in `insert()` alongside `hdm_by_drv_path`.
`get_by_drv_path()` becomes:

```rust
let aterm_hash = self.drv_path_to_aterm.get(drv_path)?;
self.by_aterm_hash.get(aterm_hash)
```

**Rationale:** One extra HashMap insert per derivation (already doing two).
Lookup goes from O(n) to O(1).

**Alternative:** Store entries in a `Vec` and build a drv_path index
separately. Rejected — more complex, no benefit over a second HashMap.

## Risks / Trade-offs

**[Memory]** → One extra `String + [u8; 32]` per derivation. Negligible.
