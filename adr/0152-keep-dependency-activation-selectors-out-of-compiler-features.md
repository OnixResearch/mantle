# 0152. Keep dependency activation selectors out of compiler features

Status: Accepted

## Context

The original-minicargo route retains dependency activation markers such as
`dep:core` while resolving optional edges. It published that whole internal set
as `Package.features`. Both compiler cfg and original-script environment
authoring consume that field, producing a static invocation containing
`CARGO_FEATURE_DEP:CORE`. The unchanged recipe consumer rejects the colon in
that environment name.

Renaming or dropping the invalid environment variable alone would leave a
bogus compiler feature and would conceal the selection/publication mismatch.
Dropping the marker before dependency selection would instead lose required
optional dependencies.

## Decision Drivers

- Preserve original dependency selection and feature forwarding.
- Publish actual named features consistently to cfg, script environments and
  downstream package consumers.
- Reuse the successor resolver's existing selector-publication convention.
- Keep consumer name/path validation and original source manifests unchanged.

## Decision

The minicargo resolver retains its complete activation state for selecting
optional dependencies and forwarding features, but excludes `dep:` activation
markers and dependency/feature selectors from published `Package.features`.
This matches the existing successor resolver boundary. Emitters continue to
consume the selected feature set; no colon-to-underscore alias, consumer
exception or environment-only filter is introduced.

A real resolver regression enables a named feature that activates an optional
`core` dependency. It requires the named feature and dependency to survive,
without publishing the activation token. With that feature disabled, it
requires both the dependency edge and selected dependency package to remain
absent. The unchanged helper admission regression runs alongside it.

## Consequences

- The original-minicargo resolver owns this distinction. The source-built test
  leg is its repeatable regression rail; compiler/environment emitters retain
  one feature vocabulary rather than maintaining separate filters.
- Ordinary development task 1603 executed 267 source-built tests with no
  failures and strict release Clippy. Full commands/results are retained under
  the active fixed-point change. This is not a frozen regular-runner build.
- Product source and authored program identities change. Freeze/qualify a
  replacement runner and regenerate/review its actual cohort; the rejected
  `c22381a8…` cohort is not repaired in place.
- No source pin, original manifest, optional-edge policy, resource bound,
  hermeticity rule, no-fetch/no-fallback policy or consumer admission is relaxed.
  No provider execution, compiler correctness, fixed-point equality or release
  eligibility is claimed by this repair.
