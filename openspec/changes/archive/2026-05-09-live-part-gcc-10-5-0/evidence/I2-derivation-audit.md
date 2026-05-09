# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.gcc.10.5.0

Audited `bootstrap/gcc-10.ncl` against the standard Crunch live-part source-hardening pattern and the stronger compiler output boundary:

- source pin: `https://ftpmirror.gnu.org/gcc/gcc-10.5.0/gcc-10.5.0.tar.xz`
- expected output contract: C and C++ compiler entrypoints (`gcc`, `cc`, `g++`, `c++` or target-prefixed equivalents), installed `libgcc.a`, and C/C++ static compile smokes
- declared predecessors: GCC 4.7.4, binutils, musl, Make, GMP, MPFR, MPC, Autoconf/Automake/Libtool, Flex/Bison/Perl/Coreutils/Gawk/Grep/Diffutils/Bash/Patch/Tar/Bzip2, and stage0
- intentional Crunch deviation: direct pass1 configure/build flags are retained, but target-libgcc, install, compiler entrypoints, and smoke checks must fail closed

No host GCC, Nix-provided GCC, or legacy compiler output may be substituted for bootstrap proof.
