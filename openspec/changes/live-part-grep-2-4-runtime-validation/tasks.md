# Tasks: Complete grep 2.4 runtime validation

## Validation

- [ ] V1 Re-run `bootstrap/grep-2.4-musl.ncl` with `nix shell nixpkgs#bubblewrap` and writable local state/store directories. [covers=bootstrap.part.grep.2.4.runtime-validation]
- [ ] V2 Record output path or failure class plus provider/fallback/placeholder fields. [covers=bootstrap.part.grep.2.4.runtime-validation]
- [ ] V3 Smoke-test produced `grep`, `egrep`, and `fgrep` output contract. [covers=bootstrap.part.grep.2.4.runtime-validation]
- [ ] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. [covers=bootstrap.part.grep.2.4.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.grep.2.4.runtime-validation]
