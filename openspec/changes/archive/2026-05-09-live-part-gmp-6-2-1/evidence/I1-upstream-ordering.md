# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.gmp.6.2.1

`bootstrap/gmp-6.2.1.ncl` builds GMP after the GCC 4.7.4 compiler boundary and before MPFR/MPC/GCC 10 consumers. The source pin is the GNU GMP 6.2.1 tarball and now records provenance and first-consumer notes beside the fixed-output fetch.
