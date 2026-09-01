## Phase 1: Baseline and contract structure

- [x] [serial] I1 [covers=build_correctness.repository_tiger_conformance] [evidence=evidence/validation.md] Preserve the complete 36-finding baseline and passing pre-change positive and negative root-library tests. r[build_correctness.repository_tiger_conformance]
  - Evidence: `evidence/baseline-2026-09-01/` records 36 findings across five files and 184 passing root-library tests.
- [ ] [serial] I2 [covers=build_correctness.repository_tiger_conformance] Make operator wire defaults, validation inputs, bounds, and invariants explicit while preserving compatibility and typed rejection. r[build_correctness.repository_tiger_conformance]
- [ ] [serial] I3 [covers=build_correctness.repository_tiger_conformance] Decompose remediation classification into ordered policy families with unchanged first-match output. r[build_correctness.repository_tiger_conformance]

## Phase 2: Protected execution and bootstrap

- [ ] [serial] I4 [covers=build_correctness.repository_tiger_conformance] Repair protected-execution predicates, conversions, arithmetic, conditions, and source validation without widening authority. r[build_correctness.repository_tiger_conformance]
- [ ] [serial] I5 [covers=build_correctness.repository_tiger_conformance] Repair seccomp and ptrace supervision with fixed-width public counts, explicit invariants, and handled response failures. r[build_correctness.repository_tiger_conformance]
- [ ] [serial] I6 [covers=build_correctness.repository_tiger_conformance] Replace production serialization panic paths and split bootstrap seed fetching at existing effect boundaries. r[build_correctness.repository_tiger_conformance]

## Phase 3: Complete repository closure

- [ ] [serial] I7 [covers=build_correctness.repository_tiger_conformance] Repeat the complete Tiger Style command and structurally repair every newly exposed first-party finding until the command exits successfully. r[build_correctness.repository_tiger_conformance]
- [ ] [serial] V1 [covers=build_correctness.repository_tiger_conformance] [evidence=evidence/validation.md] Run `nix develop -c cargo test -p mantle --lib` before and after core changes, plus serial protected-execution tests, with positive and negative coverage. r[build_correctness.repository_tiger_conformance]
- [ ] [serial] V2 [covers=build_correctness.repository_tiger_conformance] [evidence=evidence/validation.md] Run `nix build .#checks.x86_64-linux.tigerstyle --no-link -L --builders ''`, strict first-party Clippy, formatting, all-target checks, Nix evaluation, diff checks, and source suppression scans. r[build_correctness.repository_tiger_conformance]
- [ ] [serial] V3 [covers=build_correctness.repository_tiger_conformance] [evidence=evidence/validation.md] Run `nix flake check -L --builders ''` and `nix flake check -L`, preserve independent blockers, then validate, sync, archive, commit, push, and integrate. r[build_correctness.repository_tiger_conformance]
