## Context

The Aspen1 witness attempt for `provider-bound-release-evidence-2026-06-28-provider-remap-fixed` narrowed the full-release blocker to self-hosting bootstrap/Cargo path identity. Provider fixed-point replay produced matching provider stage digests, but the self-hosting proof produced Aspen `stage1 == stage2` digest `bbaf4413111a5e0d9b9a751ee662ace330313cd1a4e96fc6d9bcd12cf353ca47` instead of the published `02-stage2-mantle` digest `70f02150224073af2dfdabad5b697072a6399c361c8b448f0e17ee01431fc703`.

After forcing the same rustup nightly and sandbox busybox, the remaining binary difference included Cargo-generated parser paths: local `nickel-lang-parser-30857036f500e0c6/out/grammar.rs` versus Aspen `nickel-lang-parser-9482673c61447cbe/out/grammar.rs`. The GCC bootstrap output path also differed: local `j18i4pgrw10q5f99j9djqcm705dgzsyl-gcc` versus Aspen `7rqgji1ircyclbz7n9gxw9j344brhp7r-gcc`.

## Decisions

### 1. Normalize tool identity at the self-build shell boundary

**Choice:** The generated self-build script will create stable in-sandbox aliases such as `/tmp/bootstrap/gcc`, `/tmp/bootstrap/rust`, `/tmp/bootstrap/binutils`, `/tmp/bootstrap/busybox`, and `/tmp/bootstrap/bwrap`, then use those aliases in Cargo configuration, `PATH`, linker flags, and compile-time environment.

**Rationale:** Cargo fingerprints include rustflags and environment. If `-L/nix/store/<hash>-gcc/lib` changes because GCC's content-addressed path changes, Cargo can legitimately derive different build-output directories and embed those paths into generated code diagnostics. Stable aliases isolate Cargo from equivalent-but-differently-addressed bootstrap outputs.

### 2. Remap generated build paths without breaking build-script I/O

**Choice:** Add a self-build-local `RUSTC_WRAPPER` that injects `--remap-path-prefix` entries for `CARGO_TARGET_DIR`, crate `OUT_DIR`, `/tmp/build/crunch`, and stable bootstrap aliases. The wrapper must pass real paths to build scripts and only affect rustc path identity.

**Rationale:** Build scripts must run from real package roots and write to real `OUT_DIR`s, but rustc diagnostics and generated-code spans should not leak host-specific Cargo build directory hashes into the final binary.

### 3. Make GCC bootstrap output deterministic, then prove it separately

**Choice:** Harden `bootstrap/gcc.ncl` with fixed `SOURCE_DATE_EPOCH`, UTC/C locale, deterministic umask, fixed timestamp replacement for current `touch` calls, deterministic archive flags/wrappers where supported, and post-install mtime normalization. Add a focused check that builds GCC twice from fresh state and asserts the same output digest/path.

**Rationale:** Stable aliases prevent GCC path drift from poisoning Cargo, but the bootstrap chain itself should still converge. Keeping a separate GCC reproducibility check makes the root cause visible instead of hidden behind remapping.

### 4. Witness acceptance remains exact-byte and fail-closed

**Choice:** Do not accept stripped binaries, normalized binaries, or "semantically equivalent" binaries as witness outputs. Improve witness audit diagnostics to surface self-build path-drift clues, but keep the signed witness condition as exact BLAKE3 digest match for every published release output.

**Rationale:** Relaxing witness acceptance would weaken release evidence. The correct fix is to remove nondeterminism before packaging a new release artifact, then prove the exact artifact can be rebuilt.

## Risks / Trade-offs

- Stable aliases can hide tool path differences from Cargo while GCC remains nondeterministic; the separate GCC convergence task prevents that from being mistaken for full bootstrap reproducibility.
- Some crates can intentionally embed `OUT_DIR` or generated absolute paths into runtime data. The remap wrapper must include negative tests for path leakage and positive tests proving build scripts still read/write real paths.
- Full Aspen witness replay is expensive. The implementation should include cheap local guards first, then run the remote witness proof only after local deterministic checks pass.
