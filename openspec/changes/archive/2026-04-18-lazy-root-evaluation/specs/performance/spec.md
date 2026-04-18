## ADDED Requirements

### Requirement: Lazy root evaluation benchmarks use honest selected-root metrics

The benchmark suite MUST provide checked-in lazy-evaluation workloads on a
fixed multi-root fixture and MUST emit machine-readable metrics that reflect the
`crunch-eval` API boundary honestly.

At minimum the suite MUST include the following checked-in workload names on
one shared fixture:
- `lazy-root-discovery-wide-package-set`,
- `lazy-selected-root-wide-package-set`,
- and `eager-all-roots-wide-package-set`.

The benchmark/autoresearch metric set MUST include:
- `selected_root_total_wall_ns` as the primary selected-root latency metric,
- `root_discovery_wall_ns`,
- `selected_root_force_wall_ns`,
- `explicit_top_level_root_force_count`,
- `explicit_nonselected_root_force_count`,
- and `all_roots_total_wall_ns`.

`selected_root_total_wall_ns` MUST be defined as the inclusive end-to-end wall
clock from opening the lazy session to obtaining one fully materialized selected
root. It MAY therefore be larger than
`root_discovery_wall_ns + selected_root_force_wall_ns` when session setup or
other caller-visible overhead exists.

`explicit_top_level_root_force_count` and
`explicit_nonselected_root_force_count` MUST count only top-level root values
that crunch explicitly forces through the lazy API. The harness MUST NOT infer
or guess hidden internal Nickel thunk activity and report it as those metrics.

The shared wide package-set fixture MUST be large enough, or repeated-sampled
enough, that the selected-root primary metric is stable above the noise floor
on the reference host before autoresearch begins.

#### Scenario: Selected-root benchmark reports the full lazy metric set

- GIVEN a maintainer runs the checked-in lazy selected-root benchmark
- WHEN the result bundle or autoresearch script output is inspected
- THEN it includes `selected_root_total_wall_ns` and the supporting lazy-eval
  metrics
- AND those metrics are defined at the `crunch-eval` discovery/force boundary,
  not by inferred Nickel-internal events

#### Scenario: Non-selected root force count stays explicit

- GIVEN a lazy selected-root benchmark run on a multi-root fixture
- WHEN crunch forces only one requested root
- THEN `explicit_nonselected_root_force_count` reports how many top-level roots
  other than the selected one crunch explicitly forced
- AND the metric is omitted or fails loudly if the harness cannot measure that
  count honestly

### Requirement: Autoresearch targets selected-root latency first

Any future autoresearch loop for lazy root evaluation MUST optimize
`selected_root_total_wall_ns` first and treat the remaining lazy-eval metrics as
secondary guardrails.

The autoresearch harness MUST use a workload and sampling strategy that avoids
sub-millisecond noise dominating decisions.

#### Scenario: Autoresearch session uses selected-root metric as primary target

- GIVEN an autoresearch session is created for lazy root evaluation
- WHEN the session configuration is inspected
- THEN the primary metric is `selected_root_total_wall_ns`
- AND the session keeps the lazy-eval discovery/force counts and
  `all_roots_total_wall_ns` as secondary metrics or checks
- AND the workload or repeated-sampling strategy is chosen so the primary
  metric is stable enough to compare across runs on the same host
