# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.flex.2.6.4

Audited `bootstrap/flex-2.6.4-musl.ncl` against the standard Crunch live-part source-hardening pattern:

- source pin: `https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz`
- expected output contract: `bin/flex`, `bin/lex`, `bin/flex++`, `lib/libfl.a`, and conditional `include/FlexLexer.h` when present in the source tree
- declared predecessors: TinyCC/Mes/musl and Make 3.82 chain plus `flex-2.5.11-musl`
- intentional Crunch deviation: direct compile/link/install logic is retained instead of a full autotools install path, but required source-object and output checks must fail closed

No host flex, Nix-provided flex, host GCC, or legacy compiler output may be substituted for bootstrap proof.
