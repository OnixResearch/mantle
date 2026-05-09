# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.perl.5.000

Audited `bootstrap/perl-5.000-musl.ncl`:

- source pin: `https://github.com/Perl/perl5/archive/perl-5.000.tar.gz`
- expected output contract: executable `$out/bin/perl` and optional `$out/bin/miniperl` when fallback path is used
- declared predecessors: stage0, TinyCC/musl v2, make, sed, coreutils, gawk, grep, diffutils, and Perl source
- intentional Crunch deviation: if the old makefile cannot run, a fixed miniperl source list is compiled; this fallback now fails closed instead of suppressing compiler/linker errors

No host Perl, Nix-provided Perl, or legacy interpreter output may be substituted for bootstrap proof.
