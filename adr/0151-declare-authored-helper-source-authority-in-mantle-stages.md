# 0151. Declare authored helper source authority in both Mantle stages

Status: Accepted

## Context

The current source-only cohort's three Mantle helpers per stage use exact
`ManifestSource` references to the `source-built-mantle-programs` record inside
the independently authenticated native source bundle. The shared Mantle-stage
input declaration listed Mantle source and vendor inputs, but omitted that
bundle. The unchanged recipe consumer rejects these six references as an
undeclared source role before helper execution.

The helpers are packaged separately from the Mantle recipe/source files. Global
source admission does not grant every stage permission to consume every role.
A prepared source profile and successful transfer therefore cannot resolve this
stage-local authority mismatch.

## Decision Drivers

- Preserve exact source-record/member ownership for authored helpers.
- Keep stage authority explicit and reject undeclared source roles, including
  already hydrated inputs.
- Repair the stage plan rather than weakening recipe admission, importing a
  helper executable or introducing a second source-copy convention.
- Retain failed cohort observations and require a newly bound plan/cohort after
  any product-source change.

## Decision

`mantle_build_inputs` declares `NativeSourceBundle` for both Mantle stages. The
existing exact `ManifestSource` helper references, record/member validation,
source digest checks and undeclared-role rejection remain unchanged. This is
source authority over the admitted bundle, not provider or executable authority.
Helpers still compile through the declared produced Rust compiler and consume
their original authenticated source members.

Do not relocate duplicate helper copies into Mantle source merely to avoid
naming their actual source owner. Do not infer stage permission from global
availability or from a populated hydration cache.

The recipe-admission regression materializes a real local source record and
lowers its exact manifest member under each Mantle stage. It requires the
resulting source authority and physical member, then removes the stage
permission and requires rejection even with that member already hydrated.

## Consequences

- The fixed-point planner and Mantle author/recipe-admission boundary own this
  contract; the source-built test leg is the repeatable regression rail.
- The stage-plan identity changes. Existing source-only cohorts and old runner
  or review identities do not become current by editing labels. Freeze and
  qualify a replacement runner/cohort and review its actual inputs before launch.
- The first development verification executed 266 source-built tests with no
  failures and passed strict release Clippy; its complete command/log is retained
  under the active fixed-point change. This ordinary development observation is
  not an isolated frozen regular-runner build or protected execution.
- Source/provider pins, strict hermeticity, no-fetch/no-fallback policy, resource
  bounds and proof/receipt requirements are unchanged. No compiler correctness,
  runtime success, fixed-point equality or release eligibility follows from this
  declaration repair.
