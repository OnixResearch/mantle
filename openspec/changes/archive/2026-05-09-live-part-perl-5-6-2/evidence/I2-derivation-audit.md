# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.perl.5.6.2

Audited `bootstrap/perl-5.6.2-musl.ncl`:

- source pin: `https://www.cpan.org/src/5.0/perl-5.6.2.tar.gz`
- expected output contract: executable `$out/bin/perl`
- declared predecessors include stage0, TinyCC/musl v2, musl, make, sed, coreutils, gawk, grep, Perl 5.005_03, and Perl 5.6.2 source
- intentional Crunch deviation: if the upstream make path cannot run, a fixed miniperl source list is compiled; this fallback now fails closed instead of hiding compiler/linker errors

No host Perl, Nix-provided Perl, or legacy interpreter output may be substituted for bootstrap proof.
