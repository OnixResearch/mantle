# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.perl.5.000

- Added source provenance and first-consumer comments beside `perl_src`.
- Removed suppressed compiler/linker failures inside the miniperl fallback.
- Required either `perl` or `miniperl` to be built before install.
- Required installed `$out/bin/perl` and added an installed interpreter smoke check.
