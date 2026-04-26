# Tasks: Live-bootstrap seed chain

## Phase 0: Seed and scaffolding

- [x] I1 Check in the hex0 AMD64 seed binary under `bootstrap/seeds/AMD64/hex0-seed`
      (256 bytes from stage0-posix). Add a README in `bootstrap/seeds/` explaining
      provenance and audit expectations.
- [x] I2 Rename current `bootstrap/seed.ncl` to `bootstrap/seed-legacy.ncl`. Update
      all downstream `import "seed.ncl"` to work with either path. Add a
      `bootstrap/seed.ncl` that selects between legacy and full-source based on
      a `CRUNCH_LEGACY_SEED` env var or similar mechanism.

## Phase 1: stage0-posix (hex0 → mescc-tools + M2-Planet)

- [x] I3 Write `bootstrap/stage0-posix.ncl`: fetch stage0-posix source tarball
      (pinned hash), mount hex0 seed, run `kaem.run` for AMD64. Output: mescc-tools
      (M1, hex2, kaem, blood-elf, get_machine), M2-Planet, mescc-tools-extra (catm,
      cp, chmod, mkdir, untar, ungz, unbz2, unxz, sha256sum). Build uses only the
      hex0 seed binary -- no host tools in the sandbox.
- [ ] V1 Validate: `crunch build bootstrap/stage0-posix.ncl` succeeds. The output
      M2-Planet can compile a trivial C test program.

## Phase 2: GNU mes (M2-Planet → mes)

- [x] I4 Write `bootstrap/mes.ncl`: build GNU mes 0.27.x from source using
      M2-Planet and mescc-tools from stage0-posix output. Adapt build steps from
      live-bootstrap `steps/mes-0.27.1/`. Output: mes Scheme interpreter + mescc
      C compiler + mes libc.
- [ ] V2 Validate: `crunch build bootstrap/mes.ncl` succeeds. `mescc` can compile
      a C program that exercises basic stdio and arithmetic.

## Phase 3: tinycc (mes → tinycc self-hosted)

- [x] I5 Write `bootstrap/tinycc-mes.ncl`: build tinycc 0.9.26 using mes C
      compiler. Adapt from live-bootstrap `steps/tcc-0.9.26/`.
- [x] I6 Write `bootstrap/tinycc.ncl`: rebuild tinycc 0.9.27 using tinycc 0.9.26
      (self-hosting step). Adapt from live-bootstrap `steps/tcc-0.9.27/`. Output:
      a self-hosted tinycc that can build early GCC.
- [ ] V3 Validate: both tinycc derivations build. The self-hosted tinycc can
      compile a multi-file C project.

## Phase 4: Supporting tools for GCC build

- [x] I7 Write `bootstrap/make-tcc.ncl`: build GNU make 3.82 using tinycc.
      Adapt from live-bootstrap `steps/make-3.82/`.
- [x] I8 Write supporting tool derivations needed by gcc-4.0.4 build: at minimum
      `bootstrap/binutils-tcc.ncl` (binutils 2.30 from tinycc), plus any other
      tools live-bootstrap requires between tinycc and gcc-4.0.4 (sed, patch,
      gawk as needed). Each adapted from corresponding live-bootstrap `steps/`.

## Phase 5: GCC version ladder

- [x] I9 Write `bootstrap/gcc-4.0.ncl`: build gcc-4.0.4 using tinycc + make +
      binutils from Phase 4. Adapt from live-bootstrap `steps/gcc-4.0.4/`.
- [x] I10 Write supporting tool upgrades needed between gcc-4.0 and gcc-4.7:
      musl-1.1.24, updated binutils, make-4.2.1, bash, m4, flex, bison, etc.
      Each adapted from corresponding live-bootstrap steps.
- [x] I11 Write `bootstrap/gcc-4.7.ncl`: build gcc-4.7.4 using gcc-4.0.4 + musl
      + updated tools. Adapt from live-bootstrap `steps/gcc-4.7.4/`.
- [x] I12 Write `bootstrap/gcc-10.ncl`: build gcc-10.5.0 (or latest in
      live-bootstrap) using gcc-4.7.4. Adapt from live-bootstrap `steps/gcc-10.5.0/`.
      This also requires gmp, mpfr, mpc libraries.
- [ ] V4 Validate: each GCC version builds from the previous. gcc-10 can compile
      a C++ program.

## Phase 6: Modern musl + binutils from modern GCC

- [x] I13 Write `bootstrap/musl-full.ncl`: build musl-1.2.x using gcc-10.
- [x] I14 Write `bootstrap/binutils-full.ncl`: build binutils-2.41 using gcc-10.

## Phase 7: Normalized seed and integration

- [x] I15 Rewrite `bootstrap/seed.ncl` to expose the gcc-10 + musl-1.2 +
      binutils-2.41 output through the normalized seed contract (target-prefixed
      binutils in `bin/`, sysroot at `<target>/`, provider metadata).
- [ ] V5 Validate: `bootstrap/selftest.ncl` and `bootstrap/integration-test.ncl`
      pass with the from-source seed.
- [ ] V6 Validate: `crunch self-build` with the from-source seed produces
      stage1==stage2 byte-identical binaries.
- [ ] I16 Remove the musl.cc tarball URL from the codebase (keep legacy path
      behind flag until V6 passes).
- [ ] V7 Validate: full self-hosting proof (`scripts/prove-self-hosting.sh`) passes
      with the from-source seed.
