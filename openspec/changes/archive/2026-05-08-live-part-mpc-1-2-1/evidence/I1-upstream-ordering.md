# I1 upstream ordering and identity evidence

Task-ID: I1
Covers: bootstrap.part.mpc.1.2.1

- Upstream source note already recorded in `openspec/changes/live-bootstrap-parts-validation.md`: `steps/manifest:154:build: mpc-1.2.1` and `steps/mpc-1.2.1/sources` uses `mpc-1.2.1.tar.gz` with SHA-256 `17503d2c395dfcf106b622dc142683c1199431d095367c6aacba6eec30340459`.
- Upstream `parts.rst` heading mismatch is recorded: line 998 says `mpc 3.2.1`, but the implemented step and source directory are `mpc-1.2.1`.
- Crunch part identity remains `bootstrap/mpc-1.2.1.ncl` and output name `mpc-1.2.1`.
- Ordering observed from direct imports: `gcc-4.7.ncl`, `binutils-tcc.ncl`, `musl-1.1.24-tcc-musl.ncl`, `make-tcc.ncl`, `bash-2.05b-tcc.ncl`, `coreutils-6.10-musl.ncl`, `sed-4.0.9-musl.ncl`, `grep-2.4-musl.ncl`, `gmp-6.2.1.ncl`, `mpfr-4.1.0.ncl`, and `stage0-posix.ncl`.
- Expected output contract hardened in this slice: `lib/libmpc.a` and `include/mpc.h` must both exist.
