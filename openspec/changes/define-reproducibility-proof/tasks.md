## Phase 1: Reproducibility proof contract

- [x] [serial] Finalize the canonical reproducibility proof report schema and proof-class enum.
  - Evidence: schema/docs define `bundle-consistent`, `self-proof-valid`, `self-rebuild-match`, `external-witness-match`, and `policy-satisfied` without over-claiming.
- [x] [depends:schema] Implement report creation/verification or wire the existing release verification output to emit the canonical report.
  - Evidence: valid release evidence plus a clean matching rebuild emits `self-rebuild-match`; unsupported recipes fail closed.
- [x] [depends:implementation] Add positive and negative tests for matching rebuilds, mismatched digest sets, unknown recipe identity, incomplete witness material, and weaker-class fallback.
  - Evidence: targeted tests demonstrate stronger classes are impossible without the required evidence.
- [x] [depends:tests] Update docs/examples to explain how to prove reproducibility and how that differs from full-source bootstrap or policy sufficiency.
  - Evidence: release docs name the proof classes, BLAKE3 output comparisons, replayable recipes, and environment assumptions.
- [x] [depends:verification] Run targeted tests, formatting, OpenSpec validation, and any release-verification docs/example checks.
  - Evidence: final transcript names each passing check and any deferred long-running proof separately.
- [ ] [depends:archive] Sync/archive this OpenSpec change after the verified implementation lands.
  - Evidence: delta synced to `openspec/specs/release-verification-tech/spec.md` and archived with the implementation commit.
