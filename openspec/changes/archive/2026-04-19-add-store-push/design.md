## Context

crunch's store layer has full read-path support for Nix binary caches
(`NixHTTPPathInfoService` fetches `.narinfo`, downloads NARs, ingests into
castore). The write path — producing `.narinfo` + `.nar` files from local
state — uses the same primitives but is only exercised in test helpers
(`render_nar_bytes`, `narinfo_body_for` in `handle.rs` tests).

The standard Nix binary cache directory layout is:

```
<cache-root>/
  nix-cache-info           # StoreDir, WantMassQuery, Priority
  <hash>.narinfo           # one per store path (32-char nixbase32 digest)
  nar/<hash>.nar           # NAR archive, referenced by narinfo URL field
```

Consumers (`nix`, `snix`, crunch itself) resolve paths by fetching
`<base>/<hash>.narinfo`, reading the `URL` field, and downloading the NAR.

## Goals / Non-Goals

**Goals:**
- Export any signed PathInfo from the local store to a flat directory that
  any Nix-compatible substituter can serve (nginx, S3, nix-serve, harmonia).
- Sign narinfos with the same key used for local PathInfo signing.
- Skip unsigned entries by default; `--trust-unsigned` to include them.
- Produce uncompressed NARs first. Compression (xz, zstd) is a follow-up.
- Write `nix-cache-info` with the correct `StoreDir`.
- Report push results (count, total bytes, skipped unsigned) to the caller.

**Non-Goals:**
- HTTP PUT upload to a remote cache server. Directory export is sufficient
  for S3 sync, rsync, or any static file host. HTTP upload is a follow-up.
- Serving a live HTTP binary cache (`crunch store serve`). Separate feature.
- Incremental/delta push. The delta-transfer protocol operates at the
  substitution layer, not the push layer.
- Garbage collection of the push target. The target is append-only; the
  operator manages retention.

## Decisions

### 1. Flat directory layout, not a database

**Choice:** Write `.narinfo` and `.nar` files as plain files in a directory.

**Rationale:** This is what every Nix cache consumer expects. It works with
nginx `autoindex`, S3 buckets, `file://` substituters, and rsync. No server
process needed. The operator controls hosting.

**Alternative rejected:** SQLite or redb index at the target. Adds complexity
and is incompatible with static file hosts.

### 2. NAR file naming and URL field

**Choice:** Write NARs to `nar/<nar-sha256-nixbase32>.nar` and set the
narinfo `URL` field to `nar/<hash>.nar`. Use the NAR's sha256 hash as the
filename, not the store path hash.

**Rationale:** This is the convention used by cache.nixos.org and nix-serve.
Using the NAR hash as the filename means identical content deduplicates
naturally (two store paths with the same content share one NAR file). The
narinfo's `FileHash` field matches the filename.

### 3. Render NAR on the fly, do not cache

**Choice:** Stream each NAR from castore at push time. Do not persist
rendered NARs in the local store.

**Rationale:** NARs are large and redundant with castore content. Caching
them would double storage. Rendering is fast (sequential read from blob
service). If the same path is pushed twice, it re-renders — that's fine
for a manual command.

### 4. Require signed PathInfo by default

**Choice:** Skip PathInfo entries without signatures unless
`--trust-unsigned` is passed. Emit a warning per skipped path.

**Rationale:** Pushing unsigned narinfos produces cache entries that
no consumer will trust. Better to fail visibly. The operator can run
`crunch store sign --all` first, or pass `--trust-unsigned` if they
control the consumer trust config.

### 5. StoreHandle gains a public render_nar method

**Choice:** Promote the test-only `render_nar_bytes()` pattern to a
public `StoreHandle::render_nar<W: AsyncWrite>(&self, node: &Node, writer: W)`
method.

**Rationale:** Both push and future serve need NAR rendering from a
`StoreHandle`. The duplex-channel + spawn pattern is a detail; callers
should not repeat it. The public method takes a generic `AsyncWrite` so
callers can stream to a file, hasher, or HTTP response.

### 6. PathInfo to NarInfo conversion

**Choice:** Use `PathInfo::to_narinfo()` from snix-store, then patch
the `url` and `file_hash`/`file_size` fields after rendering the NAR.

**Rationale:** `to_narinfo()` already fills store_path, nar_hash,
nar_size, references, signatures, ca, and deriver. It leaves `url`
empty and `file_hash`/`file_size` as `None`. After rendering the NAR
to a file, we know the file hash and size, so we fill those fields
and set `url` to the relative path. Then `Display` renders the final
`.narinfo` text.

### 7. Push result reporting

**Choice:** Return a `PushReport` struct with counts (pushed, skipped,
bytes written) rather than printing directly.

**Rationale:** The CLI layer formats the report. Library callers
(future HTTP upload, CI integrations) consume the struct. Matches the
pattern used by `GcReport`.

**Implementation:**

```rust
pub struct PushReport {
    pub pushed_count: u32,
    pub skipped_unsigned_count: u32,
    pub skipped_already_present_count: u32,
    pub total_nar_bytes: u64,
    pub total_narinfo_bytes: u64,
    pub paths: Vec<PushedPath>,
}

pub struct PushedPath {
    pub store_path: String,
    pub nar_hash_hex: String,
    pub nar_size: u64,
}
```

### 8. Idempotent writes — skip existing narinfos

**Choice:** Before rendering a NAR, check whether `<hash>.narinfo`
already exists in the target directory. If so, skip that path and
increment `skipped_already_present_count`.

**Rationale:** Pushing is often repeated (after each CI build, after
adding a few new paths). Re-rendering and re-writing unchanged NARs
wastes I/O. The narinfo file is the index key; if it exists, the NAR
must also exist (or the cache is corrupt, which is the operator's
problem).

## Risks / Trade-offs

**[Large NARs and disk space]** Rendering uncompressed NARs to the
target directory can use significant disk space. Mitigation: document
that compression support is planned; operators can post-process with
`xz` or `zstd` and rewrite the narinfo `URL`/`Compression`/`FileHash`
fields. Or wait for the compression follow-up.

**[Store prefix mismatch]** narinfo `StorePath` uses `/nix/store` in
the `Display` impl. If crunch is running with `--store-prefix /crunch/store`,
the narinfo will say `/nix/store` but the actual paths use `/crunch/store`.
Mitigation: the push command should use the logical store path from
`PathInfo.store_path`, which already carries the correct prefix. The
`NarInfo::Display` impl hardcodes `/nix/store`; we need to verify this
works or patch the URL field. If the prefix is `/crunch/store`, the push
target is a crunch-native cache, not a Nix cache, and consumers must
understand that prefix. Document this constraint.

**[No compression]** Uncompressed NARs are ~2-3x larger than xz-compressed.
This is acceptable for local directories and fast networks. Compression is
the natural follow-up.
