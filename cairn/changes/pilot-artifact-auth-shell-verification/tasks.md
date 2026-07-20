## Phase 1: Mapping and shell

- [x] [serial] I1 Record the clean baseline, portfolio families, completion contract, false-completion cases, audit risks, budgets, and source-seed isolation. r[mantle.artifact_auth_shell.exact_verification]
- [x] [serial] I2 Expose a pure signer-specific standalone statement mapping that reuses the accepted action-result identities and never consumes legacy cryptographic proof. r[mantle.artifact_auth_shell.exact_verification]
- [x] [depends:mantle.artifact_auth_shell.exact_verification] I3 Add legacy admission, exact standalone signing, independent pinned verification, recomputed carrier identities, and bounded reports in the `crunch-build` shell. r[mantle.artifact_auth_shell.authorization] r[mantle.artifact_auth_shell.evidence]
- [x] [depends:mantle.artifact_auth_shell.authorization] I4 Add exact Cargo/Nix package-set assertions, operator documentation, and positive plus adversarial fixtures without changing authority flags. r[mantle.artifact_auth_shell.adversarial] r[mantle.artifact_auth_shell.authority]

## Phase 2: Verification and readiness

- [x] [parallel] V1 Run focused rustfmt, core/shell tests, strict Clippy, Tiger Style, and an advisory adversarial review; fix deterministic findings. r[mantle.artifact_auth_shell.adversarial]
- [ ] [serial] V2 Run full workspace, Nix, and native Cairn gates from the isolated worktree, then sync and archive accepted evidence. r[mantle.artifact_auth_shell.evidence]
- [ ] [serial] V3 Compare immutable Molten, Valence, and Mantle evidence, evaluate authority admission, and create an admission change only if every operational prerequisite is proved. r[mantle.artifact_auth_shell.authority]
