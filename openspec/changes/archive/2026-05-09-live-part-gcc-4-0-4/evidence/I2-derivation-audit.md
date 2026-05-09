# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.gcc.4.0.4

Audited `bootstrap/gcc-4.0.ncl` against the standard Crunch live-part source-hardening pattern and the known GCC 4.0 TinyCC/Mes boundary:

- source pin: `https://ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2`
- expected output contract: `gcc`/`cc`, installed `cc1`, installed `libgcc.a`, and a static C compile smoke
- declared predecessors: TinyCC/Mes/musl path, binutils, Make, sed/m4/grep/diffutils/bash, and stage0
- intentional Crunch deviations: deterministic stubs and source normalizations are retained to isolate the known c-parse/libtcc boundary; install and smoke checks must fail closed rather than copying partial compiler fragments

No host GCC, Nix-provided GCC, or legacy compiler output may be substituted for bootstrap proof.
