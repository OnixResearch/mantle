# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.perl.5.004.05

Audited `bootstrap/perl-5.004_05-musl.ncl`:

- source pin: `https://www.cpan.org/src/5.0/perl5.004_05.tar.gz`
- expected output contract: executable `$out/bin/perl`
- declared predecessors: stage0, TinyCC/musl v2, musl 1.1.24, make, sed, coreutils, gawk, grep, Perl 5.003, and Perl 5.004_05 source
- intentional Crunch deviation: if the upstream make path cannot run, a fixed miniperl source list is compiled; this fallback now fails closed instead of hiding compiler/linker errors

No host Perl, Nix-provided Perl, or legacy interpreter output may be substituted for bootstrap proof.
