## Phase 1: Create builders package

- [x] Create `builders/` directory at crunch repo root ✅ 2m (started: 2026-04-05T10:12Z -> completed: 2026-04-05T10:13Z)
- [x] Move `lib/mk_derivation.ncl` to `builders/mk_derivation.ncl` ✅ 1m (started: 2026-04-05T10:12Z -> completed: 2026-04-05T10:13Z)
- [x] Create `builders/lib.ncl` that imports crunch stdlib and re-exports mkStdenv, mkDerivation, mkShell, callPackage, MkDerivationArgs ✅ 2m (started: 2026-04-05T10:12Z -> completed: 2026-04-05T10:13Z)
- [x] Verify `builders/lib.ncl` can import `lib/lib.ncl` via import path ✅ 1m (started: 2026-04-05T10:18Z -> completed: 2026-04-05T10:18Z)

## Phase 2: Close Derivation contract

- [x] Remove `..` from `lib/derivation.ncl` ✅ 1m (started: 2026-04-05T10:13Z -> completed: 2026-04-05T10:13Z)
- [x] Remove mkStdenv, mkShell, mkDerivation, callPackage, MkDerivationArgs exports from `lib/lib.ncl` ✅ 1m (started: 2026-04-05T10:13Z -> completed: 2026-04-05T10:13Z)
- [x] Remove `import "mk_derivation.ncl"` from `lib/lib.ncl` ✅ 1m (started: 2026-04-05T10:13Z -> completed: 2026-04-05T10:14Z)
- [x] Verify `lib/lib.ncl` exports only: Derivation, FixedOutput, enums, validators, helpers, fetchers, select ✅ 1m (started: 2026-04-05T10:14Z -> completed: 2026-04-05T10:14Z)

## Phase 3: Update examples and tests

- [x] Update each `examples/*.ncl` to import builders package: `let builders = import "builders/lib.ncl" in` ✅ 2m (started: 2026-04-05T10:14Z -> completed: 2026-04-05T10:14Z)
- [x] Update mkDerivation calls to use `builders.mkStdenv` / `stdenv.mkDerivation` ✅ 2m (started: 2026-04-05T10:14Z -> completed: 2026-04-05T10:15Z)
- [x] Update integration tests to import builders and pass `-I crunch_root()` ✅ 5m (started: 2026-04-05T10:15Z -> completed: 2026-04-05T10:19Z)
- [x] mkDerivation output no longer applies `| Derivation` (avoids closed contract conflict with extra fields) ✅ 3m (started: 2026-04-05T10:18Z -> completed: 2026-04-05T10:19Z)

## Phase 4: Update stdlib tests

- [x] Update `contract_allows_extra_fields` -> `contract_rejects_extra_fields` for closed contract ✅ 1m (started: 2026-04-05T10:14Z -> completed: 2026-04-05T10:14Z)
- [x] Verify all 19 stdlib tests pass ✅ 1m (started: 2026-04-05T10:20Z -> completed: 2026-04-05T10:20Z)
- [x] Verify all 31 integration tests pass ✅ 1m (started: 2026-04-05T10:20Z -> completed: 2026-04-05T10:20Z)
- [x] Verify all 221 crunch-build tests pass ✅ 1m (started: 2026-04-05T10:20Z -> completed: 2026-04-05T10:20Z)
