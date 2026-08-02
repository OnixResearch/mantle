# Tasks

## Phase 1: Pure observation and classification core

- [x] [depends:realize-foreign-derivation-adapter] I1 Define bounded payload, reference, shebang, symlink, container, finding, policy, and audit-result types. r[foreign_derivation_import.castore_provenance_audit]
- [x] [serial] I2 Add pure classification for regular data, executable ELF, executable script, symlink, supported archive, supported initrd, malformed content, and unsupported executable payloads. r[foreign_derivation_import.executable_payload_classification]
- [x] [serial] I3 Add pure exact-path resolution against admitted closure identities and foreign-to-target path maps. Reject untranslated paths, escapes, missing targets, and undeclared references. r[foreign_derivation_import.executable_payload_classification]
- [x] [parallel] I4 Add positive and negative property tests for determinism, bounds, malformed bytes, non-UTF-8 content, shebangs, symlinks, path suffixes, unknown references, and unclassified executables. r[foreign_derivation_import.executable_payload_classification]

## Phase 2: Castore and container observation shell

- [x] [serial] I5 Add a thin signed-PathInfo and castore walker with bounded node, blob, byte, depth, finding, and duplicate limits. r[foreign_derivation_import.castore_provenance_audit]
- [x] [serial] I6 Add bounded format readers for the accepted archive and initrd classes. Do not execute host archive, shell, or decompression commands. r[foreign_derivation_import.castore_provenance_audit]
- [x] [serial] I7 Convert scanner output into existing build-correctness reference observations and add typed observations only for missing payload classes. r[foreign_derivation_import.castore_provenance_audit]
- [x] [parallel] I8 Add castore fixture tests for complete closures, missing blobs, duplicate nodes, symlink loops, archive traversal, decompression bounds, malformed containers, and limit exhaustion. r[foreign_derivation_import.castore_provenance_audit]

## Phase 3: Adapter report and operator surface

- [x] [serial] I9 Add the foreign realization audit command or post-realization hook with explicit policy, receipt, state, and selected-root inputs. r[foreign_derivation_import.provenance_audit_receipt]
- [x] [serial] I10 Emit deterministic `mantle-foreign-provenance-audit-v1` and link it from realization receipts without rewriting the original build report. r[foreign_derivation_import.provenance_audit_receipt]
- [x] [parallel] I11 Update trust-model and machine-artifact documentation with realized versus provenance-audited states, scan limits, failure meaning, and non-claims. r[foreign_derivation_import.provenance_audit_receipt]

## Phase 4: Verification and lifecycle evidence

- [x] [serial] V1 Run focused pure classifier, castore walker, build-correctness, and foreign CLI tests. Record exact output in `cairn/changes/audit-foreign-realization-provenance/evidence/verification.md`. r[foreign_derivation_import.castore_provenance_audit]
- [x] [serial] V2 Run one passing two-node audit and negative audits for leftover `/gnu/store`, missing shebang target, malformed ELF, symlink escape, hidden archive executable, incomplete closure, and every configured limit. Record exact commands, statuses, receipt identities, and a short inline evidence summary. r[foreign_derivation_import.provenance_audit_receipt]
- [x] [serial] V3 Run `nix develop -c cargo test -p mantle --bin mantle build_correctness`, `nix develop -c cargo test -p mantle --test foreign_import_cli`, `git diff --check`, focused first-party formatting and Clippy, Cairn validation, all three gates, and Tracey coverage. Record exact output before archive. r[foreign_derivation_import.provenance_audit_receipt]
