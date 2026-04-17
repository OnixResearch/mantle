# Tasks: Explore compiled evaluation backends

## Phase 1: Record future-work boundary

- [x] Write proposal describing where Cranelift or LLVM could fit in crunch
- [x] Write a future-work delta spec for compiled evaluation backends
- [x] Write design notes covering boundary, priorities, and non-goals

## Phase 2: Evidence before implementation

- [x] Add or identify benchmarks that separate Nickel evaluation cost from
      build/store/sandbox cost on representative workloads
  - Evidence: `docs/benchmark-suite.md` now serves as the compiled-eval gate,
    with `eval-fetch-git` as the narrow eval probe,
    `workflow-package-set-eval-build-graph` as the multi-phase workflow gate,
    and the remaining suite entries as non-eval controls.
- [x] Define success criteria that would justify evaluator codegen work
  - Evidence: `design.md` decision 7 now requires both a dominant eval share
    on `workflow-package-set-eval-build-graph` (>90% of `total_wall_ns`) and a
    future >=2x eval-phase speedup on `eval-fetch-git` with no non-eval
    regression past `benchmark_compare` thresholds.
- [x] Identify which current workloads are interpreter-bound versus I/O-bound
  - Evidence: `cargo run --example benchmark_suite -- --bundle-out
    target/benchmarks/compiled-eval-gate.json --repeat-count 2` produced
    `openspec/changes/explore-compiled-eval-backends/evidence/compiled-eval-gate.json`
    and `.../benchmark-suite-baseline.txt`, showing `eval-fetch-git` at 99.98%
    eval time and `workflow-package-set-eval-build-graph` at 99.11% eval time;
    no current checked-in eval-bearing workload is I/O-bound, and the other
    suite entries remain non-eval controls.

## Phase 3: Backend experiment plan

- [x] Sketch a backend-neutral internal interface in `crunch-eval` for future
      experiments without changing downstream crates
  - Evidence: `crates/crunch-eval/src/backend.rs` now defines private
    `EvalBackend`, `EvalRequest`, and `NickelBackend` types, and
    `crates/crunch-eval/src/lib.rs` delegates the existing `evaluate*` helpers
    through `default_backend()` with no downstream call-site changes.
  - Validation:
    `openspec/changes/explore-compiled-eval-backends/evidence/backend-interface-tests.txt`
    captures `cargo test -p crunch-eval --lib` → `test result: ok. 35 passed`
    and `cargo test -p crunch --test examples_eval` → `test result: ok. 4
    passed`.
- [x] Prototype a narrow Cranelift-backed derivation-evaluation subset if the
      profiling gate is met, while keeping benchmark/profiling helpers in
      `examples/` or test-only paths instead of `src/lib.rs`
  - Evidence: `crates/crunch-eval/Cargo.toml` now adds optional
    `cranelift-proto` dependencies only behind a feature flag;
    `crates/crunch-eval/src/cranelift_proto.rs` implements a flat-derivation
    subset; `crates/crunch-eval/src/backend.rs` and `src/lib.rs` expose the
    experimental path without changing default callers.
  - Validation:
    `openspec/changes/explore-compiled-eval-backends/evidence/cranelift-prototype-tests.txt`
    captures `cargo test -p crunch-eval --lib --features cranelift-proto` →
    `test result: ok. 41 passed`, `cargo test -p crunch-eval --lib` →
    `test result: ok. 35 passed`, and `cargo test -p crunch --test
    examples_eval` → `test result: ok. 4 passed`.
- [ ] Run `benchmark_compare` against
      `openspec/changes/explore-compiled-eval-backends/evidence/compiled-eval-gate.json`
      after prototype integration and confirm non-eval workloads stay within
      the documented comparison thresholds
  - Current evidence:
    `openspec/changes/explore-compiled-eval-backends/evidence/post-prototype-benchmark-compare.txt`
    still reports `build-graph-package-set.build_graph_wall_ns` at `+26.15%`,
    so this task remains open pending a stable regression story.
- [ ] Reassess LLVM only if Cranelift cannot meet measured goals or required
      optimization/target/toolchain constraints
- [ ] Define interpreter-vs-compiled equivalence tests for contracts, merges,
      import resolution, package sets, recursive records, and nested
      derivation inputs, using `/nix/store` for checked-in seed-backed
      conversion fixtures when required

## Validation

- [x] Run `openspec validate explore-compiled-eval-backends`
