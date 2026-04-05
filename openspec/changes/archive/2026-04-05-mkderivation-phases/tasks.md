## Phase 1: Default Phases in mk_derivation.ncl

- [x] Refactor `make_script` to emit defaults when phase is absent ✅ 10m
- [x] Default unpackPhase: handle dir/tarball src, enter single subdir ✅ 5m
- [x] Default configurePhase: `./configure --prefix=$out` if executable exists ✅ 3m
- [x] Default buildPhase: `make -j${NIX_BUILD_CORES:-1}` ✅ 2m
- [x] Default installPhase: `make install` ✅ 2m
- [x] Wire `$src` into env when src field is set ✅ 5m
- [x] Set `NIX_BUILD_CORES=4` in default env ✅ 2m
- [x] Empty string phases skip (don't emit anything) ✅ 3m

## Phase 2: mkShell

- [x] Add `mkShell` function to `mk_derivation.ncl` ✅ 10m
- [x] mkShell produces a valid Derivation (passes contract) ✅ 3m
- [x] mkShell build deliberately fails with descriptive message �� 2m
- [x] Export from `lib.ncl` ✅ 1m

## Phase 3: Integration Tests

- [x] Test: mkDerivation with default phases builds a C program ✅ 5m
- [x] Test: mkDerivation with custom configurePhase overrides default ✅ 3m
- [x] Test: mkDerivation with empty configurePhase skips it ✅ 3m
- [x] Test: mkShell evaluates to valid JSON ✅ 3m
- [x] Test: multi-output derivation produces all outputs ✅ 3m
- [x] Test: output selection with crunch.select ✅ 3m
- [x] Test: --no-substitute flag accepted ✅ 2m
- [x] Test: --substituters flag accepted ✅ 2m
- [x] Test: mkDerivation src wired to $src env and inputs ✅ 3m
