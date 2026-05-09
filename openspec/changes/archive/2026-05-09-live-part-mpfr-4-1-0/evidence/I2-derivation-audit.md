# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.mpfr.4.1.0

Audited `bootstrap/mpfr-4.1.0.ncl`:

- source pin: `https://ftpmirror.gnu.org/mpfr/mpfr-4.1.0.tar.xz`
- expected output contract: `lib/libmpfr.a`, `include/mpfr.h`, `include/mpf2mpfr.h`
- declared predecessors: GCC 4.7.4, binutils, musl, make, bash, coreutils, sed, grep, GMP, stage0
- strengthened smoke boundary: compile and run a static trivial executable against installed MPFR and GMP archives once prerequisites are available

No host MPFR/GMP/GCC, Nix-provided libraries, or legacy tool output may be substituted for bootstrap proof.
