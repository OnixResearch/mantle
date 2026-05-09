# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.musl.1.2.5.full

Audited `bootstrap/musl-full.ncl`:

- source pin: `https://musl.libc.org/releases/musl-1.2.5.tar.gz`
- expected output contract: `lib/libc.a`, startup objects, and installed headers
- declared predecessors: GCC 10.5.0, binutils, make, bash, coreutils, sed, grep, stage0, and older musl
- strengthened smoke boundary: statically link and run a trivial executable against the produced musl headers/archive/startup objects once trusted prerequisites exist

No host musl/GCC, Nix-provided libc objects, or legacy tool output may be substituted for bootstrap proof.
