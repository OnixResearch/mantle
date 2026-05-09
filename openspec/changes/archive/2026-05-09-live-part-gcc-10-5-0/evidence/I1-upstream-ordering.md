# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.gcc.10.5.0

`bootstrap/gcc-10.ncl` is the GCC 10.5.0 pass1/final-provider compiler part, ordered after the GCC 4.7.4, GMP, MPFR, MPC, binutils, musl, Make, and generated-tool predecessor chain.

The source pin is the GNU GCC 10.5.0 tarball. The derivation now records provenance and first-consumer notes beside the fixed-output source pin.
