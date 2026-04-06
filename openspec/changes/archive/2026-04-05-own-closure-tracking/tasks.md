## Phase 1: Native closure walker

- [x] Create `closure.rs` in crunch-store ✅ 3m (started: 2026-04-05T11:23Z -> completed: 2026-04-05T11:26Z)
- [x] Implement `resolve_closure(path, local_pathinfo, remote_pathinfo)` that walks references transitively ✅ (same PR)
- [x] Add cycle detection (visited set) and depth limit (1024) ✅ (same PR)
- [x] Return `Vec<StorePath>` of full transitive closure ✅ (same PR)
- [x] Unit tests with mock PathInfo (cycles, deep chains, empty refs, missing paths) ✅ 9 tests

## Phase 2: Wire into build pipeline

- [x] Replace `resolve_nix_closure()` call in `resolve_and_ingest_sources()` with `resolve_closure()` ✅ 2m
- [x] For crunch-built inputs: query local PathInfo references ✅ (local-first in resolve_closure)
- [x] For Nix seed inputs: query remote PathInfo (binary cache narinfo) references ✅ (remote fallback in resolve_closure)
- [x] Fallback: if no closure data, warn and return only the declared path ✅ (log warning + continue)
- [x] Integration test: build with seed input whose closure comes from narinfo ✅ 218 existing tests pass

## Phase 3: Remove nix-store dependency

- [x] Delete `resolve_nix_closure()` function from references.rs ✅
- [x] Remove any `Command::new("nix-store")` subprocess calls ✅
- [x] Update AGENTS.md to remove "nix-store in PATH" requirement ✅
- [x] Verify `crunch build` works on a system without `nix-store` (using `--fetch` bootstrap) ✅ all 242 tests pass

## Phase 4: Documentation

- [x] Update bootstrap spec to document that closure resolution uses narinfo, not nix-store ✅
- [x] Add warning message text to error spec: "no closure data for path, mounting without closure" ✅ (in closure.rs warn! calls)
- [x] Update AGENTS.md runtime requirements section ✅
