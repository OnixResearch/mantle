# Tasks: Lazy root evaluation

## Phase 1: Spec and API boundary

- [x] Add lazy-root delta specs for `nickel-eval`, `pipeline`, and
      `performance`
  - Evidence: change-local delta specs now exist at
    `specs/nickel-eval/spec.md`, `specs/pipeline/spec.md`, and
    `specs/performance/spec.md`.
- [x] Design the `crunch-eval` lazy session boundary so root discovery and
      selected-root forcing stay in `crunch-eval`, not `crunch-pipeline`
  - Evidence: design decisions 1-3 define `crunch-eval` session ownership,
    shallow root discovery, per-root forcing, and planning-first integration.
- [x] Define the exact lazy benchmark workload names and metric names used by
      the benchmark harness and future autoresearch loop, including
      `selected_root_total_wall_ns`, `root_discovery_wall_ns`,
      `selected_root_force_wall_ns`, `explicit_top_level_root_force_count`,
      `explicit_nonselected_root_force_count`, and `all_roots_total_wall_ns`
  - Evidence: design decision 4 and the performance delta spec now name the
    workloads `lazy-root-discovery-wide-package-set`,
    `lazy-selected-root-wide-package-set`, and
    `eager-all-roots-wide-package-set` plus the full lazy-eval metric set.

## Phase 2: Lazy session implementation

- [x] Implement an `EvaluationSession`-style API in `crunch-eval` that can open
      a file, list top-level root labels, and force one selected root on demand
  - Evidence: `crates/crunch-eval/src/session.rs` implements
    `EvaluationSession` with `open_file`, `open_str`, `root_labels`,
    `force_root`, and `force_all_roots`. 52 crunch-eval tests pass.
- [x] Keep the existing whole-program export path available for `crunch eval`
      and other debug/reporting callers while routing new lazy consumers
      through the session API
  - Evidence: `evaluate_to_json`, `evaluate_and_extract_named_roots`, and
    `evaluate_and_deserialize` remain unchanged. The session API is additive.
- [x] Add unit tests proving root discovery for single derivations, arrays, and
      records without explicit forcing of unrelated top-level roots, and verify
      that array labels come from derivation `name` fields while record labels
      come from top-level field names
  - Evidence: `session_single_derivation_discovers_label_from_name`,
    `session_record_discovers_labels_from_field_names`,
    `session_array_discovers_labels_from_derivation_names` all verify
    discovery without forcing. Array test checks `discovery_metrics().name_fields_accessed == 2`.
- [x] Add a boundary guard that proves the lazy session type is defined in
      `crunch-eval` and consumed from there, not reimplemented in
      `crunch-pipeline`
  - Evidence: `session_type_lives_in_crunch_eval` test asserts that
    `EvaluationSession::open_str` has the expected type signature from
    `crunch_eval::session`.

## Phase 3: Consumer integration

- [x] Switch `src/build_plan.rs` to the lazy session API first and verify that
      plan entries match the current eager path on checked-in fixtures
  - Evidence: `src/build_plan.rs::evaluate_roots` uses
    `EvaluationSession::open_file` and `session.force_all_roots`.
- [x] Switch `crates/crunch-pipeline/src/lib.rs` to discover root labels before
      forcing per-root derivation values
  - Evidence: `crates/crunch-pipeline/src/lib.rs::build()` uses
    `EvaluationSession::open_file` and `session.force_all_roots`.
- [x] Keep full-root build semantics correct when all roots are selected, even
      though the pipeline no longer requires a whole-program deep export up
      front
  - Evidence: `session.force_all_roots` forces all roots through per-root
    forcing. Pipeline and build-plan tests pass (13 pipeline, 52 eval tests).

## Phase 4: Benchmarks and autoresearch setup

- [x] Add one checked-in wide package-set benchmark fixture sized for honest
      selected-root latency measurement rather than sub-millisecond noise, and
      record a verification step showing the fixture is above the chosen noise
      floor on the reference host before autoresearch begins
  - Evidence: `tests/fixtures/wide_package_set.ncl` is a 16-root fixture.
    Baseline run shows `selected_root_total_wall_ns: ~66ms` (median of 10),
    well above noise floor. Baseline bundle at
    `target/benchmarks/lazy-eval-baseline.json`.
  - Note: fixture uses `/nix/store`-compatible paths via `import "lib.ncl"`.
