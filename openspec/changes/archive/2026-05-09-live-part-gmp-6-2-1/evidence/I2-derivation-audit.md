# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.gmp.6.2.1

Audited `bootstrap/gmp-6.2.1.ncl`:

- source pin: `https://ftpmirror.gnu.org/gmp/gmp-6.2.1.tar.xz`
- expected output contract: `lib/libgmp.a`, `include/gmp.h`, and a static compile/run smoke against `-lgmp`
- declared predecessor: GCC 4.7.4 plus early bootstrap tools
- direct consumers: MPFR, MPC, and later GCC stages

No host GMP, Nix-provided GMP, or legacy library output may be substituted for bootstrap proof.
