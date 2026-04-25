## Why

crunch's store has PathInfo with references (dependency edges), closure resolution, and narinfo parsing — but there's no user-facing way to inspect closures. You can't answer "why does this derivation pull in gcc?" or "what changed between these two builds?" These are daily debugging tools.

## What Changes

- **`crunch store why-depends <path> <dep>`**: Trace the shortest reference chain from a store path to a dependency. Walks PathInfo references recursively and prints the path. Reports "no dependency" if unreachable.
- **`crunch store closure <path>`**: List all transitive dependencies of a store path, with sizes. Summarize total closure size. Uses the existing `resolve_closure()` from `crunch-store`.
- **`crunch store diff <path-a> <path-b>`**: Compare two store path closures. Show added, removed, and changed (same name, different hash) members. Useful for understanding what a rebuild changed.
- **`crunch store tree <path>`**: Tree-formatted view of the dependency graph, similar to `cargo tree`. Supports `--depth` to limit recursion.

## Capabilities

### New Capabilities
- `why-depends`: Trace dependency chains between store paths
- `closure-list`: List a store path's full closure with sizes
- `closure-diff`: Compare two closures for added/removed/changed members
- `closure-tree`: Tree view of the dependency graph

### Modified Capabilities
- `store`: Gains new subcommands — existing store commands unchanged

## Impact

- **Files**: Extend `src/store_cmd.rs` with new subcommands, new closure-walking utilities in `crates/crunch-store/`
- **APIs**: New CLI subcommands. Closure walking logic may become public API in `crunch-store`.
- **Dependencies**: None
- **Testing**: Unit tests with synthetic PathInfo graphs for each traversal mode. Integration test with a real build output.
