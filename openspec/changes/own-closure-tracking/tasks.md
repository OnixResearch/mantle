## Phase 1: Native closure walker

- [ ] Create `closure.rs` in crunch-store (or crunch-build references module)
- [ ] Implement `resolve_closure(path, local_pathinfo, remote_pathinfo)` that walks references transitively
- [ ] Add cycle detection (visited set) and depth limit (1024)
- [ ] Return `Vec<StorePath>` of full transitive closure
- [ ] Unit tests with mock PathInfo (cycles, deep chains, empty refs, missing paths)

## Phase 2: Wire into build pipeline

- [ ] Replace `resolve_nix_closure()` call in `resolve_and_ingest_sources()` with `resolve_closure()`
- [ ] For crunch-built inputs: query local PathInfo references
- [ ] For Nix seed inputs: query remote PathInfo (binary cache narinfo) references
- [ ] Fallback: if no closure data, warn and return only the declared path
- [ ] Integration test: build with seed input whose closure comes from narinfo

## Phase 3: Remove nix-store dependency

- [ ] Delete `resolve_nix_closure()` function from references.rs
- [ ] Remove any `Command::new("nix-store")` subprocess calls
- [ ] Update AGENTS.md to remove "nix-store in PATH" requirement
- [ ] Verify `crunch build` works on a system without `nix-store` (using `--fetch` bootstrap)

## Phase 4: Documentation

- [ ] Update bootstrap spec to document that closure resolution uses narinfo, not nix-store
- [ ] Add warning message text to error spec: "no closure data for path, mounting without closure"
- [ ] Update AGENTS.md runtime requirements section
