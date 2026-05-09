# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.tar.1.12

Audited `bootstrap/tar-tcc.ncl`:

- source pin: `https://ftpmirror.gnu.org/tar/tar-1.12.tar.gz`
- expected output contract: executable `$out/bin/tar`
- declared predecessors: stage0, TinyCC 0.9.27, gzip 1.2.4, and tar 1.12 source
- intentional Crunch deviations: fixed `config.h` metadata, a `getdate_stub.c` replacement to avoid yacc/bison, direct object compilation, and root-level object copies for the Mes-linked TinyCC directory-component link limitation

No host tar, Nix-provided tar, or legacy archive tool output may be substituted for bootstrap proof.
