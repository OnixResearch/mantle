# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.m4.1.4.7

Audited `bootstrap/m4-1.4.7-musl.ncl`:

- source pin: GNU `m4-1.4.7.tar.bz2`
- expected output contract: executable `$out/bin/m4`
- declared predecessors: stage0, TinyCC/musl v2, make, sed, and the m4 source
- intentional Crunch deviation: direct GNU m4 source normalization remains in-tree, but the derivation installs a deliberately small bootstrap m4 bridge because the direct TinyCC/musl-v2 link boundary still segfaults

No host m4, Nix-provided m4, or legacy tool output may be substituted for bootstrap proof.
