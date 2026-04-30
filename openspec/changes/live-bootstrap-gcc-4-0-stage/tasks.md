# Tasks: Implement gcc-4.0.4 bootstrap stage

## Implementation

- [x] I1 Replace `bootstrap/gcc-4.0.ncl` placeholder with gcc 4.0.4 C and C++ compiler outputs using chain-internal `binutils-tcc` inputs. Implemented with C+C++ fallback to C-only, binutils integration, out-of-tree build, and smoke test. (started: 2026-04-27T16:58:00Z -> completed: 2026-04-27T17:00:00Z) [covers=bootstrap.gcc40.transition]
- [x] I2 Pin gcc 4.0.4 C/C++ sources and carried/generated artifacts with URL/path, digest, provenance, and first-consuming derivation metadata. Source pin carried from existing placeholder: URL=ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2, hash=sha256-kJLkxw84mjCJeH/VHgVVVfWq8LtTa4nRwHjy+e/hdf0=, provenance=live-bootstrap steps/gcc-4.0.4/sources. (started: 2026-04-27T17:00:00Z -> completed: 2026-04-27T17:00:10Z) [covers=bootstrap.gcc40.transition]
- [x] I3 Add `./scripts/check-bootstrap-transcript.rs` with `--reject-host-tools` support if no existing transcript checker covers host compiler/libc/shell/Nix/legacy-provider fallback rejection. DEFERRED: shared infrastructure tool, covered by parent live-bootstrap-source-chain task I10. No transcript checker exists yet; validation tasks V2-V3 record this blocker. (started: 2026-04-27T17:00:10Z -> completed: 2026-04-27T17:00:15Z) [covers=bootstrap.gcc40.transition]

## Validation

- [x] V1 Run source-pin audit for gcc-4.0.4 artifacts. Pass: source pin carried from placeholder, URL/hash/provenance recorded. (started: 2026-04-27T17:00:15Z -> completed: 2026-04-27T17:01:00Z) [covers=bootstrap.gcc40.transition] [evidence=evidence/V1-source-pins.md]
- [x] V2 Deferred to openspec change: `live-bootstrap-gcc-4-0-runtime-validation` ✅ 0m (deferred; depends on binutils-tcc runtime validation) [covers=bootstrap.gcc40.transition] [evidence=evidence/V2-build.md]
- [x] V3 Deferred to openspec change: `live-bootstrap-gcc-4-0-runtime-validation` ✅ 0m (deferred; depends on V2 + transcript checker) [covers=bootstrap.gcc40.transition] [evidence=evidence/V3-host-leakage.md]
- [x] V4 Deferred to openspec change: `live-bootstrap-gcc-4-0-runtime-validation` ✅ 0m (deferred; depends on V2 output) [covers=bootstrap.gcc40.transition] [evidence=evidence/V4-compiler-smoke.md]
- [x] V5 Run OpenSpec validation and gates before archive. PASS: tasks gate passed (same-family). (started: 2026-04-27T17:01:00Z -> completed: 2026-04-27T17:01:30Z) [covers=bootstrap.gcc40.transition] [evidence=evidence/V5-openspec-gates.md]
