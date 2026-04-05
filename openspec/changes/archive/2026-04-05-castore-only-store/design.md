## Context

Current `check_cache` (orchestrate.rs ~L1097):

```rust
let abs = PathBuf::from(
    output_path.to_absolute_path_with_prefix(&self.output_dir_str)
);
let stored = self.pathinfo_service.get(digest).await?;
match (stored, abs.exists()) {
    (Some(pi), true)  => { /* cache hit */ }
    (Some(_), false)  => { /* "PathInfo but no file, rebuilding" */ }
    (None, true)      => { /* "file but no PathInfo, rebuilding" */ }
    (None, false)     => { /* miss */ }
}
```

The `abs.exists()` check is the problem. It gates cache hits on the
output dir being writable and the previous export having succeeded.
But the castore already has the content — `PathInfo.node` carries the
root `Node` with blob digests that reference content in the blob/directory
services.

Current data flow for intermediate deps:
1. Build produces output → ingested into blob+directory services → Node
2. Node stored in `output_nodes` HashMap
3. PathInfo persisted to redb
4. `export_castore_to_disk` writes to `output_dir` (fails silently on RO)
5. Next build's `collect_sandbox_inputs` reads from `output_nodes` first,
   falls back to disk

Steps 4-5 disk fallback is unnecessary when the castore path (step 2)
succeeds. And `output_nodes` is in-memory only — lost between runs.
The fix is making `check_cache` reconstruct step 2 from step 3.

## Goals / Non-Goals

**Goals:**
- `check_cache` works without a writable output dir.
- Intermediate deps never touch disk.
- Final roots export to `output_dir` (best-effort, warn on failure).
- No performance regression for the writable-store case.

**Non-Goals:**
- FUSE filesystem exposing the castore. Possible future work but
  orthogonal. The bwrap sandbox already gets content via castore nodes.
- Garbage collection of castore blobs. Separate concern, separate
  openspec.
- Removing `output_dir` entirely. It's still useful for users who want
  outputs on disk.

## Decisions

### 1. Replace disk check with castore content probe

**Choice:** In `check_cache`, replace `abs.exists()` with a check that
the castore actually has the content referenced by the PathInfo node.

For `Node::File { digest, .. }`: `blob_service.has(digest)`.
For `Node::Directory { digest, .. }`: `directory_service.get(digest)` is
Some.
For `Node::Symlink { .. }`: always present (target is inline in the Node).

```rust
match stored {
    Some(path_info) => {
        if self.castore_has_content(&path_info.node).await? {
            // Cache hit — populate output_nodes from PathInfo
            self.output_nodes.insert(output_path.clone(), path_info.node.clone());
            infos.insert(output_name.clone(), path_info);
        } else {
            // PathInfo exists but castore content missing (corruption)
            warn!("PathInfo exists but castore content missing, rebuilding");
            return Ok(None);
        }
    }
    None => return Ok(None),
}
```

**Rationale:** `blob_service.has()` is a local key lookup in redb —
fast. The disk check was already doing stat(2) per output per build,
so this is comparable cost. And it's correct: if the blob exists in
the castore, we can reconstruct the output for any downstream sandbox.

**Alternative rejected:** Keep the disk check as a secondary signal.
This splits the cache into two tiers (castore-only vs castore+disk)
and complicates the logic for no benefit. If the castore has it, the
build is cached. Period.

### 2. Root-only disk export

**Choice:** Add a `is_root: bool` parameter to `persist_and_export_output`
(or check against a root set). Only call `export_castore_to_disk` when
`is_root` is true.

The root set is the set of derivations the user passed to `crunch build`.
The Worker already knows which goals are roots (they're the ones passed
to `want()`). Thread a `roots: HashSet<String>` from Worker into Builder,
or tag the `PreparedBuild`/`BuildOutcome` with `is_root`.

**Rationale:** Intermediate deps are consumed only by other builds, which
access them through the castore. Writing them to disk is wasted I/O. On
a writable store with 50 transitive deps, this saves 50 stat+write
sequences. On a read-only store, it eliminates 50 silent failures.

**Alternative considered:** Always export, keep current behavior. Simpler
but defeats the purpose. The whole point is that the castore is
sufficient for the build pipeline.

**Fallback behavior when export fails for roots:**
```
warning: could not export hello-1.0 to /nix/store: read-only file system
         output is available in castore (use `crunch store export` to retry)
```

Not a fatal error. The build succeeded. The user can retry with a
writable dir or a future `crunch store export` command.

### 3. Reconstruct output_nodes from PathInfo on cache hit

**Choice:** When `check_cache` finds a PathInfo hit, insert the
`path_info.node` into `self.output_nodes`. This makes the node
available for downstream `collect_sandbox_inputs` without any disk
access.

This is the key insight: `output_nodes` is the in-memory bridge
between builds. Currently it's populated only during the current
session's builds. After this change, cache hits also populate it,
closing the gap between fresh builds and cached builds.

### 4. castore_has_content helper

**Choice:** New method on Builder:

```rust
async fn castore_has_content(&self, node: &Node) -> Result<bool, Error> {
    match node {
        Node::File { digest, .. } => {
            self.blob_service.has(digest).await
                .map_err(|e| Error::Store(format!("blob check: {e}")))
        }
        Node::Directory { digest, .. } => {
            self.directory_service.get(digest).await
                .map(|opt| opt.is_some())
                .map_err(|e| Error::Store(format!("directory check: {e}")))
        }
        Node::Symlink { .. } => Ok(true),
    }
}
```

Pure lookup, no mutation, no I/O beyond the local store.

## Risks / Trade-offs

**[Castore corruption]** If redb PathInfo is intact but the blob service
lost data (disk corruption, manual deletion), we get a false cache hit
and downstream builds fail when trying to read the blob for sandbox
mounting. Mitigation: `castore_has_content` explicitly probes the blob.
This is strictly better than the current disk check which doesn't
verify content integrity at all.

**[Disk export confusion]** Users expect outputs at a filesystem path.
With root-only export, `crunch build foo.ncl` puts `foo` on disk but
not its transitive deps. Mitigation: clear CLI output showing which
paths were exported. Future: `crunch store export <path>` to manually
materialize any castore output.

**[Memory pressure from output_nodes]** Populating `output_nodes` for
all cached outputs increases memory use. Each entry is a `Node` enum
(small — digest + size + flags). For 1000 outputs, this is ~50KB.
Not a concern.
