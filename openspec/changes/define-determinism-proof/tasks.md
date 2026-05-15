## Phase 1: Contract and schema

- [x] [serial] Define deterministic-build proof receipt types, canonical serialization, closed verdict enum, and BLAKE3 digest set comparison.
- [x] [depends:schema] Wire strict-mode proof attempts so practical/impure runs cannot be promoted to deterministic claims.
- [x] [depends:schema] Add clean-store repeated build orchestration that prevents reusing the derivation-under-test output.
- [x] [depends:schema] Add ambient host perturbation matrix coverage for environment, cwd, temp dirs, and umask.

## Phase 2: Release integration and docs

- [x] [depends:schema] Teach release verification to consume deterministic proof receipts for all required artifacts without broadening claim scope.
- [x] [depends:schema] Update README/operator docs to distinguish self-rebuild match, deterministic-release claim, and global/Nix-like determinism.
- [x] [depends:schema] Add positive and negative regression tests for deterministic-match, mismatch, missing evidence, host leakage, and impure-mode verdicts.
- [ ] [depends:verification] Validate and archive this OpenSpec after implementation lands.
