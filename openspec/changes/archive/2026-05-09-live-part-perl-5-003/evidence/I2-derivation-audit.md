# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.perl.5.003

Audited `bootstrap/perl-5.003-musl.ncl`:

- source pin: `https://github.com/Perl/perl5/archive/perl-5.003.tar.gz`
- expected output contract: executable `$out/bin/perl` and optional fallback-built miniperl only as an intermediate
- declared predecessors: stage0, TinyCC/musl v2, musl 1.1.24, make, sed, coreutils, gawk, grep, Perl 5.000, and Perl 5.003 source
- intentional Crunch deviation: if the upstream make path cannot run, a fixed miniperl source list is compiled; this fallback now fails closed instead of hiding compiler/linker errors

No host Perl, Nix-provided Perl, or legacy interpreter output may be substituted for bootstrap proof.
