# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.gawk.3.0.4

Audited `bootstrap/gawk-3.0.4-musl.ncl` against the standard Crunch live-part source-hardening pattern:

- source pin: `https://mirrors.kernel.org/gnu/gawk/gawk-3.0.4.tar.gz`
- expected output contract: `bin/gawk` and `bin/awk` symlink
- declared predecessors: TinyCC/Mes/musl and Make 3.82 chain plus `sed-4.0.9-musl`
- intentional Crunch deviation: direct compile/link/install logic is retained instead of a full autotools install path, but required source-object, parser-object, link, output, and smoke checks must fail closed

No host awk/gawk, Nix-provided awk/gawk, host GCC, or legacy compiler output may be substituted for bootstrap proof.
