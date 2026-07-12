## Tasks

- [x] [serial] r[mantle.ast_grep_structural_rails.toolchain] Add a pinned ast-grep package/toolchain profile with reproducible binary identity.
- [x] [depends:ast-grep-toolchain] r[mantle.ast_grep_structural_rails.sidecar] Define ast-grep rule-test and scan sidecar fields for build/report evidence.
- [x] [depends:ast-grep-sidecar] r[mantle.ast_grep_structural_rails.identity] Bind rule bundles, scan scopes, sidecars, and receipts with BLAKE3 identity.
- [x] [depends:ast-grep-sidecar] r[mantle.ast_grep_structural_rails.shell_boundary] Keep ast-grep process execution and output reads in build/CLI shell code.
- [x] [depends:ast-grep-identity] r[mantle.ast_grep_structural_rails.fixtures] Add positive and negative sidecar fixtures for valid scans, stale hashes, wrong tool identity, missing non-claims, and overclaimed labels.
- [x] [depends:ast-grep-fixtures] r[mantle.ast_grep_structural_rails.validation] Run package identity smoke, fixture validation, Cairn gates, and focused Mantle checks.
