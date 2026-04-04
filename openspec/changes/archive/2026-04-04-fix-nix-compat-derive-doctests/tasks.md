## Phase 1: Try dev-dependency approach

- [x] Add `nix-compat = { path = "../nix-compat" }` to nix-compat-derive's `[dev-dependencies]`
- [x] Run `cargo test -p nix-compat-derive --doc` and check if all 17 pass ✅ 17/17 pass

## Phase 2: Fallback — ignore doctests

- Skipped — Phase 1 succeeded.

## Phase 3: Verify clean workspace

- [x] Run `cargo test --workspace` with no `--exclude` flags ✅ 0 failures