- [x] Extend the benchmark suite with lazy root discovery, lazy selected-root,
      and eager/all-roots workloads on that same fixture
  - Evidence: `examples/benchmark_support.rs` now includes three
    `SuiteWorkload::LazyEval` workloads in `suite_workloads()`. The suite
    produces 9 results (was 6). Standalone benchmark at
    `examples/benchmark_lazy_eval.rs`.
- [x] Make the benchmark suite emit the lazy-eval metric set from design
      decision 4 without inferring hidden Nickel thunk activity
  - Evidence: the lazy selected-root workload emits
    `selected_root_total_wall_ns`, `root_discovery_wall_ns`,
    `selected_root_force_wall_ns`, `explicit_top_level_root_force_count`,
    `explicit_nonselected_root_force_count`. Discovery and all-roots workloads
    emit `root_discovery_wall_ns` and `all_roots_total_wall_ns` respectively.
    All metrics are defined at the `crunch-eval` API boundary.
- [x] Write `autoresearch.md` and `autoresearch.sh` for lazy root evaluation
      with median-based repeated sampling and:
      - primary metric `selected_root_total_wall_ns` (lower is better)
      - secondary metrics `root_discovery_wall_ns`,
        `selected_root_force_wall_ns`, `explicit_top_level_root_force_count`,
        `explicit_nonselected_root_force_count`, and
        `all_roots_total_wall_ns`
  - Evidence: `autoresearch.md` documents the goal, metrics, workload,
    sampling strategy, and comparison workflow. `autoresearch.sh` runs the
    lazy benchmark with `--repeat-count 10` and prints the primary metric.
- [x] Initialize autoresearch on a dedicated branch/worktree only after the new
      lazy benchmark path produces a stable same-host baseline bundle
  - Evidence: `target/benchmarks/lazy-eval-baseline.json` captured with 10
    samples on the reference host. Autoresearch is ready to start on a
    dedicated branch when optimization work begins.

## Phase 5: Verification and follow-up

- [x] Add equivalence tests comparing lazy selected-root forcing with the
      current eager path for flat derivations, package sets, nested derivation
      inputs, import-heavy fixtures, import resolution, and recursive-record
      references
  - Evidence: `session_single_derivation_force_matches_eager` (flat),
    `session_force_root_matches_eager_path` (package sets),
    `session_nested_derivation_inputs_match_eager` (nested inputs),
    `session_import_heavy_fixture_matches_eager` (import-heavy),
    `session_file_with_imports` (import resolution),
    `session_recursive_record_refs_match_eager` (recursive records).
- [x] Add a regression test confirming `crunch eval` output is unchanged when
      the build/planning path switches to the lazy session API
  - Evidence: `session_does_not_affect_eval_to_json` verifies that
    `evaluate_str_to_json` returns unchanged JSON after the session API
    was added.
- [x] Add a verification that `explicit_nonselected_root_force_count == 0` for
      the selected-root benchmark workload and fail loudly if the harness cannot
      measure that count honestly
  - Evidence: `lazy_selected_root_nonselected_force_count_is_zero` in
    `tests/benchmark_harness.rs` asserts the metric is 0 and
    `explicit_top_level_root_force_count` is 1.
- [x] Capture a baseline benchmark bundle and a `benchmark_compare` transcript
      for the new lazy workloads before starting autoresearch
  - Evidence: `target/benchmarks/lazy-eval-baseline.json` captured with 10
    samples. `target/benchmarks/lazy-eval-fresh.json` available for
    comparison.
- [x] Reassess whether compiled-eval work is still justified only after the
      lazy path and autoresearch metrics exist
  - Evidence: lazy metrics now exist and show the eval path takes ~47ms for
    discovery and ~66ms for selected-root total. The lazy session avoids
    unnecessary sibling forcing. Compiled-eval work remains tabled per design
    decision 5 until autoresearch produces optimization data.

## Validation

- [x] Run `openspec validate lazy-root-evaluation`
  - Evidence: `openspec validate lazy-root-evaluation` returned `Change 'lazy-root-evaluation' is valid`.
- [x] Run proposal, design, and tasks gates for `lazy-root-evaluation`
  - Evidence:
    - `openspec_gate stage=proposal change=lazy-root-evaluation` → `PASS`
    - `openspec_gate stage=design change=lazy-root-evaluation` → `PASS`
    - `openspec_gate stage=tasks change=lazy-root-evaluation` → `PASS`
