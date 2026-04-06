## Phase 1: Create builders package

- [ ] Create `builders/` directory at crunch repo root
- [ ] Move `lib/mk_derivation.ncl` to `builders/mk_derivation.ncl`
- [ ] Create `builders/lib.ncl` that imports crunch stdlib and re-exports mkStdenv, mkDerivation, mkShell, callPackage, MkDerivationArgs
- [ ] Verify `builders/lib.ncl` can import `lib/lib.ncl` via import path

## Phase 2: Close Derivation contract

- [ ] Remove `..` from `lib/derivation.ncl`
- [ ] Remove mkStdenv, mkShell, mkDerivation, callPackage, MkDerivationArgs exports from `lib/lib.ncl`
- [ ] Remove `import "mk_derivation.ncl"` from `lib/lib.ncl`
- [ ] Verify `lib/lib.ncl` exports only: Derivation, FixedOutput, enums, validators, helpers, fetchers, select

## Phase 3: Update bootstrap files

- [ ] Update each `bootstrap/*.ncl` to import builders package: `let builders = import "builders/lib.ncl" in`
- [ ] Update mkDerivation calls to use `builders.mkStdenv` / `stdenv.mkDerivation`
- [ ] Add `builders/` to crunch binary's default import paths (or document --import-path usage)
- [ ] Verify `crunch build bootstrap/hello.ncl` still works

## Phase 4: Update stdlib module in Rust

- [ ] Update `crunch-eval/src/stdlib.rs` to NOT include builders in the stdlib import path
- [ ] Add builders as a separate import path in the pipeline (not bundled with stdlib)
- [ ] Verify `crunch eval` works with the closed Derivation contract
- [ ] Update stdlib tests for closed contract behavior (extra fields rejected)
