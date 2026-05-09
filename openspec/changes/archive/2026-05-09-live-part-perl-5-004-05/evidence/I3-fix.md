# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.perl.5.004.05

- Added source provenance and first-consumer comments beside `perl_src`.
- Removed suppressed compiler/linker failures inside the miniperl fallback.
- Required a produced `perl` or `miniperl` before install.
- Required installed `$out/bin/perl` and added an installed interpreter smoke check.
- Updated the change spec to allow explicit prerequisite-gated evidence while forbidding host/Nix Perl substitution.
