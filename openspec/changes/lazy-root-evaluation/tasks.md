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

- [ ] Implement an `EvaluationSession`-style API in `crunch-eval` that can open
      a file, list top-level root labels, and force one selected root on demand
- [ ] Keep the existing whole-program export path available for `crunch eval`
      and other debug/reporting callers while routing new lazy consumers
      through the session API
- [ ] Add unit tests proving root discovery for single derivations, arrays, and
      records without explicit forcing of unrelated top-level roots, and verify
      that array labels come from derivation `name` fields while record labels
      come from top-level field names
- [ ] Add a boundary guard that proves the lazy session type is defined in
      `crunch-eval` and consumed from there, not reimplemented in
      `crunch-pipeline`

## Phase 3: Consumer integration

- [ ] Switch `src/build_plan.rs` to the lazy session API first and verify that
      plan entries match the current eager path on checked-in fixtures
- [ ] Switch `crates/crunch-pipeline/src/lib.rs` to discover root labels before
      forcing per-root derivation values
- [ ] Keep full-root build semantics correct when all roots are selected, even
      though the pipeline no longer requires a whole-program deep export up
      front

## Phase 4: Benchmarks and autoresearch setup

- [ ] Add one checked-in wide package-set benchmark fixture sized for honest
      selected-root latency measurement rather than sub-millisecond noise, and
      record a verification step showing the fixture is above the chosen noise
      floor on the reference host before autoresearch begins
  - Note: if the fixture uses checked-in seed-backed conversion inputs, keep it
    `/nix/store`-compatible rather than relying on the default benchmark prefix
- [ ] Extend the benchmark suite with lazy root discovery, lazy selected-root,
      and eager/all-roots workloads on that same fixture
- [ ] Make the benchmark suite emit the lazy-eval metric set from design
      decision 4 without inferring hidden Nickel thunk activity
- [ ] Write `autoresearch.md` and `autoresearch.sh` for lazy root evaluation
      with median-based repeated sampling and:
      - primary metric `selected_root_total_wall_ns` (lower is better)
      - secondary metrics `root_discovery_wall_ns`,
        `selected_root_force_wall_ns`, `explicit_top_level_root_force_count`,
        `explicit_nonselected_root_force_count`, and
        `all_roots_total_wall_ns`
- [ ] Initialize autoresearch on a dedicated branch/worktree only after the new
      lazy benchmark path produces a stable same-host baseline bundle

## Phase 5: Verification and follow-up

- [ ] Add equivalence tests comparing lazy selected-root forcing with the
      current eager path for flat derivations, package sets, nested derivation
      inputs, import-heavy fixtures, import resolution, and recursive-record
      references
- [ ] Add a regression test confirming `crunch eval` output is unchanged when
      the build/planning path switches to the lazy session API
- [ ] Add a verification that `explicit_nonselected_root_force_count == 0` for
      the selected-root benchmark workload and fail loudly if the harness cannot
      measure that count honestly
- [ ] Capture a baseline benchmark bundle and a `benchmark_compare` transcript
      for the new lazy workloads before starting autoresearch
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
