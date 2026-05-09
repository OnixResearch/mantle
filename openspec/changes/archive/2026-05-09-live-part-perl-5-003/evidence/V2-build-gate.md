# V2 Perl 5.003 build gate evidence

Task-ID: V2
Covers: bootstrap.part.perl.5.003

No successful `crunch build bootstrap/perl-5.003-musl.ncl` is claimed in this closeout.

The derivation is hardened, but the declared predecessor chain depends on newly archived Perl 5.000/TinyCC/musl outputs whose full trusted runtime proof is still gated, so Perl 5.003 is not promoted here as trusted source-built interpreter proof.

No host Perl, Nix-provided Perl, or legacy interpreter output was substituted as evidence.
