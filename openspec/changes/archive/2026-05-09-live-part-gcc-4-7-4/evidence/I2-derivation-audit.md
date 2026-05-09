# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.gcc.4.7.4

Audited `bootstrap/gcc-4.7.ncl` against the standard Crunch live-part source-hardening pattern and GCC ladder trust boundary:

- source pin: `https://ftpmirror.gnu.org/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2`
- expected output contract: C/C++ compiler entrypoints (`gcc`, `cc`, `g++`, `c++`), installed `cc1`/`cc1plus`, installed `libgcc.a`, and C/C++/C++11 compile smokes
- declared predecessor: real GCC 4.0.4 bootstrap output plus declared early toolchain inputs
- intentional Crunch deviation removed: the previous C-only configure fallback could mask the C++ provider contract and must fail closed instead

No host GCC, Nix-provided GCC, or legacy compiler output may be substituted for bootstrap proof.
