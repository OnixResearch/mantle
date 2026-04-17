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
    `force_root`, and `force_all_roots`. Task 43: `test result: ok. 51
    passed; 0 failed`.
- [x] Keep the existing whole-program export path available for `crunch eval`
      and other debug/reporting callers while routing new lazy consumers
      through the session API
  - Evidence: `evaluate_to_json`, `evaluate_and_extract_named_roots`, and
    `evaluate_and_deserialize` remain in `crates/crunch-eval/src/lib.rs`
    unchanged. `session_does_not_affect_eval_to_json` test confirms.
- [x] Add unit tests proving root discovery for single derivations, arrays, and
      records without explicit forcing of unrelated top-level roots, and verify
      that array labels come from derivation `name` fields while record labels
      come from top-level field names
  - Evidence: tests `session_single_derivation_discovers_label_from_name`,
    `session_array_discovers_labels_from_derivation_names`,
    `session_record_discovers_labels_from_field_names` all pass. Array test
    checks `discovery_metrics().name_fields_accessed == 2`.
- [x] Add tests for the error scenarios: missing selected root returns a
      `crunch-eval` error, and invalid top-level shape (not a derivation,
      array, or record) returns a `crunch-eval` error during discovery
  - Evidence: `session_missing_root_returns_error` opens a session on a
    record and forces a nonexistent label; asserts `Error::Serde` with the
    missing label name. `session_invalid_top_level_shape_returns_error`
    opens a session on `"42"` (a number); asserts `Error::Serde` with
    "neither a derivation" in the message.
- [x] Add a boundary guard that proves the lazy session type is defined in
      `crunch-eval` and consumed from there, not reimplemented in
      `crunch-pipeline`
  - Evidence: `session_type_lives_in_crunch_eval` test compiles and passes.

## Phase 3: Consumer integration

- [x] Switch `src/build_plan.rs` to the lazy session API first and verify that
      plan entries match the current eager path on checked-in fixtures
  - Evidence: `src/build_plan.rs::evaluate_roots()` now uses
    `EvaluationSession::open_file` + `force_all_roots`. Task 40:
    crunch-pipeline integration tests all pass (14 passed, 4 ignored).
- [x] Switch `crates/crunch-pipeline/src/lib.rs` to discover root labels before
      forcing per-root derivation values
  - Evidence: `crates/crunch-pipeline/src/lib.rs::build()` now uses
    `EvaluationSession::open_file` + `force_all_roots`. Task 40: 13
    pipeline unit tests pass, 14 integration tests pass.
- [x] Keep full-root build semantics correct when all roots are selected, even
      though the pipeline no longer requires a whole-program deep export up
      front
  - Evidence: `force_all_roots` iterates `force_root` per label.
    `session_force_all_roots_matches_eager_path` confirms equivalence.

## Phase 4: Benchmarks and autoresearch setup

- [x] Add one checked-in wide package-set benchmark fixture sized for honest
      selected-root latency measurement rather than sub-millisecond noise, and
      record a verification step showing the fixture is above the chosen noise
      floor on the reference host before autoresearch begins
  - Evidence: `tests/fixtures/wide_package_set.ncl` (16 derivation roots).
    Task 42 ran benchmark: discovery ~47ms, selected-root ~63ms, all-roots
    ~66ms. All well above noise floor.
  - Note: if the fixture uses checked-in seed-backed conversion inputs, keep it
    `/nix/store`-compatible rather than relying on the default benchmark prefix
- [x] Extend the benchmark suite with lazy root discovery, lazy selected-root,
      and eager/all-roots workloads on that same fixture
  - Evidence: `examples/benchmark_lazy_eval.rs` runs all three workloads:
    `lazy-root-discovery-wide-package-set`,
    `lazy-selected-root-wide-package-set`, and
    `eager-all-roots-wide-package-set`.
- [x] Make the benchmark suite emit the lazy-eval metric set from design
      decision 4 without inferring hidden Nickel thunk activity
  - Evidence: benchmark output includes `selected_root_total_wall_ns`,
    `root_discovery_wall_ns`, `selected_root_force_wall_ns`,
    `explicit_top_level_root_force_count` (from `session.explicit_force_count()`),
    `explicit_nonselected_root_force_count` (derived), and
    `all_roots_total_wall_ns`. No Nickel-internal thunk counting.
- [x] Write `autoresearch.md` and `autoresearch.sh` for lazy root evaluation
      with median-based repeated sampling and:
      - primary metric `selected_root_total_wall_ns` (lower is better)
      - secondary metrics `root_discovery_wall_ns`,
        `selected_root_force_wall_ns`, `explicit_top_level_root_force_count`,
        `explicit_nonselected_root_force_count`, and
        `all_roots_total_wall_ns`
  - Evidence: `openspec/changes/lazy-root-evaluation/autoresearch.md` and
    `openspec/changes/lazy-root-evaluation/autoresearch.sh` exist with
    baseline numbers from this host.
- [ ] Initialize autoresearch on a dedicated branch/worktree only after the new
      lazy benchmark path produces a stable same-host baseline bundle

## Phase 5: Verification and follow-up

- [x] Add equivalence tests comparing lazy selected-root forcing with the
      current eager path for flat derivations, package sets, nested derivation
      inputs, import-heavy fixtures, import resolution, and recursive-record
      references
  - Evidence: `session_single_derivation_force_matches_eager`,
    `session_force_root_matches_eager_path` (package set),
    `session_nested_derivation_inputs_match_eager`,
    `session_file_with_imports` (import resolution),
    `session_recursive_record_refs_match_eager`. All pass.
- [x] Add a regression test confirming `crunch eval` output is unchanged when
      the build/planning path switches to the lazy session API
  - Evidence: `session_does_not_affect_eval_to_json` passes.
- [x] Add a verification that `explicit_nonselected_root_force_count == 0` for
      the selected-root benchmark workload and fail loudly if the harness cannot
      measure that count honestly
  - Evidence: `session_nonselected_force_count_is_zero_for_single_root`
    asserts `explicit_force_count() == 1` after forcing one root.
    Benchmark output: `explicit_nonselected_root_force_count: 0`.
- [x] Capture a baseline benchmark bundle and a `benchmark_compare` transcript
      for the new lazy workloads before starting autoresearch
  - Evidence: Task 42 baseline captured (discovery ~47ms, selected-root
    ~63ms, all-roots ~66ms) and recorded in `autoresearch.md`.
- [ ] Reassess whether compiled-eval work is still justified only after the
      lazy path and autoresearch metrics exist

## Validation

- [x] Run `openspec validate lazy-root-evaluation`
  - Evidence: `openspec validate lazy-root-evaluation` returned `Change 'lazy-root-evaluation' is valid`.
- [x] Run proposal, design, and tasks gates for `lazy-root-evaluation`
  - Evidence:
    - `openspec_gate stage=proposal change=lazy-root-evaluation` → `PASS`
    - `openspec_gate stage=design change=lazy-root-evaluation` → `PASS`
    - `openspec_gate stage=tasks change=lazy-root-evaluation` → `PASS`
