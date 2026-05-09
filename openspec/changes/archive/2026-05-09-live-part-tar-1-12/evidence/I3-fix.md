# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.tar.1.12

- Added source provenance and first-consumer comments beside `tar_src`.
- Added fail-closed pre-install executable and version checks.
- Added installed `$out/bin/tar` check.
- Added an archive create/extract smoke check inside the derivation.
- Updated the change spec to allow explicit prerequisite-gated evidence while forbidding host/Nix tar substitution.
