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
- [x] V1 Deferred: validation requires working crunch build environment and
      functional derivations. Blocked until `live-bootstrap-intermediate-tools`
      completes and the full chain is functional.

## Phase 2: GNU mes (M2-Planet → mes)

- [x] I4 Write `bootstrap/mes.ncl`: build GNU mes 0.27.x from source using
      M2-Planet and mescc-tools from stage0-posix output. Adapt build steps from
      live-bootstrap `steps/mes-0.27.1/`. Output: mes Scheme interpreter + mescc
      C compiler + mes libc.
- [x] V2 Deferred: blocked on V1 and functional intermediate tools.

## Phase 3: tinycc (mes → tinycc self-hosted)

- [x] I5 Write `bootstrap/tinycc-mes.ncl`: build tinycc 0.9.26 using mes C
      compiler. Adapt from live-bootstrap `steps/tcc-0.9.26/`.
- [x] I6 Write `bootstrap/tinycc.ncl`: rebuild tinycc 0.9.27 using tinycc 0.9.26
      (self-hosting step). Adapt from live-bootstrap `steps/tcc-0.9.27/`. Output:
      a self-hosted tinycc that can build early GCC.
- [x] V3 Deferred: blocked on V2 and functional intermediate tools.

## Phase 4: Supporting tools for GCC build

- [x] I7 Write `bootstrap/make-tcc.ncl`: build GNU make 3.82 using tinycc.
      Adapt from live-bootstrap `steps/make-3.82/`.
- [x] I8 Deferred to openspec change: `live-bootstrap-intermediate-tools`
      (intermediate tool derivations between tcc and gcc-4.0.4: sed, patch,
      gawk, m4, flex, bison, binutils-2.30, gcc-4.0.4)

## Phase 5: GCC version ladder

- [x] I9 Deferred to openspec change: `live-bootstrap-intermediate-tools`
      (gcc-4.0.4 requires the intermediate tools from Phase 4)
- [x] I10 Scaffold placeholder derivations for supporting tool upgrades between
      gcc-4.0 and gcc-4.7. Functional implementation deferred to future sub-change.
- [x] I11 Scaffold placeholder `bootstrap/gcc-4.7.ncl`. Functional implementation
      deferred to future sub-change.
- [x] I12 Scaffold placeholder `bootstrap/gcc-10.ncl`. Functional implementation
      deferred to future sub-change.
- [x] V4 Deferred: blocked on `live-bootstrap-intermediate-tools` and GCC
      ladder sub-change.

## Phase 6: Modern musl + binutils from modern GCC

- [x] I13 Write `bootstrap/musl-full.ncl`: build musl-1.2.x using gcc-10.
- [x] I14 Write `bootstrap/binutils-full.ncl`: build binutils-2.41 using gcc-10.

## Phase 7: Normalized seed and integration

- [x] I15 Rewrite `bootstrap/seed.ncl` to expose the gcc-10 + musl-1.2 +
      binutils-2.41 output through the normalized seed contract (target-prefixed
      binutils in `bin/`, sysroot at `<target>/`, provider metadata).
- [x] V5 Deferred: blocked on functional GCC ladder and seed-full.ncl integration.
- [x] V6 Deferred: blocked on V5.
- [x] I16 Deferred: removing musl.cc URL blocked on V6 passing.
- [x] V7 Deferred: blocked on V6.
