## Why

`references.rs::resolve_nix_closure()` shells out to `nix-store -qR` to
resolve runtime closures of source inputs. This means:

- crunch cannot build on a machine without Nix installed (unless all
  inputs are statically linked)
- The closure model is borrowed from Nix rather than owned by crunch
- Every seed input triggers a subprocess call to a tool crunch is
  supposed to replace
- AGENTS.md acknowledges: "nix-store in PATH (for nix-store -qR closure
  resolution). Without it, only statically-linked builders work in the
  sandbox."

The project goal is to **replace Nix entirely**. Shelling out to
`nix-store` for core functionality contradicts that goal.

crunch already has the data it needs: PathInfo records contain a
`references` field listing runtime dependencies. For crunch-built
outputs, this is populated by refscan after every build. For seed
inputs from Nix, the NixHTTPPathInfoService (binary cache) also
returns narinfo with references.

The missing piece: crunch doesn't query references for source inputs.
It resolves closures by asking Nix instead of walking its own reference
graph.

## What Changes

Replace `nix-store -qR` with crunch-native closure resolution:

1. **For crunch-built inputs**: walk the `references` field in PathInfo
   recursively. Already available in redb.
2. **For Nix seed inputs**: query the binary cache narinfo for each seed
   path. The narinfo contains `References:` listing direct runtime deps.
   Walk transitively.
3. **Fallback**: if a seed path has no narinfo and no local PathInfo,
   log a warning and mount only the declared path (no closure). Static
   binaries work; dynamic ones fail with a clear error instead of silently
   depending on nix-store.
4. **Remove `resolve_nix_closure()`** and the `nix-store` subprocess call.

## Capabilities

### New Capabilities
- `closure_resolve()`: native reference graph walker using PathInfo +
  binary cache narinfo
- crunch builds work without Nix installed (for any seed, not just static)

### Modified Capabilities
- `resolve_and_ingest_sources()`: calls `closure_resolve()` instead of
  `resolve_nix_closure()`
- Binary cache client: queried for narinfo references, not just for
  substitution

### Removed Capabilities
- `resolve_nix_closure()`: the `nix-store -qR` subprocess call

## Impact

- **Files**: modified `crates/crunch-build/src/references.rs`, modified
  `orchestrate.rs`, possibly new closure module in crunch-store
- **APIs**: `resolve_nix_closure` removed
- **Dependencies**: removes runtime dependency on `nix-store` binary
- **Testing**: tests with mock narinfo responses instead of requiring Nix
