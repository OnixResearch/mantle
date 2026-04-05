## Context

`check_cache` today does: for each output of a derivation, compute the
store path digest → `pathinfo_service.get(digest)` → check castore
content exists. If any output misses, return `None` (rebuild).

`NixHTTPPathInfoService::get(digest)` does the full pipeline: fetch
`<digest>.narinfo` → parse → download NAR → decompress → ingest into
blob_service + directory_service → verify nar_hash + nar_size → return
`PathInfo` with the root `Node`. It shares the same `BlobService` and
`DirectoryService` as the Builder, so after a remote get, the castore
content is immediately available for downstream builds.

## Goals / Non-Goals

**Goals:**
- Substitute from `cache.nixos.org` and custom Nix binary caches.
- Write-through to local redb so each path is fetched at most once.
- Skip substitution for FODs (they have declared hashes; the build
  itself is the fetch).
- `--no-substitute` for offline / hermetic builds.
- Progress indication: print which paths are being substituted.

**Non-Goals:**
- Pushing to a remote cache (write support). `NixHTTPPathInfoService::put`
  returns an error by design.
- Signature verification in the first pass. Can be added later via
  `trusted_public_keys`. The first version trusts the cache URL.
- Multi-cache fan-out (query N caches in parallel). Single substituter
  first, extend later.
- Substituting source inputs. Only derivation *outputs* are substituted.
  Source inputs (`input_sources`) are expected on disk already.

## Decisions

### 1. Remote PathInfoService as a field on Builder

**Choice:** Add `remote_pathinfo: Option<Arc<dyn PathInfoService>>`
to `Builder`. `check_cache` queries it after local miss.

**Rationale:** `NixHTTPPathInfoService` already implements
`PathInfoService`. No new trait or abstraction needed. The `Arc<dyn>`
avoids infecting Builder's generic params (Builder already has 4 type
params; a 5th would cascade through every call site).

**Alternative rejected:** snix-store's `CachePathInfoService` wraps
a near + far service. But it doesn't let us control *when* to skip
the remote (e.g., for FODs). Calling the remote explicitly in
`check_cache` gives us that control.

**Implementation:**

```rust
// In Builder:
remote_pathinfo: Option<Arc<dyn PathInfoService>>,

// In check_cache, after local miss:
if let Some(remote) = &self.remote_pathinfo {
    if let Some(remote_pi) = remote.get(digest).await? {
        // Persist locally for next time.
        self.pathinfo_service.put(remote_pi.clone()).await?;
        // The NAR was already ingested into our blob/dir services
        // by NixHTTPPathInfoService::get.
        self.output_nodes.insert(output_path.clone(), remote_pi.node.clone());
        infos.insert(output_name.clone(), remote_pi);
        continue; // next output
    }
}
```

### 2. Skip substitution for FODs

**Choice:** If the derivation has `fixed_output` (any output has
`ca_hash`), don't query the remote cache. Build locally via the
fetcher pipeline.

**Rationale:** FODs are content-addressed by their declared hash.
The fetcher pipeline already downloads the content and verifies the
hash. Substituting a FOD from cache.nixos.org would work but the
fetcher path is simpler and already handles `--fix` for hash
mismatches. Also, FOD store paths use sha256 output hashes that
differ from crunch's BLAKE3 paths — the cache won't have them under
the same digest anyway.

### 3. CLI flags

**Choice:**
- `--substituters <url>[,<url>]` — default `https://cache.nixos.org`
- `--no-substitute` — disables remote lookup

**Rationale:** Matches Nix's flag names for familiarity. Default to
cache.nixos.org since crunch derivation paths use `/nix/store` and
can reference Nix-built dependencies.

**Implementation:** In `main.rs`, when `--no-substitute` is absent,
construct a `NixHTTPPathInfoService` sharing the same `MemoryBlobService`
and `RedbDirectoryService` as the Builder. Wrap in `Arc<dyn PathInfoService>`
and pass to `Builder::with_state_dir`.

### 4. Output path digest mapping

**Choice:** Use the output path's 20-byte digest directly for the
remote lookup. This is the same digest that `pathinfo_service.get()`
uses locally.

**Rationale:** Nix binary caches index by the 20-byte nixbase32
digest of the store path. `NixHTTPPathInfoService::get(digest)`
constructs the `.narinfo` URL from this digest. crunch's output
paths (for input-addressed derivations) are computed the same way
as Nix's, so the digests match. For CA derivations, the output
path isn't known until after build, so substitution can't help
(and shouldn't — the point of CA is content identity, not input
identity).

**Caveat:** This only works for input-addressed derivations whose
store paths are computed identically to Nix. If crunch's HDM or
path computation diverges (e.g., BLAKE3 HDM), the digests won't
match cache.nixos.org and substitution will 404. That's fine —
404 means "not cached, build locally."

### 5. Error handling for remote failures

**Choice:** Remote failures (network errors, parse errors, bad NAR)
are logged as warnings and treated as cache misses. The build
proceeds locally. No build failure from a flaky cache.

**Rationale:** Substitution is an optimization. A broken cache
should degrade to building, not crash. The user sees a warning
and can investigate or pass `--no-substitute`.

## Risks / Trade-offs

**[Digest mismatch for BLAKE3 paths]** crunch uses BLAKE3 for HDM
and store path computation. If the ATerm hash differs from Nix's
SHA256-based computation, the output path digest won't match
cache.nixos.org. Substitution will 404 for those paths. Mitigation:
this is expected and correct — only Nix-compatible paths can be
substituted from Nix caches. Document this clearly.

**[Memory for large NARs]** `NixHTTPPathInfoService::get` streams
the NAR through decompression into castore. It doesn't buffer the
whole NAR in memory. The existing snix implementation handles this.

**[Write amplification]** Every substituted path writes to both
castore (blobs + directories) and redb (pathinfo). This is the
same write path as a local build, so no new concern.
