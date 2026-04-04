## Phase 1: Test infrastructure

- [ ] Add `assert_cmd` and `predicates` as dev-dependencies
- [ ] Create `tests/integration.rs` skeleton
- [ ] Create `tests/fixtures/` directory with simple `.ncl` files
- [ ] Write a helper function that resolves the crunch binary path

## Phase 2: Eval tests

- [ ] Test `crunch eval` on a simple derivation prints valid JSON
- [ ] Test `crunch eval` on a multi-derivation file prints record-of-records JSON
- [ ] Test `crunch eval` with `-I` flag resolves cross-directory imports
- [ ] Test `crunch eval` on invalid Nickel exits with code 2
- [ ] Test `crunch eval` on nonexistent file exits with code 3 (internal/IO)
- [ ] Test `crunch eval --json` on invalid Nickel emits parseable JSON error

## Phase 3: Bootstrap tests

- [ ] Test `crunch bootstrap -o seed.ncl` creates a valid Nickel file
- [ ] Test generated seed.ncl is importable by `crunch eval`
- [ ] Test `crunch bootstrap` with unknown package exits non-zero

## Phase 4: Build tests (Linux-only)

- [ ] Test `crunch build` on trivial derivation (echo > $out) produces output path
- [ ] Test `crunch build` with `--store /tmp/...` writes output to temp store
- [ ] Test `crunch build` on failing builder exits with code 1
- [ ] Test `crunch build` re-run on same derivation is cached (output says "cached")
- [ ] Test `crunch build` on multi-derivation file builds all derivations

## Phase 5: Error and edge cases

- [ ] Test `crunch build` with missing store dir exits with code 3
- [ ] Test `crunch build` with missing source input exits with code 1
- [ ] Test `--verbose` flag produces additional output on stderr
