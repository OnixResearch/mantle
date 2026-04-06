## Context

`resolve_nix_closure()` in references.rs runs `nix-store -qR <path>` as
a subprocess to get the transitive runtime closure of each source input.
crunch's goal is to replace Nix. Depending on `nix-store` for core
functionality contradicts that.

The data already exists: PathInfo.references for crunch-built outputs,
and narinfo References for Nix cache entries. crunch just doesn't use it.

## Goals / Non-Goals

**Goals:** Remove the `nix-store` dependency. Resolve closures from
crunch's own data (PathInfo + narinfo).

**Non-Goals:** Change the sandbox mounting logic. Change what closures
contain. Implement a full Nix-compatible reference scanner.

## Decisions

### 1. Two-source closure resolution

**Choice:** Check local PathInfo first, then binary cache narinfo.

**Rationale:** Crunch-built outputs have PathInfo in redb. Nix seed
inputs don't have local PathInfo, but their narinfo is available from
cache.nixos.org. Together these cover all cases.

### 2. Narinfo reference parsing

**Choice:** Parse the `References:` field from narinfo. Already
available — `NixHTTPPathInfoService` fetches narinfo and constructs
PathInfo which includes references.

**Rationale:** The remote PathInfo already has references. We just
need to query it for closure purposes, not just for substitution.
The implementation is: `remote_pathinfo.get(digest)` → read
`path_info.references` → recurse.

### 3. Graceful fallback

**Choice:** If neither local nor remote closure data is available,
mount only the declared path and warn.

**Rationale:** Static binaries (the `--fetch` bootstrap case) have
no runtime deps. Dynamic binaries without closure data will fail at
runtime with a clear "library not found" error. This is better than
silently depending on an external tool.

### 4. Closure walks in crunch-store

**Choice:** The closure walker lives in crunch-store, since it queries
PathInfo/narinfo.

**Rationale:** It's a store-level operation. The build engine calls
`store.resolve_closure(path)` and gets back a list of paths.

## Risks / Trade-offs

**[Bootstrap regression]** The `crunch bootstrap` (non-`--fetch`) mode
generates seed.ncl from Nix store paths. Those paths' closures were
previously resolved by `nix-store -qR`. After this change, they're
resolved from narinfo. If a seed path has no narinfo (local-only Nix
build, not in any cache), closure resolution fails silently (warning +
mount without closure).

**Mitigation:** `crunch bootstrap --fetch` is the recommended path and
doesn't need closures (static binaries). For `crunch bootstrap` from
Nix, the user's Nix store paths are almost always in cache.nixos.org.

**[Network dependency for Nix seeds]** Closure resolution now requires
binary cache access for Nix seed inputs. Previously it required
`nix-store` on PATH. The `--no-substitute` flag should also disable
closure resolution from remote, falling back to mount-without-closure.
