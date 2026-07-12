## Design

1. Add ast-grep to Mantle/Onix-facing package and development-shell surfaces with an explicit version and binary identity.
2. Define a stable evidence sidecar shape for repository-owned ast-grep `test` and `scan` executions: tool identity, command identity, rule bundle BLAKE3 hash, scan scope, output format, finding summary, and non-claims.
3. Keep source rule files and rule-pack policy in the owning repository. Mantle records package and execution evidence; it does not own rule semantics.
4. Use BLAKE3 for Mantle-owned sidecar and bundle identities while preserving any upstream or tool-emitted identifiers as metadata.
5. Integrate ast-grep outputs into build/report evidence only as declared structural evidence, not as proof of source correctness or build correctness.
6. Add positive fixtures for a valid scan/test sidecar and negative fixtures for mismatched tool identity, stale bundle hash, missing non-claims, and overclaimed release labels.
7. Keep CLI/build shell code responsible for invoking ast-grep and reading outputs; pure evidence validators consume already-loaded DTOs.

## Non-Goals

- No Mantle-owned ast-grep rule catalog.
- No semantic interpretation of source matches.
- No automatic codemod execution during builds.
