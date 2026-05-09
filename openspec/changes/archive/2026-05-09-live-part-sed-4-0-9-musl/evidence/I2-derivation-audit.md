# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.sed.4.0.9.musl

Audited `bootstrap/sed-4.0.9-musl.ncl`:

- source pin: `https://ftpmirror.gnu.org/sed/sed-4.0.9.tar.gz`
- expected output contract: executable `$out/bin/sed`
- declared predecessors: stage0, TinyCC/musl v2, musl 1.1.24, make, prior `sed-tcc`, and sed 4.0.9 source
- intentional Crunch deviation: the current derivation preserves an explicit `sed-tcc` bridge because the TinyCC/musl source compile boundary is documented as blocked on GNU sed 4.0.9 getline replacement

The bridge is hardened and smoke-checked, but it is not claimed as musl source-build proof.
