## Phase 1: Baseline and bounded evaluation

- [x] [serial] I1 [covers=build_correctness.pipeline_tiger_conformance] [evidence=evidence/validation.md] Preserve the focused eight-finding baseline and passing pre-change positive and negative pipeline tests. r[build_correctness.pipeline_tiger_conformance]
  - Evidence: `evidence/baseline-2026-09-01/` records eight findings in one source file and 58 passing tests with four ignored integration cases.
- [ ] [serial] I2 [covers=build_correctness.pipeline_tiger_conformance] Bound eager message collection by the root count admitted before streaming, with positive and excess-message tests. r[build_correctness.pipeline_tiger_conformance]

## Phase 2: Registration and private interfaces

- [ ] [serial] I3 [covers=build_correctness.pipeline_tiger_conformance] Extract a checked, deduplicated managed-generation plan while preserving registration order, source labels, classes, and commit timing. r[build_correctness.pipeline_tiger_conformance]
- [ ] [serial] I4 [covers=build_correctness.pipeline_tiger_conformance] Pass the existing builder bundle and named private request records without changing public pipeline APIs or effect order. r[build_correctness.pipeline_tiger_conformance]
- [ ] [serial] I5 [covers=build_correctness.pipeline_tiger_conformance] Decompose cache-only builder construction and name failed-key roles without changing strict cache policy or key normalization. r[build_correctness.pipeline_tiger_conformance]

## Phase 3: Verification and lifecycle

- [ ] [serial] V1 [covers=build_correctness.pipeline_tiger_conformance] [evidence=evidence/validation.md] Run `nix develop -c cargo test -p crunch-pipeline --lib --tests` before and after core changes, with positive and negative coverage. r[build_correctness.pipeline_tiger_conformance]
- [ ] [serial] V2 [covers=build_correctness.pipeline_tiger_conformance] [evidence=evidence/validation.md] Run `nix run .#tigerstyle -- check -- -p crunch-pipeline`, repository Tiger Style, strict Clippy, formatting, caller compilation, and diff checks without allowances. r[build_correctness.pipeline_tiger_conformance]
- [ ] [serial] V3 [covers=build_correctness.pipeline_tiger_conformance] [evidence=evidence/validation.md] Run `nix flake check -L --builders ''` and `nix flake check -L`, preserve any later blocker, then validate, sync, archive, commit, push, and integrate. r[build_correctness.pipeline_tiger_conformance]
