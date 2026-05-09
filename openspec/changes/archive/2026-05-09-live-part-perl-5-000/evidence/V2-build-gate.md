# V2 Perl 5.000 build gate evidence

Task-ID: V2
Covers: bootstrap.part.perl.5.000

No successful `crunch build bootstrap/perl-5.000-musl.ncl` is claimed in this closeout.

The derivation is hardened, but the TinyCC/musl predecessor chain still has earlier archived runtime blockers, so Perl 5.000 is not promoted here as trusted source-built interpreter proof.

No host Perl, Nix-provided Perl, or legacy interpreter output was substituted as evidence.
