# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.gcc.4.0.4

`bootstrap/gcc-4.0.ncl` is the GCC 4.0.4 bootstrap compiler part that depends on the TinyCC/Mes/musl path, binutils, Make, and early POSIX tools. It is the predecessor compiler for later GCC 4.7/GCC 10 parts.

The source pin is the GNU GCC 4.0.4 tarball. The derivation now records provenance and first-consumer notes beside the fixed-output source pin.
