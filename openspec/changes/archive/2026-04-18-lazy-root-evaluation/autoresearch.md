# Autoresearch: Lazy root evaluation

## Session

- **Name:** lazy-root-eval
- **Primary metric:** `selected_root_total_wall_ns` (lower is better)
- **Unit:** ns
- **Direction:** lower

## Secondary metrics

- `root_discovery_wall_ns` — time to open session and discover labels
- `selected_root_force_wall_ns` — time to deep-force one selected root
- `explicit_top_level_root_force_count` — should be 1 for single-root
- `explicit_nonselected_root_force_count` — should be 0 for single-root
- `all_roots_total_wall_ns` — guardrail: all-roots path must not regress

## Workload

Fixed wide package-set fixture: `tests/fixtures/wide_package_set.ncl`
(16 derivation roots, record-of-derivations shape)

Selected root: `alpha`

## Sampling strategy

- 10 repeated samples per workload per run
- Report median of samples (avoids outlier noise)
- Cold per sample: fresh `EvaluationSession` each iteration

## Baseline (2026-04-17, this host)

```
lazy-root-discovery-wide-package-set:   ~47ms
lazy-selected-root-wide-package-set:    ~63ms
eager-all-roots-wide-package-set:       ~66ms
explicit_top_level_root_force_count:    1
explicit_nonselected_root_force_count:  0
```

## What to optimize

The primary target is `selected_root_total_wall_ns`. Current breakdown:
- ~75% discovery (shallow eval + label extraction)
- ~25% forcing (deep eval + deserialization)

Potential directions (for future autoresearch loop):
1. Reduce discovery overhead by caching the shallow context
2. Force only the selected root's subtree instead of whole-program deep export
3. Batch sibling discovery without full contract validation
4. Profile Nickel's `eval_shallow` for unnecessary work on wide records
