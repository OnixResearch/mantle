# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.seed.full

`bootstrap/seed-full.ncl` normalizes the intended full-source seed provider after the GCC 10.5.0, musl 1.2.5, and binutils 2.41 source-chain parts. It has no independent tarball fetch; its source-chain provenance is inherited from those declared predecessor derivations.
