## Phase 1: Try dev-dependency approach

- [ ] Add `nix-compat = { path = "../nix-compat" }` to nix-compat-derive's `[dev-dependencies]`
- [ ] Run `cargo test -p nix-compat-derive --doc` and check if all 17 pass
- [ ] If circular dep error, fall back to Phase 2

## Phase 2: Fallback — ignore doctests

- [ ] If Phase 1 fails, mark all 17 doctest blocks as `ignore`
- [ ] Add a comment explaining why: "requires nix-compat in scope, see vendoring note"

## Phase 3: Verify clean workspace

- [ ] Run `cargo test --workspace` with no `--exclude` flags
- [ ] Confirm 0 failures
