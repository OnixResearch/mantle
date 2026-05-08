# Tasks: Complete grep 2.4 runtime validation

## Validation

- [x] V1 Re-run `bootstrap/grep-2.4-musl.ncl` with `nix shell nixpkgs#bubblewrap` and writable local state/store directories. Evidence: `evidence/V1-validation-rerun.md` plus copied focused validation artifacts from the archived binutils-tcc chain drain. [covers=bootstrap.part.grep.2.4.runtime-validation]
- [x] V2 Record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-output-path.md`; focused validation passed with output `/crunch/store/mnd29ba2jam4hwgfmrxg0k3ckxhqn2kl-grep-2.4-musl`. [covers=bootstrap.part.grep.2.4.runtime-validation]
- [x] V3 Smoke-test produced `grep`, `egrep`, and `fgrep` output contract. Evidence: `evidence/V3-output-contract-smoke.md`; builder smoke and install checks passed before validation exit 0. [covers=bootstrap.part.grep.2.4.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V4-host-leakage.md`; validation summary reports empty leakage findings. [covers=bootstrap.part.grep.2.4.runtime-validation]
- [x] V5 Run OpenSpec validation/gates before archive. Evidence: `evidence/V5-openspec-validation.md`; strict OpenSpec validation and `git diff --check` passed before archive. [covers=bootstrap.part.grep.2.4.runtime-validation]
