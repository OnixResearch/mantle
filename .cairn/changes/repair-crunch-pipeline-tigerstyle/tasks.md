## Phase 1: Baseline and bounded evaluation

- [x] [serial] I1 [covers=build_correctness.pipeline_tiger_conformance] [evidence=evidence/validation.md] Preserve the focused eight-finding baseline and passing pre-change positive and negative pipeline tests. r[build_correctness.pipeline_tiger_conformance]
  - Evidence: `evidence/baseline-2026-09-01/` records eight findings in one source file and 58 passing tests with four ignored integration cases.
- [x] [serial] I2 [covers=build_correctness.pipeline_tiger_conformance] Bound eager message collection by the root count admitted before streaming, with positive and excess-message tests. r[build_correctness.pipeline_tiger_conformance]
  - Evidence: collection reserves the session root count, rejects an excess message before insertion, and retains the ordinary eager-evaluation tests.

## Phase 2: Registration and private interfaces

- [x] [serial] I3 [covers=build_correctness.pipeline_tiger_conformance] Extract a checked, deduplicated managed-generation plan while preserving registration order, source labels, classes, and commit timing. r[build_correctness.pipeline_tiger_conformance]
  - Evidence: checked output-plus-source capacity and first-observation deduplication precede the unchanged batch registration effect.
- [x] [serial] I4 [covers=build_correctness.pipeline_tiger_conformance] Pass the existing builder bundle and named private request records without changing public pipeline APIs or effect order. r[build_correctness.pipeline_tiger_conformance]
  - Evidence: `PipelineBuilderBundle` stays intact across the private helper; worker, managed-batch, retained-root, and result order remain unchanged.
- [x] [serial] I5 [covers=build_correctness.pipeline_tiger_conformance] Decompose cache-only builder construction and name failed-key roles without changing strict cache policy or key normalization. r[build_correctness.pipeline_tiger_conformance]
  - Evidence: cache-only construction still requires source overrides and strict hermeticity; named key input preserves existing normalization.

## Phase 3: Verification and lifecycle

- [x] [serial] V1 [covers=build_correctness.pipeline_tiger_conformance] [evidence=evidence/validation.md] Run `nix develop -c cargo test -p crunch-pipeline --lib --tests` before and after core changes, with positive and negative coverage. r[build_correctness.pipeline_tiger_conformance]
  - Evidence: 58 pre-change tests and 59 post-change tests pass; four unchanged integration tests remain ignored.
- [x] [serial] V2 [covers=build_correctness.pipeline_tiger_conformance] [evidence=evidence/validation.md] Run `nix run .#tigerstyle -- check -- -p crunch-pipeline`, repository Tiger Style, strict Clippy, formatting, caller compilation, and diff checks without allowances. r[build_correctness.pipeline_tiger_conformance]
  - Evidence: both Tiger transcripts contain zero pipeline finding and reach 36 root-library findings. Clippy, formatting, caller compilation, Nix evaluation, diff, and suppression checks pass.
- [ ] [serial] V3 [covers=build_correctness.pipeline_tiger_conformance] [evidence=evidence/validation.md] Run `nix flake check -L --builders ''` and `nix flake check -L`, preserve any later blocker, then validate, sync, archive, commit, push, and integrate. r[build_correctness.pipeline_tiger_conformance]
