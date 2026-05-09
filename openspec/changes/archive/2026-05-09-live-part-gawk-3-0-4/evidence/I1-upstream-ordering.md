# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.gawk.3.0.4

`bootstrap/gawk-3.0.4-musl.ncl` is the musl-linked Gawk part used by later Autoconf/Automake-generated source processing. It depends on the TinyCC/Mes/musl and Make predecessor chain plus `sed-4.0.9-musl`.

The source pin is the GNU Gawk 3.0.4 release tarball. The derivation now records provenance and first-consumer notes beside the fixed-output source pin.
