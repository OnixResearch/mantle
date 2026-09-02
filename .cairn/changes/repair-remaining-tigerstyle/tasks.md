## Phase 1: Baseline and contract structure

- [x] [serial] I1 [covers=build_correctness.repository_tiger_conformance] [evidence=evidence/validation.md] Preserve the complete 36-finding baseline and passing pre-change positive and negative root-library tests. r[build_correctness.repository_tiger_conformance]
  - Evidence: `evidence/baseline-2026-09-01/` records 36 findings across five files and 184 passing root-library tests.
- [x] [serial] I2 [covers=build_correctness.repository_tiger_conformance] Make operator wire defaults, validation inputs, bounds, and invariants explicit while preserving compatibility and typed rejection. r[build_correctness.repository_tiger_conformance]
  - Evidence: explicit serde default functions preserve omitted empty collections; named inputs and positive and negative validation tests preserve typed contract behavior.
- [x] [serial] I3 [covers=build_correctness.repository_tiger_conformance] Decompose remediation classification into ordered policy families with unchanged first-match output. r[build_correctness.repository_tiger_conformance]
  - Evidence: policy-family helpers retain source order, and the overlap test selects the original first source-remediation match.

## Phase 2: Protected execution and bootstrap

- [x] [serial] I4 [covers=build_correctness.repository_tiger_conformance] Repair protected-execution predicates, conversions, arithmetic, conditions, and source validation without widening authority. r[build_correctness.repository_tiger_conformance]
  - Evidence: checked root limits, ordered admission guards, checked promotion arithmetic, and source-set postconditions retain the same rejection variant and exact authority.
- [x] [serial] I5 [covers=build_correctness.repository_tiger_conformance] Repair seccomp and ptrace supervision with fixed-width public counts, explicit invariants, and handled response failures. r[build_correctness.repository_tiger_conformance]
  - Evidence: public audit counts use `u32`; callers convert explicitly; response, memory, and descendant failures remain fail-closed; 62 serial protected tests pass.
- [x] [serial] I6 [covers=build_correctness.repository_tiger_conformance] Replace production serialization panic paths and split bootstrap seed fetching at existing effect boundaries. r[build_correctness.repository_tiger_conformance]
  - Evidence: deterministic serialization fallback replaces three `expect` paths with successful-byte parity coverage; raw-seed services moved into one phase helper.

## Phase 3: Complete repository closure

- [x] [serial] I7 [covers=build_correctness.repository_tiger_conformance] Repeat the complete Tiger Style command and structurally repair every newly exposed first-party finding until the command exits successfully. r[build_correctness.repository_tiger_conformance]
  - Evidence: complete rounds moved 36 findings to four, one, and then zero; the final exact command exits 0.
- [x] [serial] V1 [covers=build_correctness.repository_tiger_conformance] [evidence=evidence/validation.md] Run `nix develop -c cargo test -p mantle --lib` before and after core changes, plus serial protected-execution tests, with positive and negative coverage. r[build_correctness.repository_tiger_conformance]
  - Evidence: 184 pre-change and 188 post-change root tests pass; 62 protected-execution tests pass with one test thread.
- [x] [serial] V2 [covers=build_correctness.repository_tiger_conformance] [evidence=evidence/validation.md] Run `nix build .#checks.x86_64-linux.tigerstyle --no-link -L --builders ''`, strict first-party Clippy, formatting, all-target checks, Nix evaluation, diff checks, and source suppression scans. r[build_correctness.repository_tiger_conformance]
  - Evidence: repository Tiger Style exits 0 with no finding; strict Clippy, formatting, all-target compilation, Nix evaluation, diff, and no-new-allowance checks pass.
- [ ] [serial] V3 [covers=build_correctness.repository_tiger_conformance] [evidence=evidence/validation.md] Run `nix flake check -L --builders ''` and `nix flake check -L`, preserve independent blockers, then validate, sync, archive, commit, push, and integrate. r[build_correctness.repository_tiger_conformance]
