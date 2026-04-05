## Phase 1: mkDerivation core

- [ ] Create `lib/mk-derivation.ncl` with `mkDerivation` function signature: `fun { name, src ? null, bash, build_inputs ? [], ... } =>`
- [ ] Implement `build_inputs` → `inputs` conversion: store path strings passed through, derivation records boxed
- [ ] Implement `build_inputs` → `$PATH` generation: join `<path>/bin` with `:` for each store path in build_inputs
- [ ] Implement inline builder script generation: bash script string that runs phases in order
- [ ] Implement default `unpackPhase`: directory → `cp -r`, file → `tar xf` + enter single subdir
- [ ] Implement default `configurePhase`: `test -x ./configure && ./configure --prefix=$out || true`
- [ ] Implement default `buildPhase`: `make -j${NIX_BUILD_CORES:-1}`
- [ ] Implement default `installPhase`: `make install`
- [ ] Wire phase overrides: each phase field is a string; if set, replaces the default; if empty string, phase is skipped
- [ ] Set standard environment variables: `$src`, `$prefix = $out`, `$NIX_BUILD_CORES`, `$CC = gcc`, `$CXX = g++`
- [ ] Apply `crunch.Derivation` contract to the output record
- [ ] Re-export `mkDerivation` from `lib/lib.ncl`

## Phase 2: mkShell

- [ ] Create `lib/mk-shell.ncl` with `mkShell` function: `fun { bash, build_inputs ? [], ... } =>`
- [ ] mkShell produces a Derivation that fails on build (`exit 1`) but has correct env
- [ ] mkShell sets `$PATH` from `build_inputs` same as mkDerivation
- [ ] Re-export `mkShell` from `lib/lib.ncl`

## Phase 3: Eval-time tests

- [ ] Test: `mkDerivation { name, bash, src }` produces record satisfying Derivation contract
- [ ] Test: `build_inputs` store paths appear in `inputs` array
- [ ] Test: `$PATH` env var contains `<input>/bin` entries
- [ ] Test: phase override replaces default (e.g., `configurePhase = "cmake ."`)
- [ ] Test: empty phase string skips the phase (not present in builder script)
- [ ] Test: `src` derivation record appears in `inputs`
- [ ] Test: `mkShell` produces valid Derivation with build_inputs on PATH
- [ ] Test: `mkDerivation` without `src` omits unpack phase

## Phase 4: End-to-end build tests

- [ ] Create `examples/hello-mkdrv.ncl`: C hello-world via mkDerivation + seed gcc
- [ ] Verify `crunch build examples/hello-mkdrv.ncl` produces a working binary (requires bwrap + seed)
- [ ] Create `examples/from-source.ncl`: fetchTarball → mkDerivation build (small C project)
- [ ] Update `examples/crunch.ncl` self-hosting example to use mkDerivation (structure only — may not fully build yet)

## Phase 5: Documentation

- [ ] Update README.md with mkDerivation usage section
- [ ] Add doc comments to mkDerivation and mkShell functions (Nickel `| doc`)
- [ ] Document phase override patterns in README or a doc/ file
