# Tasks: Complete binutils-tcc runtime validation

## Validation

- [x] V1 Re-run build preflight with `nix shell nixpkgs#bubblewrap` and writable local state/store directories. Evidence: `evidence/V1-preflight.md`, `evidence/V1-preflight.log`. [covers=bootstrap.binutils.tcc.runtime-validation]
- [x] V2 Complete or split the `bootstrap/bzip2-tcc.ncl` prerequisite build and record the Mes runtime boundary. Evidence: `evidence/V2-bzip2-tcc-boundary.md`, `evidence/validation-summary.md`, `evidence/V2-bzip2-tcc-failed-derivation.log`; blocked on active change `live-part-patch-2-5-9-runtime-validation`. [covers=bootstrap.binutils.tcc.runtime-validation]
- [ ] V3 Build every parent epoch derivation in dependency order and record command/provider/exit/output/fallback/placeholder fields. [covers=bootstrap.binutils.tcc.runtime-validation]
- [ ] V4 Run no-host-leakage audit over successful epoch transcripts. [covers=bootstrap.binutils.tcc.runtime-validation]
- [ ] V5 Run post-musl linkage checks and final binutils assembler smoke. [covers=bootstrap.binutils.tcc.runtime-validation]
- [ ] V6 Run OpenSpec validation/gates before archive. [covers=bootstrap.binutils.tcc.runtime-validation]
